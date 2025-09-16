use crate::actor::{Actor, ActorFlags, ActorTask, DebugSprite, Direction};
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

pub fn op_execute(ctx: &mut Context, this_actor: usize, state: &mut ActorScriptState, states: &mut Vec<ActorScriptState>, actors: &mut Vec<Actor>, map: &mut Map, scene_map: &mut SceneMap, memory: &mut SceneScriptMemory) -> OpResult {
    let op = match state.current_op {
        Some(op) => op,
        None => return OpResult::YIELD | OpResult::COMPLETE,
    };

    match op {
        Op::NOP => OpResult::COMPLETE,

        Op::Yield { forever } => {
            if forever {
                OpResult::YIELD
            } else {
                OpResult::YIELD | OpResult::COMPLETE
            }
        },

        Op::Return => {

            // If the current priority is the least urgent, just yield.
            // Priority 7 is what the object init scripts are started at.
            if state.current_priority == 7 {
                return OpResult::YIELD | OpResult::COMPLETE;
            }

            // Remove the current priority pointer.
            state.priority_return_ptrs[state.current_priority] = 0;

            // Find the next non-zero lower priority pointer and use that as the current priority.
            // Priority 7 should always be available, as that is the init function.
            for priority_index in state.current_priority + 1..8 {
                state.current_priority = priority_index;
                state.current_address = state.priority_return_ptrs[priority_index];
                if state.priority_return_ptrs[priority_index] != 0 {
                    break;
                }
            }

            OpResult::COMPLETE | OpResult::JUMPED
        },

        Op::Call { actor, priority, function } => {
            let target = actor.deref(this_actor);
            let target_actor = &mut actors[target];
            let target_state = &mut states[target];

            // Complete immediately if the object is not interactive, dead or disabled.
            if !target_actor.flags.contains(ActorFlags::INTERACTABLE) {
                return OpResult::COMPLETE;
            }
            if target_actor.flags.contains(ActorFlags::DEAD) {
                return OpResult::COMPLETE;
            }
            if target_actor.flags.contains(ActorFlags::SCRIPT_DISABLED) {
                return OpResult::COMPLETE;
            }

            // Complete if the current priority is the same as the call priority.
            if target_state.current_priority == priority {
                return OpResult::COMPLETE;
            }

            // If the current priority is more important than the new one, store the new function
            // address at that priority pointer, if not already set. A future call to return can
            // then exit to the new function.
            if target_state.current_priority < priority {
                if target_state.priority_return_ptrs[target_state.current_priority] > 0 {
                    return OpResult::COMPLETE;
                }

                target_state.priority_return_ptrs[priority] = target_state.function_ptrs[function];
                return OpResult::COMPLETE;
            }

            // The new priority is more important than the current one. Set the priority pointer to
            // the new function and immediately continue execution there.
            target_state.priority_return_ptrs[target_state.current_priority] = target_state.current_address;
            target_state.current_address = target_state.function_ptrs[function];
            target_state.current_priority = priority;

            // The new function must interrupt any active movement/task.
            target_actor.task = ActorTask::None;

            OpResult::COMPLETE
        },

        Op::CallWaitCompletion { actor, priority, function } => {
            let target = actor.deref(this_actor);
            let target_actor = &mut actors[target];
            let target_state = &mut states[target];

            // Wait until a non-interactive target object becomes interactive.
            if !target_actor.flags.contains(ActorFlags::INTERACTABLE) {
                return OpResult::YIELD;
            }

            // Complete immediately if the object is dead or disabled.
            if target_actor.flags.contains(ActorFlags::DEAD) {
                return OpResult::COMPLETE;
            }
            if target_actor.flags.contains(ActorFlags::SCRIPT_DISABLED) {
                return OpResult::COMPLETE;
            }

            // Complete if the current priority is the same as the call priority.
            if target_state.current_priority <= priority {
                return OpResult::YIELD;
            }

            // The new priority is more important than the current one. Set the priority pointer to
            // the new function and immediately continue execution there.
            target_state.priority_return_ptrs[target_state.current_priority] = target_state.current_address;
            target_state.current_address = target_state.function_ptrs[function];
            target_state.current_priority = priority;

            // The new function must interrupt any active movement/task.
            target_actor.task = ActorTask::None;

            OpResult::COMPLETE
        },

        Op::CallWaitReturn { actor, priority, function } => {
            let target = actor.deref(this_actor);
            let target_actor = &mut actors[target];
            let target_state = &mut states[target];

            if !state.call_waiting {

                // Wait until a non-interactive target object becomes interactive.
                if !target_actor.flags.contains(ActorFlags::INTERACTABLE) {
                    return OpResult::YIELD;
                }

                // Complete immediately if the object is dead or disabled.
                if target_actor.flags.contains(ActorFlags::DEAD) {
                    return OpResult::COMPLETE;
                }
                if target_actor.flags.contains(ActorFlags::SCRIPT_DISABLED) {
                    return OpResult::COMPLETE;
                }

                // Wait until the target object is done executing a function of
                // the same or more importance.
                if target_state.current_priority <= priority {
                    return OpResult::YIELD;
                }

                // The new priority is more important than the current one. Set the priority pointer to the new function
                // and immediately continue execution there.
                target_state.priority_return_ptrs[target_state.current_priority] = target_state.current_address;
                target_state.current_address = target_state.function_ptrs[function];
                target_state.current_priority = priority;

                // The new function must interrupt any active movement.
                target_actor.task = ActorTask::None;

                // We are now waiting on the target object to finish their function.
                state.call_waiting = true;

                return OpResult::YIELD;
            }

            // Complete immediately if the object is dead or disabled.
            if target_actor.flags.contains(ActorFlags::DEAD) {
                state.call_waiting = false;
                return OpResult::COMPLETE;
            }
            if target_actor.flags.contains(ActorFlags::SCRIPT_DISABLED) {
                state.call_waiting = false;
                return OpResult::COMPLETE;
            }

            // Wait until the target object is done executing our previously set
            // function call.
            if target_state.current_priority <= priority {
                return OpResult::YIELD;
            }

            // The call we were waiting for has completed.
            state.call_waiting = false;
            OpResult::COMPLETE
        },

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
            state.current_address = (state.current_address as i64 + offset) as u64;
            OpResult::COMPLETE | OpResult::JUMPED
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
                state.current_address = (state.current_address as i64 + offset) as u64;
                return OpResult::COMPLETE | OpResult::JUMPED;
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
