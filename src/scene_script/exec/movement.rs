use std::f64::consts::PI;
use crate::actor::{Actor, ActorFlags, ActorTask, DebugSprite};
use crate::Context;
use crate::scene_script::scene_script::{ActorScriptState, OpResult};

pub fn exec_movement_tile(ctx: &mut Context, state: &mut ActorScriptState, actor_index: usize, actors: &mut Vec<Actor>, tile_x: i32, tile_y: i32, steps: Option<u32>, update_facing: bool, animated: bool) -> OpResult {
    let actor = actors.get_mut(actor_index).unwrap();

    // Only match tile movements.
    if let ActorTask::MoveToTile { steps, .. } = actor.task {
        // Wait for destination to be reached.
        if steps > 0 {
            return OpResult::YIELD;
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

            // Move on x-axis first.
            if x != dest_x {
                (move_x, move_y) = (
                    (dest_x - x).signum() as f64 * 1.0,
                    0.0,
                );
                move_steps = 1;

            // Move on y-axis last.
            } else if y != dest_y {
                (move_x, move_y) = (
                    0.0,
                    (dest_y - y).signum() as f64 * 1.0,
                );
                move_steps = 1;

            // Destination reached, snap to whole pixel coordinate.
            } else {
                actor.x = dest_x as f64;
                actor.y = dest_y as f64;
            }
        }

    // (Re)calculate the destination.
    } else {

        // Move towards the destination tile.
        let angle = (tile_y as f64 - actor_tile_y as f64).atan2(tile_x as f64 - actor_tile_x as f64);
        (move_x, move_y) = (
            (actor.move_speed / 2.0) * angle.cos(),
            (actor.move_speed / 2.0) * angle.sin(),
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
            ctx.sprites_states.get_state_mut(actor_index).reset_animation();
        }
        actor.task = ActorTask::None;
        actor.debug_sprite = DebugSprite::None;
        return OpResult::COMPLETE;
    }

    actor.task = ActorTask::MoveToTile {
        tile_x, tile_y,
        move_x, move_y,
        steps: move_steps,
    };
    actor.debug_sprite = DebugSprite::Moving;

    if update_facing {
        actor.face_towards(actor.x + move_x, actor.y + move_y);
    }
    if animated {
        ctx.sprites_states.get_state_mut(actor_index).animate_for_movement(actor.class, move_x, move_y);
    }

    OpResult::YIELD
}

pub fn exec_movement_vector(ctx: &mut Context, actor_index: usize, actors: &mut Vec<Actor>, angle: f64, steps: u32, update_facing: bool, animated: bool) -> OpResult {
    let actor = actors.get_mut(actor_index).unwrap();

    // Only match angle movements.
    if let ActorTask::MoveByAngle { steps, .. } = actor.task {

        // Wait for destination to be reached.
        if steps > 0 {
            return OpResult::YIELD;
        }

        // No more steps to be taken, complete op.
        if animated {
            ctx.sprites_states.get_state_mut(actor_index).reset_animation();
        }
        actor.task = ActorTask::None;
        actor.debug_sprite = DebugSprite::None;
        return OpResult::COMPLETE;
    }

    // Calculate the movement vector.
    let angle_rads = angle * (PI / 180.0);
    let move_x = (actor.move_speed / 2.0) * angle_rads.cos();
    let move_y = (actor.move_speed / 2.0) * angle_rads.sin();

    actor.task = ActorTask::MoveByAngle {
        angle,
        move_x, move_y,
        steps,
    };
    actor.debug_sprite = DebugSprite::Moving;

    if update_facing {
        actor.face_towards(actor.x + move_x, actor.y + move_y);
    }
    if animated {
        ctx.sprites_states.get_state_mut(actor_index).animate_for_movement(actor.class, move_x, move_y);
    }

    OpResult::YIELD
}
