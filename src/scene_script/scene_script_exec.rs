use crate::actor::{Actor, ActorFlags, Direction};
use crate::Context;
use crate::map::Map;
use crate::scene::scene_map::SceneMap;
use crate::scene_script::ops::Op;
use crate::scene_script::ops_char_load::CharacterType;
use crate::scene_script::ops_jump::CompareOp;
use crate::scene_script::ops_math::{BitMathOp, ByteMathOp};
use crate::scene_script::scene_script::{ActorScriptState, OpResult};
use crate::scene_script::scene_script_decoder::CopyTilesFlags;
use crate::scene_script::scene_script_memory::SceneScriptMemory;

pub fn op_execute(ctx: &mut Context, state: &mut ActorScriptState, this_actor: usize, actors: &mut Vec<Actor>, map: &mut Map, scene_map: &mut SceneMap, memory: &mut SceneScriptMemory) -> OpResult {
    let op = match state.current_op {
        Some(op) => op,
        None => return (true, true),
    };

    match op {
        Op::NOP => (false, true),
        Op::Yield { forever } => {
            (true, !forever)
        },

        // todo return never completes for now, but should move execution elsewhere.
        Op::Return => (true, false),

        // Copy.
        Op::Copy8 { source, dest } => {
            dest.put_u8(memory, source.get_u8(memory));
            (false, true)
        },
        Op::Copy16 { source, dest } => {
            dest.put_u16(memory, source.get_u16(memory));
            (false, true)
        },
        Op::CopyBytes { dest, bytes, length } => {
            dest.put_bytes(memory, bytes, length);
            (false, true)
        },

        // Jump.
        Op::Jump { offset } => {
            state.address = (state.address as i64 + offset - 1) as u64;
            (false, true)
        },
        Op::JumpConditional8 { lhs, cmp, rhs, offset } => {
            let lhs_value = lhs.get_u8(memory);
            let rhs_value = rhs.get_u8(memory);
            let result = match cmp {
                CompareOp::Eq => lhs_value == rhs_value,
                CompareOp::NotEq => lhs_value != rhs_value,
                CompareOp::Gt => lhs_value > rhs_value,
                CompareOp::GtEq => lhs_value >= rhs_value,
                CompareOp::Lt => lhs_value < rhs_value,
                CompareOp::LtEq => lhs_value <= rhs_value,
                CompareOp::And => lhs_value & rhs_value > 0,
                CompareOp::Or => lhs_value | rhs_value > 0,
            };
            if !result {
                state.address = (state.address as i64 + offset - 1) as u64;
            }

            (false, true)
        },

        // Math.
        Op::ByteMath8 { dest, lhs, op, rhs } => {
            let lhs_value = lhs.get_u8(memory);
            let rhs_value = rhs.get_u8(memory);

            let result = match op {
                ByteMathOp::Add => lhs_value.overflowing_add(rhs_value).0,
                ByteMathOp::Subtract => lhs_value.overflowing_sub(rhs_value).0,
            };
            dest.put_u8(memory, result);

            (false, true)
        },
        Op::ByteMath16 { dest, lhs, op, rhs } => {
            let lhs_value = lhs.get_u16(memory);
            let rhs_value = rhs.get_u16(memory);

            let result = match op {
                ByteMathOp::Add => lhs_value.overflowing_add(rhs_value).0,
                ByteMathOp::Subtract => lhs_value.overflowing_sub(rhs_value).0,
            };
            dest.put_u16(memory, result);

            (false, true)
        },
        Op::BitMath { dest, lhs, op, rhs } => {
            let lhs_value = lhs.get_u8(memory);
            let rhs_value = rhs.get_u8(memory);

            let result = match op {
                BitMathOp::And => lhs_value & rhs_value,
                BitMathOp::Or => lhs_value | rhs_value,
                BitMathOp::Xor => lhs_value ^ rhs_value,
                BitMathOp::ShiftLeft => lhs_value << rhs_value,
                BitMathOp::ShiftRight => lhs_value >> rhs_value,
            };
            dest.put_u8(memory, result);

            (false, true)
        },

        // todo must_be_in_party
        Op::LoadCharacter { char_type, index, is_static, battle_index, .. } => {
            let real_index = match char_type {
                CharacterType::PC => index,
                CharacterType::PCAsNPC => index,
                CharacterType::NPC => index + 7,
                CharacterType::Enemy => index + 256, // todo another +7 for PC version, does it store more NPC sprites?
            };

            ctx.sprite_assets.load(&ctx.fs, real_index);

            actors[this_actor].battle_index = battle_index;
            actors[this_actor].flags |= ActorFlags::RENDERED | ActorFlags::VISIBLE;
            if is_static {
                actors[this_actor].flags |= ActorFlags::BATTLE_STATIC;
            }

            if char_type == CharacterType::PC {
                actors[this_actor].player_index = Some(index);
            }

            let state = &mut ctx.sprites_states.get_state_mut(this_actor);
            state.enabled = true;
            state.sprite_index = real_index;
            ctx.sprites_states.set_animation(&ctx.sprite_assets, this_actor, 0, true);

            (false, true)
        },

        Op::ActorCoordinatesSet { actor, x, y } => {
            let actor_index = actor.deref(this_actor);
            let x = x.get_u8(memory) as f64;
            let y = y.get_u8(memory) as f64;

            actors[actor_index].x = x * 16.0 + 8.0;
            actors[actor_index].y = y * 16.0 + 16.0;

            // Set sprite priority from scene map properties.
            let tile_x = (actors[actor_index].x / 16.0) as u32;
            let tile_y = (actors[actor_index].y / 16.0 - 1.0) as u32;
            let index = (tile_y * scene_map.props.width + tile_x) as usize;
            if index < scene_map.props.props.len() {
                if let Some(sprite_priority) = scene_map.props.props[index].sprite_priority {
                    actors[actor_index].sprite_priority_top = sprite_priority;
                    actors[actor_index].sprite_priority_bottom = sprite_priority;
                }
            }

            (false, true)
        },

        Op::ActorUpdateFlags { actor, set, remove } => {
            let actor_index = actor.deref(this_actor);
            actors[actor_index].flags |= set;
            actors[actor_index].flags.remove(remove);

            (false, true)
        },

        Op::ActorCoordinatesSetPrecise { actor, x, y } => {
            let actor_index = actor.deref(this_actor);
            let x = x.get_u16(memory) as f64;
            let y = y.get_u16(memory) as f64;

            actors[actor_index].x = x;
            actors[actor_index].y = y + 1.0;

            // Set sprite priority from scene map properties.
            let tile_x = (actors[actor_index].x / 16.0) as u32;
            let tile_y = (actors[actor_index].y / 16.0 - 1.0) as u32;
            let index = (tile_y * scene_map.props.width + tile_x) as usize;
            if index < scene_map.props.props.len() {
                if let Some(sprite_priority) = scene_map.props.props[index].sprite_priority {
                    actors[actor_index].sprite_priority_top = sprite_priority;
                    actors[actor_index].sprite_priority_bottom = sprite_priority;
                }
            }

            (false, true)
        },

        Op::ActorSetDirection { actor, direction } => {
            let actor_index = actor.deref(this_actor);
            let direction = Direction::from_index(direction.get_u8(memory) as usize);
            actors[actor_index].direction = direction;
            ctx.sprites_states.set_direction(&ctx.sprite_assets, actor_index, direction);

            (false, true)
        },

        Op::ActorSetSpriteFrame { actor, frame } => {
            let actor_index = actor.deref(this_actor);
            let frame_index = frame.get_u8(memory) as usize;
            ctx.sprites_states.set_sprite_frame(actor_index, frame_index);

            (false, true)
        },

        // todo mode, unknowns
        Op::ActorSetSpritePriority { actor, top, bottom, .. } => {
            let actor_index = actor.deref(this_actor);
            actors[actor_index].sprite_priority_top = top;
            actors[actor_index].sprite_priority_bottom = bottom;

            (false, true)
        },

        // todo loops, wait
        Op::Animate { actor, animation, run, .. } => {
            let actor_index = actor.deref(this_actor);
            let anim_index = animation.get_u8(memory) as usize;
            ctx.sprites_states.set_animation(&ctx.sprite_assets, actor_index, anim_index, run);

            (false, true)
        },

        // Copy tiles around on the map.
        Op::CopyTiles { left, top, right, bottom, dest_x, dest_y, flags } => {
            println!("CopyTiles: from {}x{} {}x{} to {}x{} with {:?}", left, top, right, bottom, dest_x, dest_y, flags);

            for (layer_index, layer) in map.layers.iter_mut().enumerate() {
                if layer_index == 0 && !flags.contains(CopyTilesFlags::COPY_L1) {
                    continue;
                }
                if layer_index == 1 && !flags.contains(CopyTilesFlags::COPY_L2) {
                    continue;
                }
                if layer_index == 2 && !flags.contains(CopyTilesFlags::COPY_L3) {
                    continue;
                }

                for chip_y in 0..bottom - top {
                    for chip_x in 0..right - left {

                        let src_chip_x =  chip_x + left;
                        let src_chip_y =  chip_y + top;
                        let src_chip_index = (src_chip_x + src_chip_y * layer.chip_width) as usize;

                        let dest_chip_x =  chip_x + dest_x;
                        let dest_chip_y =  chip_y + dest_y;
                        let dest_chip_index = (dest_chip_x + dest_chip_y * layer.chip_width) as usize;

                        layer.chips[dest_chip_index] = layer.chips[src_chip_index];

                        // todo copy flagged props
                    }
                }
            }

            (false, true)
        },

        Op::SetScriptDelay { delay } => {
            state.delay = delay;
            (false, true)
        },

        Op::Wait { ticks } => {
            // Convert from 1/16th intervals to 1/60th ticks.
            state.delay_counter = (ticks as f64 * (1.0 / 3.75)).ceil() as u32;
            println!("Delay: {} == {} ticks?", ticks, state.delay_counter);
            (true, true)
        },

        _ => (false, true),
    }
}
