use std::f64::consts::PI;
use crate::actor::{Actor, ActorFlags, ActorTask, DebugSprite};
use crate::Context;
use crate::scene_script::scene_script::{ActorScriptState, OpResult};
use crate::util::vec2df64::Vec2Df64;
use crate::util::vec2di32::Vec2Di32;

pub fn exec_movement_to_tile(ctx: &mut Context, state: &mut ActorScriptState, actor_index: usize, actors: &mut Vec<Actor>, tile_dest_pos: Vec2Di32, cycle_count: Option<u32>, update_facing: bool, animated: bool) -> OpResult {
    let actor = actors.get_mut(actor_index).unwrap();
    let sprite_state = ctx.sprites_states.get_state_mut(actor_index);

    // Only match tile movements.
    if let ActorTask::MoveToTile { cycles, .. } = actor.task {

        // Wait for destination to be reached.
        if cycles > 0 {
            return OpResult::YIELD;
        }
    }

    let actor_tile_pos = (actor.pos / 16.0).as_vec2d_i32();
    let mut move_by = Vec2Df64::default();
    let mut move_cycle_count = 0;

    // Destination tile was reached?
    if actor_tile_pos == tile_dest_pos {

        // If enabled, slowly move the actor to the bottom center of the tile, x first.
        if actor.flags.contains(ActorFlags::MOVE_ONTO_TILE) {
            let actor_pos = actor.pos.as_vec2d_i32();
            let dest_pos = tile_dest_pos * 16 + Vec2Di32::new(7, 15);

            // Move on x-axis first.
            if actor_pos.x != dest_pos.x {
                move_by = Vec2Df64::new(
                    (dest_pos.x - actor_pos.x).signum() as f64 * 1.0,
                    0.0,
                );
                move_cycle_count = 1;

            // Move on y-axis last.
            } else if actor_pos.y != dest_pos.y {
                move_by = Vec2Df64::new(
                    0.0,
                    (dest_pos.y - actor_pos.y).signum() as f64 * 1.0,
                );
                move_cycle_count = 1;

            // Destination reached, snap to whole pixel coordinate.
            } else {
                actor.pos = dest_pos.as_vec2d_f64();
            }
        }

    // (Re)calculate the destination.
    } else {

        // Move towards the destination tile.
        let angle_rads = Vec2Di32::angle_rad_between(actor_tile_pos, tile_dest_pos);
        move_by = Vec2Df64::new(
            (actor.move_speed / 2.0) * angle_rads.cos(),
            (actor.move_speed / 2.0) * angle_rads.sin(),
        );

        // Script speed is the number of movement steps, or an immediate value if set.
        move_cycle_count = if let Some(cycle_count) = cycle_count {
            cycle_count
        } else {
            state.delay
        };
    }

    // No more steps to be taken, complete op.
    if move_cycle_count == 0 {
        if animated {
            ctx.sprites_states.get_state_mut(actor_index).reset_animation();
        }
        actor.task = ActorTask::None;
        actor.debug_sprite = DebugSprite::None;
        return OpResult::COMPLETE;
    }

    // Set new movement task.
    actor.task = ActorTask::MoveToTile {
        tile_pos: tile_dest_pos,
        move_by,
        cycles: move_cycle_count,
    };
    actor.debug_sprite = DebugSprite::Moving;
    sprite_state.anim_loops_remaining = move_cycle_count;

    if update_facing {
        actor.face_towards(actor.pos + move_by);
    }
    if animated {
        ctx.sprites_states.get_state_mut(actor_index).animate_for_movement(actor.class, move_by);
    }

    OpResult::YIELD
}

pub fn exec_movement_by_vector(ctx: &mut Context, actor_index: usize, actors: &mut Vec<Actor>, angle: f64, cycle_count: u32, update_facing: bool, animated: bool) -> OpResult {
    let actor = actors.get_mut(actor_index).unwrap();
    let sprite_state = ctx.sprites_states.get_state_mut(actor_index);

    // Only match angle movement tasks.
    if let ActorTask::MoveByAngle { cycles, .. } = actor.task {

        // Wait for steps to run out.
        if cycles > 0 {
            return OpResult::YIELD;
        }

        // End op.
        if animated {
            ctx.sprites_states.get_state_mut(actor_index).reset_animation();
        }
        actor.task = ActorTask::None;
        actor.debug_sprite = DebugSprite::None;
        return OpResult::COMPLETE;
    }

    // Calculate the movement vector.
    let angle_rads = angle * (PI / 180.0);
    let move_by = Vec2Df64::new(
        (actor.move_speed / 2.0) * angle_rads.cos(),
        (actor.move_speed / 2.0) * angle_rads.sin(),
    );

    // Set new movement task.
    actor.task = ActorTask::MoveByAngle {
        angle,
        move_by,
        cycles: cycle_count,
    };
    actor.debug_sprite = DebugSprite::Moving;
    sprite_state.anim_loops_remaining = cycle_count;

    if update_facing {
        actor.face_towards(actor.pos + move_by);
    }
    if animated {
        ctx.sprites_states.get_state_mut(actor_index).animate_for_movement(actor.class, move_by);
    }

    OpResult::YIELD
}

