use std::f64::consts::PI;
use crate::actor::{Actor, ActorFlags, ActorTask, DebugSprite};
use crate::Context;
use crate::scene_script::scene_script::{ActorScriptState, OpResult};
use crate::util::vec2df64::Vec2Df64;
use crate::util::vec2di32::Vec2Di32;

pub fn exec_movement_tile(ctx: &mut Context, state: &mut ActorScriptState, actor_index: usize, actors: &mut Vec<Actor>, tile_dest_pos: Vec2Di32, steps: Option<u32>, update_facing: bool, animated: bool) -> OpResult {
    let actor = actors.get_mut(actor_index).unwrap();

    // Only match tile movements.
    if let ActorTask::MoveToTile { steps, .. } = actor.task {

        // Wait for destination to be reached.
        if steps > 0 {
            return OpResult::YIELD;
        }
    }

    let actor_tile_pos = (actor.pos / 16.0).as_vec2d_i32();
    let mut move_by = Vec2Df64::default();
    let mut move_steps = 0;

    // Destination tile was reached?
    if actor_tile_pos.x == tile_dest_pos.x && actor_tile_pos.y == tile_dest_pos.y {

        // If enabled, slowly move the actor to the bottom center of the tile, x first.
        if actor.flags.contains(ActorFlags::MOVE_ONTO_TILE) {
            let actor_pos = actor.pos.as_vec2d_i32();
            let dest_pos = tile_dest_pos * 16 + Vec2Di32::new(8, 15);

            // Move on x-axis first.
            if actor_pos.x != dest_pos.x {
                move_by = Vec2Df64::new(
                    (dest_pos.x - actor_pos.x).signum() as f64 * 1.0,
                    0.0,
                );
                move_steps = 1;

            // Move on y-axis last.
            } else if actor_pos.y != dest_pos.y {
                move_by = Vec2Df64::new(
                    0.0,
                    (dest_pos.y - actor_pos.y).signum() as f64 * 1.0,
                );
                move_steps = 1;

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
        tile_pos: tile_dest_pos,
        move_by,
        steps: move_steps,
    };
    actor.debug_sprite = DebugSprite::Moving;

    if update_facing {
        actor.face_towards(actor.pos + move_by);
    }
    if animated {
        ctx.sprites_states.get_state_mut(actor_index).animate_for_movement(actor.class, move_by);
    }

    OpResult::YIELD
}

pub fn exec_movement_vector(ctx: &mut Context, actor_index: usize, actors: &mut Vec<Actor>, angle: f64, steps: u32, update_facing: bool, animated: bool) -> OpResult {
    let actor = actors.get_mut(actor_index).unwrap();

    // Only match angle movement tasks.
    if let ActorTask::MoveByAngle { steps, .. } = actor.task {

        // Wait for steps to run out.
        if steps > 0 {
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

    actor.task = ActorTask::MoveByAngle {
        angle,
        move_by,
        steps,
    };
    actor.debug_sprite = DebugSprite::Moving;

    if update_facing {
        actor.face_towards(actor.pos + move_by);
    }
    if animated {
        ctx.sprites_states.get_state_mut(actor_index).animate_for_movement(actor.class, move_by);
    }

    OpResult::YIELD
}
