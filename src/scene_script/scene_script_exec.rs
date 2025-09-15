use crate::actor::{Actor, ActorFlags, DebugSprite, Direction};
use crate::Context;
use crate::map::Map;
use crate::scene::scene_map::SceneMap;
use crate::scene_script::exec_movement::{exec_movement_tile, exec_movement_vector};
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
        None => return OpResult::YIELD | OpResult::COMPLETE,
    };

    match op {
        Op::NOP => OpResult::COMPLETE,
        Op::Yield { forever } => {
            if !forever {
                OpResult::YIELD | OpResult::COMPLETE
            } else {
                OpResult::YIELD
            }
        },

        // todo return never completes for now, but should move execution elsewhere.
        Op::Return => OpResult::YIELD,

        // Copy.
        Op::Copy8 { source, dest } => {
            dest.put_u8(memory, source.get_u8(memory));
            OpResult::COMPLETE
        },
        Op::Copy16 { source, dest } => {
            dest.put_u16(memory, source.get_u16(memory));
            OpResult::COMPLETE
        },
        Op::CopyBytes { dest, bytes, length } => {
            dest.put_bytes(memory, bytes, length);
            OpResult::COMPLETE
        },

        // Jump.
        Op::Jump { offset } => {
            state.address = (state.address as i64 + offset - 1) as u64;
            OpResult::COMPLETE
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

            OpResult::COMPLETE
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

            OpResult::COMPLETE
        },
        Op::ByteMath16 { dest, lhs, op, rhs } => {
            let lhs_value = lhs.get_u16(memory);
            let rhs_value = rhs.get_u16(memory);

            let result = match op {
                ByteMathOp::Add => lhs_value.overflowing_add(rhs_value).0,
                ByteMathOp::Subtract => lhs_value.overflowing_sub(rhs_value).0,
            };
            dest.put_u16(memory, result);

            OpResult::COMPLETE
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

            OpResult::COMPLETE
        },

        // todo must_be_in_party
        Op::LoadCharacter { char_type, index, is_static, battle_index, .. } => {
            let sprite_index = match char_type {
                CharacterType::PC => index,
                CharacterType::PCAsNPC => index,
                CharacterType::NPC => index + 7,
                CharacterType::Enemy => index + 256,
            };

            let actor = actors.get_mut(this_actor).unwrap();

            actor.battle_index = battle_index;
            actor.flags |= ActorFlags::RENDERED | ActorFlags::VISIBLE | ActorFlags::SOLID;
            if is_static {
                actor.flags |= ActorFlags::BATTLE_STATIC;
            }

            if char_type == CharacterType::PC {
                actor.player_index = Some(index);
            }

            let state = &mut ctx.sprites_states.get_state_mut(this_actor);
            ctx.sprite_assets.load(&ctx.fs, sprite_index);
            state.sprite_index = sprite_index;
            state.enabled = true;
            ctx.sprites_states.set_animation(&ctx.sprite_assets, this_actor, 0, true, actor.direction);

            OpResult::COMPLETE
        },

        Op::ActorCoordinatesSet { actor, x, y } => {
            let actor_index = actor.deref(this_actor);
            let x = x.get_u8(memory) as f64;
            let y = y.get_u8(memory) as f64;

            actors[actor_index].move_to(x * 16.0 + 8.0, y * 16.0 + 16.0, true, &scene_map);

            OpResult::COMPLETE
        },

        Op::ActorUpdateFlags { actor, set, remove } => {
            let actor_index = actor.deref(this_actor);
            actors[actor_index].flags |= set;
            actors[actor_index].flags.remove(remove);

            OpResult::COMPLETE
        },

        Op::ActorCoordinatesSetPrecise { actor, x, y } => {
            let actor_index = actor.deref(this_actor);
            let x = x.get_u16(memory) as f64;
            let y = y.get_u16(memory) as f64;

            actors[actor_index].move_to(x, y, true, &scene_map);

            OpResult::COMPLETE
        },

        Op::ActorSetDirection { actor, direction } => {
            let actor_index = actor.deref(this_actor);
            let direction = Direction::from_index(direction.get_u8(memory) as usize);
            actors[actor_index].direction = direction;
            ctx.sprites_states.set_direction(&ctx.sprite_assets, actor_index, direction);

            OpResult::YIELD | OpResult::COMPLETE
        },

        Op::ActorSetSpriteFrame { actor, frame } => {
            let actor_index = actor.deref(this_actor);
            let frame_index = frame.get_u8(memory) as usize;

            ctx.sprites_states.set_sprite_frame(actor_index, frame_index);

            OpResult::COMPLETE
        },

        // todo mode, unknowns
        Op::ActorSetSpritePriority { actor, top, bottom, .. } => {
            let actor_index = actor.deref(this_actor);
            actors[actor_index].sprite_priority_top = top;
            actors[actor_index].sprite_priority_bottom = bottom;

            OpResult::COMPLETE
        },

        Op::ActorSetSpeed { actor, speed } => {
            let actor_index = actor.deref(this_actor);
            actors[actor_index].move_speed = speed.get_u8(memory) as f64 / 16.0;
            OpResult::COMPLETE
        },

        Op::Animate { actor, animation, run, loops, wait } => {
            let actor_index = actor.deref(this_actor);
            let anim_index = animation.get_u8(memory) as usize;
            let loops = loops.get_u8(memory) as u32;

            let state = ctx.sprites_states.get_state(actor_index);

            // Start animating.
            if state.anim_index != anim_index {
                ctx.sprites_states.set_animation(&ctx.sprite_assets, actor_index, anim_index, run, actors[actor_index].direction);
                return if wait {
                    actors[actor_index].debug_sprite = DebugSprite::Animating;
                    OpResult::YIELD
                } else {
                    OpResult::COMPLETE
                }

            // Wait for animation to complete.
            } else if wait && loops < 0xFFFFFFFF && state.anim_loop_count <= loops {
                return OpResult::YIELD;
            }

            // Wait completed.
            actors[actor_index].debug_sprite = DebugSprite::None;
            OpResult::COMPLETE
        },

        Op::ActorMoveAtAngle { actor, angle, steps, update_direction, animated } => {
            let actor_index = actor.deref(this_actor);
            let angle = angle.get_u8(memory) as f64 * 1.40625;
            let steps = steps.get_u8(memory) as u32;

            exec_movement_vector(ctx, actor_index, actors, angle, steps, update_direction, animated)
        },

        Op::ActorMoveTo { actor, x, y, steps, update_direction, animated } => {
            let actor_index = actor.deref(this_actor);
            let dest_tile_x = x.get_u8(memory) as i32;
            let dest_tile_y = y.get_u8(memory) as i32;
            let steps = if let Some(steps) = steps { Some(steps.get_u8(memory) as u32) } else { None };

            exec_movement_tile(ctx, state, actor_index, actors, dest_tile_x, dest_tile_y, steps, update_direction, animated)
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

            OpResult::COMPLETE
        },

        Op::SetScriptDelay { delay } => {
            state.delay = delay + 1;
            state.delay_counter = delay + 1;
            OpResult::COMPLETE
        },

        Op::Wait { actor, ticks } => {
            let actor_index = actor.deref(this_actor);
            let actor = actors.get_mut(actor_index).unwrap();

            // Start counting.
            if state.pause_counter == 0 {
                state.pause_counter = 1;
                actor.debug_sprite = DebugSprite::Waiting;
                return OpResult::YIELD;

            // Count one more tick.
            } else if state.pause_counter <= ticks {
                state.pause_counter += 1;
                return OpResult::YIELD;
            }

            // Finished counting.
            state.pause_counter = 0;
            actor.debug_sprite = DebugSprite::None;
            OpResult::COMPLETE
        },

        Op::Control { forever } => {
            if forever {
                OpResult::YIELD
            } else {
                OpResult::YIELD | OpResult::COMPLETE
            }
        },

        _ => OpResult::COMPLETE,
    }
}
