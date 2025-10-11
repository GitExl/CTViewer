use crate::actor::{ActorClass, ActorFlags, DebugSprite, DrawMode};
use crate::camera::CameraMoveTo;
use crate::Context;
use crate::facing::Facing;
use crate::gamestate::gamestate_scene::SceneState;
use crate::scene::textbox::TextBoxPosition;
use crate::scene_script::exec::animation::{exec_animation, exec_animation_loop_count, exec_animation_reset, exec_animation_static_frame};
use crate::scene_script::exec::call::{exec_call, exec_call_return, exec_call_wait_completion, exec_call_wait_return};
use crate::scene_script::exec::movement::{exec_movement_to_tile, exec_movement_by_vector, exec_movement_to_actor};
use crate::scene_script::ops::Op;
use crate::scene_script::decoder::ops_char_load::CharacterType;
use crate::scene_script::decoder::ops_jump::CompareOp;
use crate::scene_script::decoder::ops_math::{BitMathOp, ByteMathOp};
use crate::scene_script::exec::tile_copy::exec_tile_copy;
use crate::scene_script::scene_script::{ActorScriptState, OpResult};
use crate::util::rect::Rect;
use crate::sprites::sprite_renderer::SpritePriority;
use crate::util::vec2df64::Vec2Df64;
use crate::util::vec2di32::Vec2Di32;

