use crate::actor::{Actor, ActorClass, ActorFlags, DebugSprite, DrawMode};
use crate::camera::{Camera, CameraMoveTo};
use crate::Context;
use crate::facing::Facing;
use crate::map::Map;
use crate::scene::textbox::{TextBox, TextBoxPosition};
use crate::scene::scene_map::SceneMap;
use crate::scene_script::exec::animation::{exec_animation, exec_animation_loop_count, exec_animation_reset, exec_animation_static_frame};
use crate::scene_script::exec::call::{exec_call, exec_call_return, exec_call_wait_completion, exec_call_wait_return};
use crate::scene_script::exec::movement::{exec_movement_to_tile, exec_movement_by_vector, exec_movement_to_actor};
use crate::scene_script::ops::Op;
use crate::scene_script::decoder::ops_char_load::CharacterType;
use crate::scene_script::decoder::ops_jump::CompareOp;
use crate::scene_script::decoder::ops_math::{BitMathOp, ByteMathOp};
use crate::scene_script::exec::tile_copy::exec_tile_copy;
use crate::scene_script::scene_script::{ActorScriptState, OpResult};
use crate::scene_script::scene_script_memory::SceneScriptMemory;
use crate::screen_fade::ScreenFade;
use crate::util::rect::Rect;
use crate::sprites::sprite_renderer::SpritePriority;
use crate::util::vec2df64::Vec2Df64;
use crate::util::vec2di32::Vec2Di32;

pub struct SceneScriptContext<'a> {
    pub states: &'a mut Vec<ActorScriptState>,
    pub actors: &'a mut Vec<Actor>,
    pub map: &'a mut Map,
    pub scene_map: &'a mut SceneMap,
    pub memory: &'a mut SceneScriptMemory,
    pub textbox_strings: &'a mut Vec<String>,
    pub textbox: &'a mut TextBox,
    pub screen_fade: &'a mut ScreenFade,
    pub camera: &'a mut Camera,
}

