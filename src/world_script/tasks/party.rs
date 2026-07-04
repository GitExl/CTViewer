use crate::character::CharacterId;
use crate::Context;
use crate::facing::Facing;
use crate::gamestate::gamestate_world::WorldState;
use crate::input::InputAction;
use crate::util::vec2df64::Vec2Df64;
use crate::world_script::world_actor::WorldActor;

pub fn task_party_leader(ctx: &mut Context, state: &mut WorldState, actor: &mut WorldActor) {
    let mut move_x = actor.memory.get_i8(0x3B);
    let mut move_y = actor.memory.get_i8(0x3C);
    let mut move_dist = actor.memory.get_u8(0x3D);
    let current_anim_index = actor.memory.get_u8(0x3E);
    let mut idle_counter = actor.memory.get_u8(0x3F);
    let mut next_anim_index = current_anim_index;

    // Initialize.
    if actor.memory.get_u8(0x02) == 0 {
        actor.pos.x = state.enter_pos.x;
        actor.pos.y = state.enter_pos.y;
        actor.facing = Facing::Down;
        if state.world_index == 5 {
            actor.palette_priority = 0xB8;
        } else {
            actor.palette_priority = 0xA8;
        }
        next_anim_index = 0x0C;
        actor.memory.put_u8(0x02, 1);
    }

    // Move steps.
    let did_move = move_dist > 0;
    if move_dist > 0 {
        move_dist -= 1;
        if move_x != 0 {
            actor.pos.x += move_x as f64;
        } else if move_y != 0 {
            actor.pos.y += move_y as f64;
        }
    }

    // Take input if we are no longer moving.
    if move_dist == 0 {
        let tile_x = (actor.pos.x / 8.0).floor() as i32 - 1;
        let tile_y = (actor.pos.y / 8.0).floor() as i32 - 1;
        let mut have_input = false;
        let step_index = actor.memory.get_u8(0x26) & 0x1;

        if ctx.input.is_down(InputAction::MoveUp) {
            have_input = true;
            actor.facing = Facing::Up;
            if state.world_map.is_walkable(tile_x, tile_y - 1, 2, 1) {
                move_x = 0;
                move_y = -1;
                move_dist = 8;
                if !did_move {
                    next_anim_index = 0x0D + 3 + step_index;
                } else {
                    next_anim_index = 0x0D + 3;
                }
            } else {
                move_x = 0;
                move_y = 0;
                move_dist = 0;
                next_anim_index = 0x0C + 3;
            }
        } else if ctx.input.is_down(InputAction::MoveDown) {
            have_input = true;
            actor.facing = Facing::Down;
            if state.world_map.is_walkable(tile_x, tile_y + 1, 2, 1) {
                move_x = 0;
                move_y = 1;
                move_dist = 8;
                if !did_move {
                    next_anim_index = 0x0D + 0 + step_index;
                } else {
                    next_anim_index = 0x0D + 0;
                }
            } else {
                move_x = 0;
                move_y = 0;
                move_dist = 0;
                next_anim_index = 0x0C + 0;
            }
        } else if ctx.input.is_down(InputAction::MoveLeft) {
            have_input = true;
            actor.facing = Facing::Left;
            if state.world_map.is_walkable(tile_x - 1, tile_y, 2, 1) {
                move_x = -1;
                move_y = 0;
                move_dist = 8;
                if !did_move {
                    next_anim_index = 0x0D + 6 + step_index;
                } else {
                    next_anim_index = 0x0D + 6;
                }
            } else {
                move_x = 0;
                move_dist = 0;
                move_y = 0;
                next_anim_index = 0x0C + 6;
            }
        } else if ctx.input.is_down(InputAction::MoveRight) {
            have_input = true;
            actor.facing = Facing::Right;
            if state.world_map.is_walkable(tile_x + 1, tile_y, 2, 1) {
                move_x = 1;
                move_y = 0;
                move_dist = 8;
                if !did_move {
                    next_anim_index = 0x0D + 9 + step_index;
                } else {
                    next_anim_index = 0x0D + 9;
                }
            } else {
                move_x = 0;
                move_y = 0;
                move_dist = 0;
                next_anim_index = 0x0C + 9;
            }
        }

        // Idle animation.
        let anim_facing_index = match actor.facing {
            Facing::Down => 0,
            Facing::Up => 3,
            Facing::Left => 6,
            Facing::Right => 9,
        };
        if !have_input {
            if idle_counter == 0xFE {
                next_anim_index = 0xA2;
                actor.facing = Facing::Down;
            } else {
                idle_counter += 1;
                next_anim_index = 0x0C + anim_facing_index;
            }
        } else {
            idle_counter = 0;

            // Reset yearbox timer.
            ctx.memory.put_u8(0x7E1BF7, 0);
        }
    }

    if current_anim_index != next_anim_index {
        actor.animation_address = state.animations.get_animation_address(next_anim_index as usize);
        actor.animation_counter = 0;
    }

    actor.memory.put_i8(0x3B, move_x);
    actor.memory.put_i8(0x3C, move_y);
    actor.memory.put_u8(0x3D, move_dist);
    actor.memory.put_u8(0x3E, next_anim_index);
    actor.memory.put_u8(0x3F, idle_counter);

    state.animations.run(ctx, actor);
    state.camera.center_to(Vec2Df64::new(actor.pos.x, actor.pos.y), false, true);

    // TODO
    //  if movement is disabled, run script ops
    //  if movement is enabled, take input, move object, set correct animation (already done)
    //  movement enabled flag is global to world, $000280, movement is disabled by a trigger or scripted exit
}

pub fn task_party_followers(_ctx: &mut Context, _state: &mut WorldState, actor: &mut WorldActor) {
    let _party_index = actor.memory.get_u8(0x24) as CharacterId;

    // if movement mirroring is disabled, or movement is disabled do nothing
    // follow pc1
}

pub fn task_party_process_movement_1(_ctx: &mut Context, _actor: &mut WorldActor) {
}

pub fn task_party_process_movement_2(_ctx: &mut Context, _actor: &mut WorldActor) {
}