pub fn op_execute(ctx: &mut Context, scene_state: &mut SceneState, this_actor: usize, state: &mut ActorScriptState) -> OpResult {
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
            let target_actor = &mut scene_state.actors[target_index];
            let target_state = &mut scene_state.script_states[target_index];

            exec_call(target_actor, target_state, function, priority)
        },
        Op::CallWaitCompletion { actor, priority, function } => {
            let target_index = actor.deref(this_actor);
            let target_actor = &mut scene_state.actors[target_index];
            let target_state = &mut scene_state.script_states[target_index];

            exec_call_wait_completion(target_actor, target_state, function, priority)
        },
        Op::CallWaitReturn { actor, priority, function } => {
            let target_index = actor.deref(this_actor);
            let target_actor = &mut scene_state.actors[target_index];
            let target_state = &mut scene_state.script_states[target_index];

            exec_call_wait_return(state, target_actor, target_state, function, priority)
        },

        // Copy.
        Op::Copy8 { source, dest } => {
            dest.put_u8(ctx, scene_state, source.get_u8(ctx, scene_state, this_actor));
            OpResult::COMPLETE
        },
        Op::Copy16 { source, dest } => {
            dest.put_u16(ctx, scene_state, source.get_u16(ctx, scene_state, this_actor));
            OpResult::COMPLETE
        },
        Op::CopyBytes { dest, bytes, length } => {
            dest.put_bytes(ctx, scene_state, bytes, length);
            OpResult::COMPLETE
        },

        // Control flow.
        Op::Jump { offset } => {
            state.current_address = (state.current_address as i64 + offset) as u64;
            OpResult::COMPLETE | OpResult::JUMPED
        },
        Op::JumpConditional8 { lhs, cmp, rhs, offset } => {
            let lhs_value = lhs.get_u8(ctx, scene_state, this_actor);
            let rhs_value = rhs.get_u8(ctx, scene_state, this_actor);
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
        Op::JumpConditionalDrawMode { actor, draw_mode, offset } => {
            let actor_index = actor.deref(this_actor);

            if scene_state.actors[actor_index].draw_mode == draw_mode {
                state.current_address = (state.current_address as i64 + offset) as u64;
                return OpResult::COMPLETE | OpResult::JUMPED;
            }

            OpResult::COMPLETE
        },

        // Math.
        Op::ByteMath8 { dest, lhs, op, rhs } => {
            let lhs_value = lhs.get_u8(ctx, scene_state, this_actor);
            let rhs_value = rhs.get_u8(ctx, scene_state, this_actor);

            let result = match op {
                ByteMathOp::Add => lhs_value.overflowing_add(rhs_value).0,
                ByteMathOp::Subtract => lhs_value.overflowing_sub(rhs_value).0,
            };
            dest.put_u8(ctx, scene_state, result);

            OpResult::COMPLETE
        },
        Op::ByteMath16 { dest, lhs, op, rhs } => {
            let lhs_value = lhs.get_u16(ctx, scene_state, this_actor);
            let rhs_value = rhs.get_u16(ctx, scene_state, this_actor);

            let result = match op {
                ByteMathOp::Add => lhs_value.overflowing_add(rhs_value).0,
                ByteMathOp::Subtract => lhs_value.overflowing_sub(rhs_value).0,
            };
            dest.put_u16(ctx, scene_state, result);

            OpResult::COMPLETE
        },
        Op::BitMath { dest, lhs, op, rhs } => {
            let lhs_value = lhs.get_u8(ctx, scene_state, this_actor);
            let rhs_value = rhs.get_u8(ctx, scene_state, this_actor);

            let result = match op {
                BitMathOp::And => lhs_value & rhs_value,
                BitMathOp::Or => lhs_value | rhs_value,
                BitMathOp::Xor => lhs_value ^ rhs_value,
                BitMathOp::ShiftLeft => lhs_value << rhs_value,
                BitMathOp::ShiftRight => lhs_value >> rhs_value,
            };
            dest.put_u8(ctx, scene_state, result);

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

            let actor = scene_state.actors.get_mut(this_actor).unwrap();

            actor.battle_index = battle_index;
            actor.flags |= ActorFlags::SOLID;
            actor.flags.remove(ActorFlags::PUSHABLE);
            if is_static {
                actor.flags |= ActorFlags::BATTLE_STATIC;
            }

            if char_type == CharacterType::PC {
                actor.player_index = Some(index);
                // todo set actual pc index
                actor.class = ActorClass::PC1;
            }

            // todo: set remaining actor classes
            if char_type == CharacterType::PCAsNPC || char_type == CharacterType::NPC {
                actor.class = ActorClass::NPC;
            }
            if char_type == CharacterType::Enemy {
                actor.class = ActorClass::Enemy;
            }

            actor.facing = Facing::Down;
            actor.sprite_priority_top = SpritePriority::BelowL2AboveL1;
            actor.sprite_priority_bottom = SpritePriority::BelowL2AboveL1;

            let state = &mut ctx.sprites_states.get_state_mut(this_actor);
            let sprite_asset = ctx.sprite_assets.load(&ctx.fs, sprite_index);
            state.sprite_index = sprite_index;
            state.anim_set_index = sprite_asset.anim_set_index;
            state.palette_offset = 0;
            state.anim_index = 0;
            state.anim_frame = 0;

            OpResult::COMPLETE
        },

        Op::ActorUpdateFlags { actor, set, remove } => {
            let actor_index = actor.deref(this_actor);

            scene_state.actors[actor_index].flags.insert(set);
            scene_state.actors[actor_index].flags.remove(remove);

            OpResult::COMPLETE
        },

        Op::ActorSetDrawMode { actor, draw_mode } => {
            let actor_index = actor.deref(this_actor);
            scene_state.actors[actor_index].draw_mode = draw_mode;

            OpResult::COMPLETE | OpResult::YIELD
        },

        Op::ActorRemove { actor } => {
            let actor_index = actor.deref(this_actor);

            scene_state.actors[actor_index].flags |= ActorFlags::DEAD;
            scene_state.actors[actor_index].draw_mode = DrawMode::Hidden;

            OpResult::COMPLETE
        },

        Op::ActorCoordinatesSet { actor, tile_x: x, tile_y: y } => {
            let actor_index = actor.deref(this_actor);
            let x = x.get_u8(ctx, scene_state, this_actor) as f64;
            let y = y.get_u8(ctx, scene_state, this_actor) as f64;

            scene_state.actors[actor_index].move_to(Vec2Df64::new(x * 16.0 + 8.0, y * 16.0 + 16.0), true, &scene_state.scene_map);

            OpResult::COMPLETE
        },

        Op::ActorCoordinatesSetPrecise { actor, x, y } => {
            let actor_index = actor.deref(this_actor);
            let pos = Vec2Df64::new(
                x.get_u16(ctx, scene_state, this_actor) as f64,
                y.get_u16(ctx, scene_state, this_actor) as f64,
            );

            scene_state.actors[actor_index].move_to(pos, true, &scene_state.scene_map);

            OpResult::COMPLETE
        },

        Op::ActorCoordinatesGet { actor, tile_x, tile_y } => {
            let actor_index = actor.deref(this_actor);
            let actor = &scene_state.actors[actor_index];

            let tile_pos_x = (actor.pos.x / 16.0) as u8;
            let tile_pos_y = (actor.pos.y / 16.0) as u8;
            tile_x.put_u8(ctx, scene_state, tile_pos_x);
            tile_y.put_u8(ctx, scene_state, tile_pos_y);

            OpResult::COMPLETE
        },

        // Actor facing.
        Op::ActorFacingSet { actor, facing } => {
            let actor_index = actor.deref(this_actor);
            let facing = Facing::from_index(facing.get_u8(ctx, scene_state, this_actor) as usize);
            let state = ctx.sprites_states.get_state_mut(actor_index);

            scene_state.actors[actor_index].facing = facing;
            state.anim_delay = 0;

            OpResult::YIELD | OpResult::COMPLETE
        },

        Op::ActorSetFacingTowards { actor, to } => {
            let actor_index = actor.deref(this_actor);
            let actor_to_index = to.deref(this_actor);
            let state = ctx.sprites_states.get_state_mut(actor_index);

            let other_actor = &scene_state.actors[actor_to_index];
            if other_actor.flags.contains(ActorFlags::DEAD) {
                return OpResult::COMPLETE;
            }
            let other_pos = other_actor.pos;
            scene_state.actors[actor_index].face_towards(other_pos);
            state.anim_delay = 0;

            OpResult::YIELD | OpResult::COMPLETE
        },

        // todo rest of bits
        Op::ActorSetSpritePriority { actor, top, bottom, set_and_lock, .. } => {
            let actor_index = actor.deref(this_actor);

            if set_and_lock {
                scene_state.actors[actor_index].flags.set(ActorFlags::SPRITE_PRIORITY_LOCKED, true);
                scene_state.actors[actor_index].sprite_priority_top = top;
                scene_state.actors[actor_index].sprite_priority_bottom = bottom;
            } else {
                scene_state.actors[actor_index].update_sprite_priority(&scene_state.scene_map);
                scene_state.actors[actor_index].flags.set(ActorFlags::SPRITE_PRIORITY_LOCKED, false);
            }

            OpResult::COMPLETE
        },

        Op::ActorSetSpeed { actor, speed } => {
            let actor_index = actor.deref(this_actor);
            scene_state.actors[actor_index].move_speed = speed.get_u8(ctx, scene_state, this_actor) as f64 / 17.0;
            OpResult::COMPLETE
        },

        Op::ActorSetResult8 { actor, result } => {
            let actor_index = actor.deref(this_actor);
            scene_state.actors[actor_index].result = result.get_u8(ctx, scene_state, this_actor) as u32;
            OpResult::COMPLETE
        },

        Op::ActorSetResult16 { actor, result } => {
            let actor_index = actor.deref(this_actor);
            scene_state.actors[actor_index].result = result.get_u16(ctx, scene_state, this_actor) as u32;
            OpResult::COMPLETE
        },

        // Animation ops.
        Op::Animation { actor, animation } => {
            let actor_index = actor.deref(this_actor);
            let anim_index = animation.get_u8(ctx, scene_state, this_actor) as usize;
            let state = ctx.sprites_states.get_state_mut(actor_index);

            exec_animation(state, anim_index)
        },

        Op::AnimationLoopCount { actor, animation, loops } => {
            let actor_index = actor.deref(this_actor);
            let anim_index = animation.get_u8(ctx, scene_state, this_actor) as usize;
            let loop_count = loops.get_u8(ctx, scene_state, this_actor) as u32;
            let state = ctx.sprites_states.get_state_mut(actor_index);

            exec_animation_loop_count(state, &mut scene_state.actors[actor_index], anim_index, loop_count)
        },

        Op::AnimationReset { actor } => {
            let actor_index = actor.deref(this_actor);
            let state = ctx.sprites_states.get_state_mut(actor_index);

            exec_animation_reset(state)
        },

        Op::AnimationStaticFrame { actor, frame} => {
            let actor_index = actor.deref(this_actor);
            let frame_index = frame.get_u8(ctx, scene_state, this_actor) as usize;
            let state = ctx.sprites_states.get_state_mut(actor_index);

            exec_animation_static_frame(state, frame_index)
        },

        // Movement ops.
        Op::ActorMoveAtAngle { angle, steps, update_facing, animated } => {
            let angle = angle.get_u8(ctx, scene_state, this_actor) as f64 * 1.40625;
            let steps = steps.get_u8(ctx, scene_state, this_actor) as u32;

            exec_movement_by_vector(ctx, scene_state, this_actor, angle, steps, update_facing, animated)
        },

        Op::ActorMoveToActor { to_actor, script_cycle_count, update_facing, animated, forever, into_battle_range } => {
            let target_actor_index = to_actor.deref(this_actor);

            let result = exec_movement_to_actor(ctx, scene_state, state, this_actor, target_actor_index, script_cycle_count, update_facing, animated, into_battle_range);
            if forever {
                OpResult::YIELD
            } else {
                result
            }
        },

        Op::ActorMoveToTile { x, y, steps, update_facing, animated } => {
            let dest_tile_x = x.get_u8(ctx, scene_state, this_actor) as i32;
            let dest_tile_y = y.get_u8(ctx, scene_state, this_actor) as i32;
            let steps = if let Some(steps) = steps { Some(steps.get_u8(ctx, scene_state, this_actor) as u32) } else { None };

            exec_movement_to_tile(ctx, scene_state, state, this_actor, Vec2Di32::new(dest_tile_x, dest_tile_y), steps, update_facing, animated)
        }

        Op::CopyTiles { left, top, right, bottom, dest_x, dest_y, flags, delayed } => {
            exec_tile_copy(scene_state, left, top, right, bottom, dest_x, dest_y, flags, delayed)
        },

        Op::SetScriptDelay { delay } => {
            state.delay = delay + 1;
            state.delay_counter = delay + 1;
            OpResult::COMPLETE
        },

        Op::SetScriptProcessing { actor, enabled } => {
            let actor_index = actor.deref(this_actor);
            if enabled {
                scene_state.actors[actor_index].flags.set(ActorFlags::SCRIPT_DISABLED, false);
            } else {
                scene_state.actors[actor_index].flags.set(ActorFlags::SCRIPT_DISABLED, true);
                if actor_index == this_actor {
                    return OpResult::COMPLETE | OpResult::YIELD;
                }
            }

            OpResult::COMPLETE
        },

        Op::Wait { ticks } => {
            let actor = scene_state.actors.get_mut(this_actor).unwrap();

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
            let random = ctx.random.get_u8();
            dest.put_u8(ctx, scene_state, random);
            OpResult::COMPLETE
        },

        Op::Battle { flags } => {

            // For now, kill all valid enemies in "battle range".
            let battle_range = Rect::new(
                scene_state.camera.pos.x as i32, scene_state.camera.pos.y as i32,
                (scene_state.camera.pos.x + scene_state.camera.size.x) as i32, (scene_state.camera.pos.y + scene_state.camera.size.y) as i32,
            );

            println!("Battle time! {:?}", flags);

            for actor in scene_state.actors.iter_mut() {
                if actor.class != ActorClass::Enemy {
                    continue;
                }
                if actor.flags.contains(ActorFlags::DEAD) || actor.flags.contains(ActorFlags::SCRIPT_DISABLED) {
                    continue;
                }
                if actor.draw_mode != DrawMode::Draw {
                    continue;
                }
                if !battle_range.contains_vec2(&actor.pos.as_vec2d_i32()) {
                    continue;
                }
                if actor.flags.contains(ActorFlags::BATTLE_STATIC) {
                    continue;
                }

                actor.flags.insert(ActorFlags::DEAD | ActorFlags::SCRIPT_DISABLED);
                actor.draw_mode = DrawMode::Removed;

                println!("Actor {} was killed in a very real battle.", actor.index);
            }

            OpResult::YIELD | OpResult::COMPLETE
        },

        Op::TextSetTable { address } => {
            ctx.fs.read_textbox_string_table(address, &mut scene_state.textbox_strings);

            OpResult::COMPLETE
        },

        Op::TextBoxShow { index, position, .. } => {
            if scene_state.textbox_strings.is_empty() {
                println!("Attempted to show a textbox without a loaded string table.");
                return OpResult::COMPLETE;
            }

            if scene_state.textbox.is_busy() {
                return OpResult::YIELD;
            }

            let actor = &mut scene_state.actors[this_actor];
            if actor.flags.contains(ActorFlags::TEXTBOX_ACTIVE) {
                actor.flags.remove(ActorFlags::TEXTBOX_ACTIVE);
                return OpResult::COMPLETE;
            }
            actor.flags.insert(ActorFlags::TEXTBOX_ACTIVE);

            // Determine position of player character vs camera top or bottom half to position
            // the textbox in auto mode.
            let real_position = if position == TextBoxPosition::Auto {
                if ((actor.pos.y - scene_state.camera.pos.y) as i32) < 130 {
                    TextBoxPosition::Bottom
                } else {
                    TextBoxPosition::Top
                }
            } else {
                position
            };

            if index < scene_state.textbox_strings.len() {
                scene_state.textbox.show(scene_state.textbox_strings[index].clone(), real_position, this_actor);
            } else {
                scene_state.textbox.show(format!("STRING INDEX {}", index), real_position, this_actor);
            }

            OpResult::YIELD
        },

        Op::DialogueSpecial { dialogue_type } => {
            println!("Show special dialogue {:?}", dialogue_type);

            OpResult::COMPLETE
        },

        Op::ScreenFade { target, delay } => {
            if delay == 0 {
                scene_state.screen_fade.set(target);
            } else {
                scene_state.screen_fade.start(target, delay);
            }

            OpResult::COMPLETE
        },
        Op::ScreenWaitForFade => {
          if scene_state.screen_fade.is_active() {
              OpResult::YIELD
          } else {
              OpResult::COMPLETE | OpResult::YIELD
          }
        },

        Op::MoveCameraTo { x, y } => {
            if scene_state.camera.move_to_state == CameraMoveTo::Enabled {
                return OpResult::YIELD;
            }
            if scene_state.camera.move_to_state == CameraMoveTo::Complete {
                scene_state.camera.move_to_state = CameraMoveTo::Disabled;
                return OpResult::COMPLETE;
            }

            scene_state.camera.move_to(Vec2Df64::new(x as f64 * 16.0, y as f64 * 16.0));

            OpResult::YIELD
        },

        Op::ChangeLocation { destination, instant, queue_different_unknown } => {
            println!("Change location to {:?}, instant? {}, queued unknown? {}", destination.info(ctx), instant, queue_different_unknown);

            scene_state.next_destination.set(destination, true);
            if !instant {
                scene_state.screen_fade.start(0.0, 2);
            }

            OpResult::YIELD | OpResult::COMPLETE
        },

        Op::ChangeLocationFromMemory { .. } => {
            // todo what about PC version?

            println!("Change location from memory");

            OpResult::YIELD | OpResult::COMPLETE
        },

        _ => {
            // println!("Unimplemented {:?}", op);
            OpResult::COMPLETE
        },
    }
}