pub fn op_execute(ctx: &mut Context, script_ctx: &mut SceneScriptContext, this_actor: usize, state: &mut ActorScriptState) -> OpResult {
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
            let target_actor = &mut script_ctx.actors[target_index];
            let target_state = &mut script_ctx.states[target_index];

            exec_call(target_actor, target_state, function, priority)
        },
        Op::CallWaitCompletion { actor, priority, function } => {
            let target_index = actor.deref(this_actor);
            let target_actor = &mut script_ctx.actors[target_index];
            let target_state = &mut script_ctx.states[target_index];

            exec_call_wait_completion(target_actor, target_state, function, priority)
        },
        Op::CallWaitReturn { actor, priority, function } => {
            let target_index = actor.deref(this_actor);
            let target_actor = &mut script_ctx.actors[target_index];
            let target_state = &mut script_ctx.states[target_index];

            exec_call_wait_return(state, target_actor, target_state, function, priority)
        },

        // Copy.
        Op::Copy8 { source, dest } => {
            dest.put_u8(script_ctx, source.get_u8(script_ctx, this_actor));
            OpResult::COMPLETE
        },
        Op::Copy16 { source, dest } => {
            dest.put_u16(script_ctx, source.get_u16(script_ctx, this_actor));
            OpResult::COMPLETE
        },
        Op::CopyBytes { dest, bytes, length } => {
            dest.put_bytes(script_ctx, bytes, length);
            OpResult::COMPLETE
        },

        // Control flow.
        Op::Jump { offset } => {
            state.current_address = (state.current_address as i64 + offset) as u64;
            OpResult::COMPLETE | OpResult::JUMPED
        },
        Op::JumpConditional8 { lhs, cmp, rhs, offset } => {
            let lhs_value = lhs.get_u8(script_ctx, this_actor);
            let rhs_value = rhs.get_u8(script_ctx, this_actor);
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

            if script_ctx.actors[actor_index].draw_mode == draw_mode {
                state.current_address = (state.current_address as i64 + offset) as u64;
                return OpResult::COMPLETE | OpResult::JUMPED;
            }

            OpResult::COMPLETE
        },

        // Math.
        Op::ByteMath8 { dest, lhs, op, rhs } => {
            let lhs_value = lhs.get_u8(script_ctx, this_actor);
            let rhs_value = rhs.get_u8(script_ctx, this_actor);

            let result = match op {
                ByteMathOp::Add => lhs_value.overflowing_add(rhs_value).0,
                ByteMathOp::Subtract => lhs_value.overflowing_sub(rhs_value).0,
            };
            dest.put_u8(script_ctx, result);

            OpResult::COMPLETE
        },
        Op::ByteMath16 { dest, lhs, op, rhs } => {
            let lhs_value = lhs.get_u16(script_ctx, this_actor);
            let rhs_value = rhs.get_u16(script_ctx, this_actor);

            let result = match op {
                ByteMathOp::Add => lhs_value.overflowing_add(rhs_value).0,
                ByteMathOp::Subtract => lhs_value.overflowing_sub(rhs_value).0,
            };
            dest.put_u16(script_ctx, result);

            OpResult::COMPLETE
        },
        Op::BitMath { dest, lhs, op, rhs } => {
            let lhs_value = lhs.get_u8(script_ctx, this_actor);
            let rhs_value = rhs.get_u8(script_ctx, this_actor);

            let result = match op {
                BitMathOp::And => lhs_value & rhs_value,
                BitMathOp::Or => lhs_value | rhs_value,
                BitMathOp::Xor => lhs_value ^ rhs_value,
                BitMathOp::ShiftLeft => lhs_value << rhs_value,
                BitMathOp::ShiftRight => lhs_value >> rhs_value,
            };
            dest.put_u8(script_ctx, result);

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

            let actor = script_ctx.actors.get_mut(this_actor).unwrap();

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

            script_ctx.actors[actor_index].flags.insert(set);
            script_ctx.actors[actor_index].flags.remove(remove);

            OpResult::COMPLETE
        },

        Op::ActorSetDrawMode { actor, draw_mode } => {
            let actor_index = actor.deref(this_actor);
            script_ctx.actors[actor_index].draw_mode = draw_mode;

            OpResult::COMPLETE | OpResult::YIELD
        },

        Op::ActorRemove { actor } => {
            let actor_index = actor.deref(this_actor);

            script_ctx.actors[actor_index].flags |= ActorFlags::DEAD;
            script_ctx.actors[actor_index].draw_mode = DrawMode::Hidden;

            OpResult::COMPLETE
        },

        Op::ActorCoordinatesSet { actor, tile_x: x, tile_y: y } => {
            let actor_index = actor.deref(this_actor);
            let x = x.get_u8(script_ctx, this_actor) as f64;
            let y = y.get_u8(script_ctx, this_actor) as f64;

            script_ctx.actors[actor_index].move_to(Vec2Df64::new(x * 16.0 + 8.0, y * 16.0 + 16.0), true, &script_ctx.scene_map);

            OpResult::COMPLETE
        },

        Op::ActorCoordinatesSetPrecise { actor, x, y } => {
            let actor_index = actor.deref(this_actor);
            let pos = Vec2Df64::new(
                x.get_u16(script_ctx, this_actor) as f64,
                y.get_u16(script_ctx, this_actor) as f64,
            );

            script_ctx.actors[actor_index].move_to(pos, true, &script_ctx.scene_map);

            OpResult::COMPLETE
        },

        Op::ActorCoordinatesGet { actor, tile_x, tile_y } => {
            let actor_index = actor.deref(this_actor);
            let actor = &script_ctx.actors[actor_index];

            let tile_pos_x = (actor.pos.x / 16.0) as u8;
            let tile_pos_y = (actor.pos.y / 16.0) as u8;
            tile_x.put_u8(script_ctx, tile_pos_x);
            tile_y.put_u8(script_ctx, tile_pos_y);

            OpResult::COMPLETE
        },

        // Actor facing.
        Op::ActorFacingSet { actor, facing } => {
            let actor_index = actor.deref(this_actor);
            let facing = Facing::from_index(facing.get_u8(script_ctx, this_actor) as usize);
            let state = ctx.sprites_states.get_state_mut(actor_index);

            script_ctx.actors[actor_index].facing = facing;
            state.anim_delay = 0;

            OpResult::YIELD | OpResult::COMPLETE
        },

        Op::ActorSetFacingTowards { actor, to } => {
            let actor_index = actor.deref(this_actor);
            let actor_to_index = to.deref(this_actor);
            let state = ctx.sprites_states.get_state_mut(actor_index);

            let other_actor = &script_ctx.actors[actor_to_index];
            if other_actor.flags.contains(ActorFlags::DEAD) {
                return OpResult::COMPLETE;
            }
            let other_pos = other_actor.pos;
            script_ctx.actors[actor_index].face_towards(other_pos);
            state.anim_delay = 0;

            OpResult::YIELD | OpResult::COMPLETE
        },

        // todo rest of bits
        Op::ActorSetSpritePriority { actor, top, bottom, set_and_lock, .. } => {
            let actor_index = actor.deref(this_actor);

            if set_and_lock {
                script_ctx.actors[actor_index].flags.set(ActorFlags::SPRITE_PRIORITY_LOCKED, true);
                script_ctx.actors[actor_index].sprite_priority_top = top;
                script_ctx.actors[actor_index].sprite_priority_bottom = bottom;
            } else {
                script_ctx.actors[actor_index].update_sprite_priority(&script_ctx.scene_map);
                script_ctx.actors[actor_index].flags.set(ActorFlags::SPRITE_PRIORITY_LOCKED, false);
            }

            OpResult::COMPLETE
        },

        Op::ActorSetSpeed { actor, speed } => {
            let actor_index = actor.deref(this_actor);
            script_ctx.actors[actor_index].move_speed = speed.get_u8(script_ctx, this_actor) as f64 / 16.0;
            OpResult::COMPLETE
        },

        Op::ActorSetResult8 { actor, result } => {
            let actor_index = actor.deref(this_actor);
            script_ctx.actors[actor_index].result = result.get_u8(script_ctx, this_actor) as u32;
            OpResult::COMPLETE
        },

        Op::ActorSetResult16 { actor, result } => {
            let actor_index = actor.deref(this_actor);
            script_ctx.actors[actor_index].result = result.get_u16(script_ctx, this_actor) as u32;
            OpResult::COMPLETE
        },

        // Animation ops.
        Op::Animation { actor, animation } => {
            let actor_index = actor.deref(this_actor);
            let anim_index = animation.get_u8(script_ctx, this_actor) as usize;
            let state = ctx.sprites_states.get_state_mut(actor_index);

            exec_animation(state, anim_index)
        },

        Op::AnimationLoopCount { actor, animation, loops } => {
            let actor_index = actor.deref(this_actor);
            let anim_index = animation.get_u8(script_ctx, this_actor) as usize;
            let loop_count = loops.get_u8(script_ctx, this_actor) as u32;
            let state = ctx.sprites_states.get_state_mut(actor_index);

            exec_animation_loop_count(state, &mut script_ctx.actors[actor_index], anim_index, loop_count)
        },

        Op::AnimationReset { actor } => {
            let actor_index = actor.deref(this_actor);
            let state = ctx.sprites_states.get_state_mut(actor_index);

            exec_animation_reset(state)
        },

        Op::AnimationStaticFrame { actor, frame} => {
            let actor_index = actor.deref(this_actor);
            let frame_index = frame.get_u8(script_ctx, this_actor) as usize;
            let state = ctx.sprites_states.get_state_mut(actor_index);

            exec_animation_static_frame(state, frame_index)
        },

        // Movement ops.
        Op::ActorMoveAtAngle { angle, steps, update_facing, animated } => {
            let angle = angle.get_u8(script_ctx, this_actor) as f64 * 1.40625;
            let steps = steps.get_u8(script_ctx, this_actor) as u32;

            exec_movement_by_vector(ctx, script_ctx, this_actor, angle, steps, update_facing, animated)
        },

        Op::ActorMoveToActor { to_actor, script_cycle_count, update_facing, animated, forever, into_battle_range } => {
            let target_actor_index = to_actor.deref(this_actor);

            let result = exec_movement_to_actor(ctx, script_ctx, state, this_actor, target_actor_index, script_cycle_count, update_facing, animated, into_battle_range);
            if forever {
                OpResult::YIELD
            } else {
                result
            }
        },

        Op::ActorMoveToTile { x, y, steps, update_facing, animated } => {
            let dest_tile_x = x.get_u8(script_ctx, this_actor) as i32;
            let dest_tile_y = y.get_u8(script_ctx, this_actor) as i32;
            let steps = if let Some(steps) = steps { Some(steps.get_u8(script_ctx, this_actor) as u32) } else { None };

            exec_movement_to_tile(ctx, script_ctx, state, this_actor, Vec2Di32::new(dest_tile_x, dest_tile_y), steps, update_facing, animated)
        }

        Op::CopyTiles { left, top, right, bottom, dest_x, dest_y, flags, delayed } => {
            exec_tile_copy(script_ctx, left, top, right, bottom, dest_x, dest_y, flags, delayed)
        },

        Op::SetScriptDelay { delay } => {
            state.delay = delay + 1;
            state.delay_counter = delay + 1;
            OpResult::COMPLETE
        },

        Op::SetScriptProcessing { actor, enabled } => {
            let actor_index = actor.deref(this_actor);
            if enabled {
                script_ctx.actors[actor_index].flags.set(ActorFlags::SCRIPT_DISABLED, false);
            } else {
                script_ctx.actors[actor_index].flags.set(ActorFlags::SCRIPT_DISABLED, true);
                if actor_index == this_actor {
                    return OpResult::COMPLETE | OpResult::YIELD;
                }
            }

            OpResult::COMPLETE
        },

        Op::Wait { ticks } => {
            let actor = script_ctx.actors.get_mut(this_actor).unwrap();

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
            dest.put_u8(script_ctx, ctx.random.get_u8());
            OpResult::COMPLETE
        },

        Op::Battle { .. } => {

            // For now, kill all valid enemies in "battle range".
            let battle_range = Rect::new(
                script_ctx.camera.pos.x as i32, script_ctx.camera.pos.y as i32,
                (script_ctx.camera.pos.x + script_ctx.camera.size.x) as i32, (script_ctx.camera.pos.y + script_ctx.camera.size.y) as i32,
            );
            for actor in script_ctx.actors.iter_mut() {
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

                actor.flags.insert(ActorFlags::DEAD | ActorFlags::SCRIPT_DISABLED);
                actor.draw_mode = DrawMode::Removed;

                println!("Actor {} was killed in a very real battle!", actor.index);
            }

            OpResult::YIELD | OpResult::COMPLETE
        },

        Op::TextSetTable { address } => {
            ctx.fs.read_textbox_string_table(address, &mut script_ctx.textbox_strings);

            OpResult::COMPLETE
        },

        Op::TextBoxShow { index, position, .. } => {
            if script_ctx.textbox_strings.is_empty() {
                println!("Attempted to show a textbox without a loaded string table.");
                return OpResult::COMPLETE;
            }

            if script_ctx.textbox.is_busy() {
                return OpResult::YIELD;
            }

            let actor = &mut script_ctx.actors[this_actor];
            if actor.flags.contains(ActorFlags::TEXTBOX_ACTIVE) {
                actor.flags.remove(ActorFlags::TEXTBOX_ACTIVE);
                return OpResult::COMPLETE;
            }
            actor.flags.insert(ActorFlags::TEXTBOX_ACTIVE);

            // Determine position of player character vs camera top or bottom half to position
            // the textbox in auto mode.
            let real_position = if position == TextBoxPosition::Auto {
                if ((actor.pos.y - script_ctx.camera.pos.y) as i32) < 130 {
                    TextBoxPosition::Bottom
                } else {
                    TextBoxPosition::Top
                }
            } else {
                position
            };

            if index < script_ctx.textbox_strings.len() {
                script_ctx.textbox.show(script_ctx.textbox_strings[index].clone(), real_position, this_actor);
            } else {
                script_ctx.textbox.show(format!("STRING INDEX {}", index), real_position, this_actor);
            }

            OpResult::YIELD
        },

        Op::DialogueSpecial { dialogue_type } => {
            println!("Show special dialogue {:?}", dialogue_type);

            OpResult::COMPLETE
        },

        Op::ScrollLayers { x, y, flags, duration } => {
            OpResult::COMPLETE
        },

        Op::ScreenFade { target, speed } => {
            if speed == 0.0 {
                script_ctx.screen_fade.set(target);
            } else {
                script_ctx.screen_fade.start(target, speed);
            }

            OpResult::COMPLETE
        },

        Op::MoveCameraTo { x, y } => {
            if script_ctx.camera.move_to_state == CameraMoveTo::Enabled {
                return OpResult::YIELD;
            }
            if script_ctx.camera.move_to_state == CameraMoveTo::Complete {
                script_ctx.camera.move_to_state = CameraMoveTo::Disabled;
                return OpResult::COMPLETE;
            }

            script_ctx.camera.move_to_state = CameraMoveTo::Enabled;
            script_ctx.camera.move_to.x = x as f64 * 16.0;
            script_ctx.camera.move_to.y = y as f64 * 16.0;

            OpResult::YIELD
        },

        Op::PaletteSetImmediate { sub_palette, color_index, data, length } => {
            OpResult::COMPLETE
        },

        _ => {
            // println!("Unimplemented {:?}", op);
            OpResult::COMPLETE
        },
    }
}
