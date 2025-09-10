use std::f64::consts::PI;
use crate::actor::{Actor, ActorFlags, DebugSprite, Direction, ActorTask};
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
            actors[this_actor].flags |= ActorFlags::RENDERED | ActorFlags::VISIBLE | ActorFlags::SOLID;
            if is_static {
                actors[this_actor].flags |= ActorFlags::BATTLE_STATIC;
            }

            if char_type == CharacterType::PC {
                actors[this_actor].player_index = Some(index);
            }

            let state = &mut ctx.sprites_states.get_state_mut(this_actor);
            state.enabled = true;
            state.sprite_index = real_index;
            ctx.sprites_states.set_animation(&ctx.sprite_assets, this_actor, 0, true, actors[this_actor].direction);

            (false, true)
        },

        Op::ActorCoordinatesSet { actor, x, y } => {
            let actor_index = actor.deref(this_actor);
            let x = x.get_u8(memory) as f64;
            let y = y.get_u8(memory) as f64;

            actors[actor_index].move_to(x * 16.0 + 8.0, y * 16.0 + 16.0, true, &scene_map);

            // Set sprite priority from scene map properties.
            let tile_x = (actors[actor_index].x / 16.0) as u32;
            let tile_y = (actors[actor_index].y / 16.0) as u32;
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
            actors[actor_index].move_to(x, y, true, &scene_map);

            // Set sprite priority from scene map properties.
            let props = scene_map.get_props_at_coordinates(actors[actor_index].x, actors[actor_index].y);
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

            (true, true)
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
            actors[actor_index].move_speed = (speed.get_u8(memory).saturating_sub(1) as f64) / 16.0;
            (false, true)
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
                    (true, false)
                } else {
                    (false, true)
                }

            // Wait for animation to complete.
            } else if wait && loops < 0xFFFFFFFF && state.anim_loop_count <= loops {
                return (true, false);
            }

            // Wait completed.
            actors[actor_index].debug_sprite = DebugSprite::None;
            (false, true)
        },

        Op::ActorMoveAtAngle { actor, angle, steps, update_direction, animated } => {
            let actor_index = actor.deref(this_actor);
            let angle = angle.get_u8(memory) as f64 * 1.40625;
            let steps = steps.get_u8(memory) as u32;

            handle_vector_movement_op(ctx, actor_index, actors, angle, steps, update_direction, animated)
        },

        Op::ActorMoveTo { actor, x, y, steps, update_direction, animated } => {
            let actor_index = actor.deref(this_actor);
            let dest_tile_x = x.get_u8(memory) as i32;
            let dest_tile_y = y.get_u8(memory) as i32;
            let steps = if let Some(steps) = steps { Some(steps.get_u8(memory) as u32) } else { None };

            handle_tile_movement_op(ctx, state, actor_index, actors, dest_tile_x, dest_tile_y, steps, update_direction, animated)
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
            state.delay_counter = delay;
            (false, true)
        },

        Op::Wait { actor, ticks } => {
            let actor_index = actor.deref(this_actor);
            let actor = actors.get_mut(actor_index).unwrap();

            // Start counting.
            if state.pause_counter == 0 {
                state.pause_counter = 1;
                actor.debug_sprite = DebugSprite::Waiting;
                return (true, false);

            // Count one more tick.
            } else if state.pause_counter < ticks {
                state.pause_counter += 1;
                return (true, false);
            }

            // Finished counting.
            state.pause_counter = 0;
            actor.debug_sprite = DebugSprite::None;
            actor.task = ActorTask::None;
            (false, true)
        },

        _ => (false, true),
    }
}

