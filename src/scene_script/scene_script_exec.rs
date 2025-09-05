use std::f64::consts::PI;
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

            actors[actor_index].move_to(x * 16.0 + 8.0, y * 16.0 + 16.0, true);

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
            actors[actor_index].move_to(x, y + 1.0, true);

            // Set sprite priority from scene map properties.
            let props = scene_map.get_props_at_coordinates(actors[actor_index].x, actors[actor_index].y - 1.0);
            if let Some(props) = props {
                if let Some(sprite_priority) = props.sprite_priority {
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

        Op::ActorSetSpeed { actor, speed } => {
            let actor_index = actor.deref(this_actor);
            actors[actor_index].move_speed = speed.get_u8(memory) as f64 / 32.0;

            (false, true)
        },

        Op::Animate { actor, animation, run, loops, wait } => {
            let actor_index = actor.deref(this_actor);
            let anim_index = animation.get_u8(memory) as usize;
            let loops = loops.get_u8(memory) as u32;

            let state = ctx.sprites_states.get_state(actor_index);
            if state.anim_index != anim_index {
                ctx.sprites_states.set_animation(&ctx.sprite_assets, actor_index, anim_index, run);
            } else if wait && loops < 0xFFFFFFFF && state.anim_loop_count <= loops {
                return (false, true);
            }

            (false, true)
        },

        Op::ActorMoveTo { actor, x, y, animated, distance, update_direction } => {
            let actor_index = actor.deref(this_actor);
            let actor = actors.get_mut(actor_index).unwrap();
            let distance = distance.get_u8(memory) as f64;

            let dest_x = x.get_u8(memory) as f64 * 16.0 + 8.0;
            let dest_y = y.get_u8(memory) as f64 * 16.0 + 16.0;

            if actor.flags.contains(ActorFlags::MOVE_ONTO_TILE) {
                // todo move onto tile does what exactly?
            }
            if actor.flags.contains(ActorFlags::MOVE_ONTO_ACTOR) {
                // todo move onto actor does what exactly?
            }

            let diff_x = dest_x - actor.x;
            let diff_y = dest_y - actor.y;

            // Set walking animation (assumed to be animation 1).
            let state = ctx.sprites_states.get_state(actor_index);
            if state.anim_index != 1 {
                ctx.sprites_states.set_animation(&ctx.sprite_assets, actor_index, 1, true);
            }

            // Actor must face the direction of movement.
            if update_direction {
                let mut angle = (diff_y.atan2(diff_x) * 180.0 / PI) - 45.0;
                if angle < 0.0 {
                    angle += 360.0;
                }
                let direction = match (angle / 90.0).floor() as u32 {
                    0 => Direction::Down,
                    1 => Direction::Left,
                    2 => Direction::Up,
                    3 => Direction::Right,
                    _ => Direction::Up,
                };
                actor.direction = direction;
                ctx.sprites_states.set_direction(&ctx.sprite_assets, actor_index, direction);
            }

            // todo keep distance
            // todo animated does what?

            // Calculate steps needed based on the longest side.
            let mut step_count = if diff_x.abs() > diff_y.abs() {
                (diff_x / actor.move_speed).abs()
            } else {
                (diff_y / actor.move_speed).abs()
            };

            // Almost there, end the movement.
            if step_count <= 2.0 {
                actor.x = dest_x;
                actor.y = dest_y;
                actor.set_velocity(0.0, 0.0);
                return (false, true);
            }

            // Slow down at the end of fast movements.
            if actor.move_speed > 1.0 && step_count < 8.0 {
                step_count += 4.0;
            }
            actor.set_velocity(diff_x / step_count, diff_y / step_count);

            // Set sprite priority from scene map properties.
            let props = scene_map.get_props_at_coordinates(actor.x, actor.y - 1.0);
            if let Some(props) = props {
                if let Some(sprite_priority) = props.sprite_priority {
                    actor.sprite_priority_top = sprite_priority;
                    actor.sprite_priority_bottom = sprite_priority;
                }
            }

            (true, false)
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

                        let src_chip_x = chip_x + left;
                        let src_chip_y = chip_y + top;
                        let src_chip_index = (src_chip_x + src_chip_y * layer.chip_width) as usize;

                        let dest_chip_x = chip_x + dest_x;
                        let dest_chip_y = chip_y + dest_y;
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
            state.delay_counter = (ticks as f64 * 3.75).ceil() as u32;
            (true, true)
        },

        _ => (false, true),
    }
}
