use crate::actor::{Actor, ActorClass, ActorFlags, DebugSprite};
use crate::Context;
use crate::facing::Facing;
use crate::map::Map;
use crate::scene::scene_map::SceneMap;
use crate::scene_script::exec::animation::{exec_animation, exec_animation_loop_count, exec_animation_reset, exec_animation_static_frame};
use crate::scene_script::exec::call::{exec_call, exec_call_return, exec_call_wait_completion, exec_call_wait_return};
use crate::scene_script::exec::movement::{exec_movement_to_tile, exec_movement_by_vector, exec_movement_to_actor};
use crate::scene_script::ops::Op;
use crate::scene_script::decoder::ops_char_load::CharacterType;
use crate::scene_script::decoder::ops_jump::CompareOp;
use crate::scene_script::decoder::ops_math::{BitMathOp, ByteMathOp};
use crate::scene_script::scene_script::{ActorScriptState, OpResult};
use crate::scene_script::scene_script_decoder::CopyTilesFlags;
use crate::scene_script::scene_script_memory::SceneScriptMemory;
use crate::util::vec2df64::Vec2Df64;
use crate::util::vec2di32::Vec2Di32;

pub fn op_execute(ctx: &mut Context, this_actor: usize, state: &mut ActorScriptState, states: &mut Vec<ActorScriptState>, actors: &mut Vec<Actor>, map: &mut Map, scene_map: &mut SceneMap, memory: &mut SceneScriptMemory, mut dialogue: &mut Vec<String>) -> OpResult {
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

        // Function calls.
        Op::Return => {
            exec_call_return(state)
        },
        Op::Call { actor, priority, function } => {
            let target_index = actor.deref(this_actor);
            let target_actor = &mut actors[target_index];
            let target_state = &mut states[target_index];

            exec_call(target_actor, target_state, function, priority)
        },
        Op::CallWaitCompletion { actor, priority, function } => {
            let target_index = actor.deref(this_actor);
            let target_actor = &mut actors[target_index];
            let target_state = &mut states[target_index];

            exec_call_wait_completion(target_actor, target_state, function, priority)
        },
        Op::CallWaitReturn { actor, priority, function } => {
            let target_index = actor.deref(this_actor);
            let target_actor = &mut actors[target_index];
            let target_state = &mut states[target_index];

            exec_call_wait_return(state, target_actor, target_state, function, priority)
        },

        // Copy.
        Op::Copy8 { source, dest } => {
            dest.put_u8(memory, source.get_u8(memory, &actors, this_actor));
            OpResult::COMPLETE
        },
        Op::Copy16 { source, dest } => {
            dest.put_u16(memory, source.get_u16(memory, &actors, this_actor));
            OpResult::COMPLETE
        },
        Op::CopyBytes { dest, bytes, length } => {
            dest.put_bytes(memory, bytes, length);
            OpResult::COMPLETE
        },

        // Control flow.
        Op::Jump { offset } => {
            state.current_address = (state.current_address as i64 + offset) as u64;
            OpResult::COMPLETE | OpResult::JUMPED
        },
        Op::JumpConditional8 { lhs, cmp, rhs, offset } => {
            let lhs_value = lhs.get_u8(memory, &actors, this_actor);
            let rhs_value = rhs.get_u8(memory, &actors, this_actor);
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
            let lhs_value = lhs.get_u8(memory, &actors, this_actor);
            let rhs_value = rhs.get_u8(memory, &actors, this_actor);

            let result = match op {
                ByteMathOp::Add => lhs_value.overflowing_add(rhs_value).0,
                ByteMathOp::Subtract => lhs_value.overflowing_sub(rhs_value).0,
            };
            dest.put_u8(memory, result);

            OpResult::COMPLETE
        },
        Op::ByteMath16 { dest, lhs, op, rhs } => {
            let lhs_value = lhs.get_u16(memory, &actors, this_actor);
            let rhs_value = rhs.get_u16(memory, &actors, this_actor);

            let result = match op {
                ByteMathOp::Add => lhs_value.overflowing_add(rhs_value).0,
                ByteMathOp::Subtract => lhs_value.overflowing_sub(rhs_value).0,
            };
            dest.put_u16(memory, result);

            OpResult::COMPLETE
        },
        Op::BitMath { dest, lhs, op, rhs } => {
            let lhs_value = lhs.get_u8(memory, &actors, this_actor);
            let rhs_value = rhs.get_u8(memory, &actors, this_actor);

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

            // todo: set remaining actor classes
            if char_type == CharacterType::PCAsNPC || char_type == CharacterType::NPC {
                actor.class = ActorClass::NPC;
            }
            if char_type == CharacterType::Enemy {
                actor.class = ActorClass::Monster;
            }

            let state = &mut ctx.sprites_states.get_state_mut(this_actor);
            let sprite_asset = ctx.sprite_assets.load(&ctx.fs, sprite_index);
            state.sprite_index = sprite_index;
            state.anim_set_index = sprite_asset.anim_set_index;
            state.palette_offset = 0;
            state.enabled = true;

            OpResult::COMPLETE
        },

        Op::ActorCoordinatesSet { actor, tile_x: x, tile_y: y } => {
            let actor_index = actor.deref(this_actor);
            let x = x.get_u8(memory, &actors, this_actor) as f64;
            let y = y.get_u8(memory, &actors, this_actor) as f64;

            actors[actor_index].move_to(Vec2Df64::new(x * 16.0 + 8.0, y * 16.0 + 16.0), true, &scene_map);

            OpResult::COMPLETE
        },

        Op::ActorUpdateFlags { actor, set, remove } => {
            let actor_index = actor.deref(this_actor);
            actors[actor_index].flags.insert(set);
            actors[actor_index].flags.remove(remove);

            OpResult::COMPLETE
        },

        Op::ActorCoordinatesSetPrecise { actor, x, y } => {
            let actor_index = actor.deref(this_actor);
            let pos = Vec2Df64::new(
                x.get_u16(memory, &actors, this_actor) as f64,
                y.get_u16(memory, &actors, this_actor) as f64,
            );

            actors[actor_index].move_to(pos, true, &scene_map);

            OpResult::COMPLETE
        },

        // Actor facing.
        Op::ActorFacingSet { actor, facing } => {
            let actor_index = actor.deref(this_actor);
            let facing = Facing::from_index(facing.get_u8(memory, &actors, this_actor) as usize);
            let state = ctx.sprites_states.get_state_mut(actor_index);

            actors[actor_index].facing = facing;
            state.anim_delay = 0;

            OpResult::YIELD | OpResult::COMPLETE
        },

        Op::ActorSetFacingTowards { actor, to } => {
            let actor_index = actor.deref(this_actor);
            let actor_to_index = to.deref(this_actor);
            let state = ctx.sprites_states.get_state_mut(actor_index);

            let other_actor = &actors[actor_to_index];
            if other_actor.flags.contains(ActorFlags::DEAD) {
                return OpResult::COMPLETE;
            }
            let other_pos = other_actor.pos;
            actors[actor_index].face_towards(other_pos);
            state.anim_delay = 0;

            OpResult::YIELD | OpResult::COMPLETE
        },

        // todo rest of bits
        Op::ActorSetSpritePriority { actor, top, bottom, set_and_lock, .. } => {
            let actor_index = actor.deref(this_actor);

            if set_and_lock {
                actors[actor_index].flags.set(ActorFlags::SPRITE_PRIORITY_LOCKED, true);
                actors[actor_index].sprite_priority_top = top;
                actors[actor_index].sprite_priority_bottom = bottom;
            } else {
                actors[actor_index].update_sprite_priority(&scene_map);
                actors[actor_index].flags.set(ActorFlags::SPRITE_PRIORITY_LOCKED, false);
            }

            OpResult::COMPLETE
        },

        Op::ActorSetSpeed { actor, speed } => {
            let actor_index = actor.deref(this_actor);
            actors[actor_index].move_speed = speed.get_u8(memory, &actors, this_actor) as f64 / 16.0;
            OpResult::COMPLETE
        },

        // Animation ops.
        Op::Animation { actor, animation } => {
            let actor_index = actor.deref(this_actor);
            let anim_index = animation.get_u8(memory, &actors, this_actor) as usize;
            let state = ctx.sprites_states.get_state_mut(actor_index);

            exec_animation(state, anim_index)
        },

        Op::AnimationLoopCount { actor, animation, loops } => {
            let actor_index = actor.deref(this_actor);
            let anim_index = animation.get_u8(memory, &actors, this_actor) as usize;
            let loop_count = loops.get_u8(memory, &actors, this_actor) as u32;
            let state = ctx.sprites_states.get_state_mut(actor_index);

            exec_animation_loop_count(state, &mut actors[actor_index], anim_index, loop_count)
        },

        Op::AnimationReset { actor } => {
            let actor_index = actor.deref(this_actor);
            let state = ctx.sprites_states.get_state_mut(actor_index);

            exec_animation_reset(state)
        },

        Op::AnimationStaticFrame { actor, frame} => {
            let actor_index = actor.deref(this_actor);
            let frame_index = frame.get_u8(memory, &actors, this_actor) as usize;
            let state = ctx.sprites_states.get_state_mut(actor_index);

            exec_animation_static_frame(state, frame_index)
        },

        // Movement ops.
        Op::ActorMoveAtAngle { actor, angle, steps, update_facing, animated } => {
            let actor_index = actor.deref(this_actor);
            let angle = angle.get_u8(memory, &actors, this_actor) as f64 * 1.40625;
            let steps = steps.get_u8(memory, &actors, this_actor) as u32;

            exec_movement_by_vector(ctx, actor_index, actors, angle, steps, update_facing, animated)
        },

        Op::ActorMoveToActor { actor, to_actor, script_cycle_count, update_facing, animated, forever, into_battle_range } => {
            let actor_index = actor.deref(this_actor);
            let target_actor_index = to_actor.deref(this_actor);

            let result = exec_movement_to_actor(ctx, state, actor_index, actors, target_actor_index, script_cycle_count, update_facing, animated, into_battle_range);
            if forever {
                OpResult::YIELD
            } else {
                result
            }
        },

        Op::ActorMoveToTile { actor, x, y, steps, update_facing, animated } => {
            let actor_index = actor.deref(this_actor);
            let dest_tile_x = x.get_u8(memory, &actors, this_actor) as i32;
            let dest_tile_y = y.get_u8(memory, &actors, this_actor) as i32;
            let steps = if let Some(steps) = steps { Some(steps.get_u8(memory, &actors, this_actor) as u32) } else { None };

            exec_movement_to_tile(ctx, state, actor_index, actors, Vec2Di32::new(dest_tile_x, dest_tile_y), steps, update_facing, animated)
        }

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

        Op::Random { dest } => {
            dest.put_u8(memory, ctx.random.get_u8());
            OpResult::COMPLETE
        },

        Op::DialogueSetTable { address } => {
            ctx.fs.read_dialogue_table(address, &mut dialogue);

            OpResult::COMPLETE
        },

        Op::DialogueShow { index, position, .. } => {
            if index < dialogue.len() {
                println!(">>>> @ {:?}: {}", position, dialogue[index]);
            } else {
                println!(">>>> @{:?}: {}", position, index);
            }

            OpResult::COMPLETE
        },

        Op::DialogueSpecial { dialogue_type } => {
            println!(">>>> {:?}", dialogue_type);

            OpResult::COMPLETE
        },

        _ => OpResult::COMPLETE,
    }
}