fn handle_tile_movement_op(ctx: &mut Context, state: &mut ActorScriptState, actor_index: usize, actors: &mut Vec<Actor>, tile_x: i32, tile_y: i32, steps: Option<u32>, update_direction: bool, animated: bool) -> OpResult {
    let actor = actors.get_mut(actor_index).unwrap();

    // Only match tile movements.
    if let ActorTask::MoveToTile { steps, .. } = actor.task {
        // Wait for destination to be reached.
        if steps > 0 {
            return (true, false);
        }
    }

    let actor_tile_x = (actor.x / 16.0) as i32;
    let actor_tile_y = (actor.y / 16.0) as i32;

    let mut move_x = 0.0;
    let mut move_y = 0.0;
    let mut move_steps = 0;

    // Destination tile was reached?
    if actor_tile_x == tile_x && actor_tile_y == tile_y {

        // If enabled, slowly move the actor to the bottom center of the tile, x first.
        if actor.flags.contains(ActorFlags::MOVE_ONTO_TILE) {
            let x = actor.x as i32;
            let y = actor.y as i32;
            let dest_x = (tile_x as f64 * 16.0 + 8.0) as i32;
            let dest_y = (tile_y as f64 * 16.0 + 15.0) as i32;

            // Destination reached, snap to whole pixel coordinate.
            if x == dest_x && y == dest_y {
                actor.x = dest_x as f64;
                actor.y = dest_y as f64;

            // Move on x-axis first.
            } else if x != dest_x {
                (move_x, move_y) = (
                    (dest_x - x).signum() as f64 * 1.0,
                    0.0,
                );
                move_steps = 1;

            // Move on y-axis last.
            } else {
                (move_x, move_y) = (
                    0.0,
                    (dest_y - y).signum() as f64 * 1.0,
                );
                move_steps = 1;
            }
        }

    // (Re)calculate the destination.
    } else {

        // Move towards the destination tile.
        let angle = (tile_y as f64 - actor_tile_y as f64).atan2(tile_x as f64 - actor_tile_x as f64);
        (move_x, move_y) = (
            actor.move_speed * angle.cos(),
            actor.move_speed * angle.sin(),
        );

        // Script speed is the number of movement steps, or an immediate value if set.
        move_steps = if let Some(steps) = steps {
            steps
        } else {
            state.delay
        };
    }

    // No more steps to be taken, complete op.
    if move_steps == 0 {
        if animated {
            ctx.sprites_states.set_animation(&ctx.sprite_assets, actor_index, 0, true, actor.direction);
        }
        actor.task = ActorTask::None;
        actor.debug_sprite = DebugSprite::None;
        return (false, true);
    }

    actor.task = ActorTask::MoveToTile {
        tile_x, tile_y,
        move_x, move_y,
        steps: move_steps,
    };
    actor.debug_sprite = DebugSprite::Moving;

    if update_direction {
        actor.face_towards(actor.x + move_x, actor.y + move_y);
    }

    if animated {

        // Player characters have a separate run animation.
        let is_pc = actor.player_index.is_some();
        let anim_index = if is_pc && move_x + move_y >= 2.0 {
            6
        } else {
            1
        };
        ctx.sprites_states.set_animation(&ctx.sprite_assets, actor_index, anim_index, true, actor.direction);
    }

    (true, false)
}

fn handle_vector_movement_op(ctx: &mut Context, actor_index: usize, actors: &mut Vec<Actor>, angle: f64, steps: u32, update_direction: bool, animated: bool) -> OpResult {
    let actor = actors.get_mut(actor_index).unwrap();

    // Only match angle movements.
    if let ActorTask::MoveByAngle { steps, .. } = actor.task {

        // Wait for destination to be reached.
        if steps > 0 {
            return (true, false);
        }

        // No more steps to be taken, complete op.
        if animated {
            ctx.sprites_states.set_animation(&ctx.sprite_assets, actor_index, 0, true, actor.direction);
        }
        actor.task = ActorTask::None;
        actor.debug_sprite = DebugSprite::None;
        return (false, true);
    }

    // Calculate the movement vector.
    let radians = angle * (PI / 180.0);
    let move_x = actor.move_speed * radians.cos();
    let move_y = actor.move_speed * radians.sin();

    actor.task = ActorTask::MoveByAngle {
        angle,
        move_x, move_y,
        steps,
    };
    actor.debug_sprite = DebugSprite::Moving;

    if update_direction {
        actor.face_towards(actor.x + move_x, actor.y + move_y);
    }

    if animated {

        // Player characters have a separate run animation.
        let is_pc = actor.player_index.is_some();
        let anim_index = if is_pc && move_x + move_y >= 2.0 {
            6
        } else {
            1
        };
        ctx.sprites_states.set_animation(&ctx.sprite_assets, actor_index, anim_index, true, actor.direction);
    }

    (true, false)
}