pub fn exec_movement_to_actor(ctx: &mut Context, state: &mut ActorScriptState, actor_index: usize, actors: &mut Vec<Actor>, target_actor_index: usize, cycle_count: Option<u32>, update_facing: bool, animated: bool, into_battle_range: bool) -> OpResult {

    // Ignore dead target actor.
    if actors[target_actor_index].flags.contains(ActorFlags::DEAD) {
        ctx.sprites_states.get_state_mut(actor_index).reset_animation();
        return OpResult::COMPLETE;
    }

    let target_pos = actors[target_actor_index].pos;
    let actor = actors.get_mut(actor_index).unwrap();
    let sprite_state = ctx.sprites_states.get_state_mut(actor_index);

    // Only match actor movements.
    if let ActorTask::MoveToActor { cycles, .. } = actor.task {

        // Wait for destination to be reached.
        if cycles > 0 {
            return OpResult::YIELD;
        }
    }

    let tile_pos = (actor.pos / 16.0).as_vec2d_i32();
    let target_tile_pos = (target_pos / 16.0).as_vec2d_i32();
    let mut move_by = Vec2Df64::default();
    let mut move_cycles = 0;

    // Move into battle range?
    let should_move = if into_battle_range {
        let diff = (target_tile_pos - tile_pos).abs();
        diff.x == 1 || diff.y == 1 || diff.x >= 7 || diff.y >= 6
    } else {
        true
    };

    // Destination tile was reached?
    if tile_pos == target_tile_pos {

        // If enabled, slowly move the actor to the target object, x first.
        if actor.flags.contains(ActorFlags::MOVE_ONTO_OBJECT) {
            let actor_pos = actor.pos.as_vec2d_i32();
            let dest_pos = target_pos.as_vec2d_i32();

            // Move on x-axis first.
            if actor_pos.x != dest_pos.x {
                move_by = Vec2Df64::new(
                    (dest_pos.x - actor_pos.x).signum() as f64 * 1.0,
                    0.0,
                );
                move_cycles = 1;
                sprite_state.anim_loops_remaining = 1;

            // Move on y-axis last.
            } else if actor_pos.y != dest_pos.y {
                move_by = Vec2Df64::new(
                    0.0,
                    (dest_pos.y - actor_pos.y).signum() as f64 * 1.0,
                );
                move_cycles = 1;
                sprite_state.anim_loops_remaining = 1;

            // Destination reached, snap to target coordinate.
            } else {
                actor.pos = target_pos;
            }
        }

    // (Re)calculate the destination.
    } else if should_move {

        // Move towards the destination tile.
        let angle_rads = Vec2Di32::angle_rad_between(tile_pos, target_tile_pos);
        move_by = Vec2Df64::new(
            (actor.move_speed / 2.0) * angle_rads.cos(),
            (actor.move_speed / 2.0) * angle_rads.sin(),
        );

        // Script speed is the number of movement steps, or an immediate value if set.
        move_cycles = if let Some(cycle_count) = cycle_count {
            cycle_count
        } else {
            state.delay
        };
    }

    // No more steps to be taken, complete op.
    if move_cycles == 0 {
        if animated {
            ctx.sprites_states.get_state_mut(actor_index).reset_animation();
        }
        actor.task = ActorTask::None;
        actor.debug_sprite = DebugSprite::None;
        return OpResult::COMPLETE;
    }

    // Set new movement task.
    actor.task = ActorTask::MoveToActor {
        actor_index: target_actor_index,
        move_by,
        cycles: move_cycles,
    };
    sprite_state.anim_loops_remaining = move_cycles;
    actor.debug_sprite = DebugSprite::Moving;

    if update_facing {
        actor.face_towards(actor.pos + move_by);
    }
    if animated {
        ctx.sprites_states.get_state_mut(actor_index).animate_for_movement(actor.class, move_by);
    }

    OpResult::YIELD
}
