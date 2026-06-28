use crate::character::CharacterId;
use crate::Context;
use crate::facing::Facing;
use crate::gamestate::gamestate_world::WorldState;
use crate::world_script::world_actor::WorldActor;

enum LeaderState {
    Init,
    Idle,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
}

impl LeaderState {
    pub fn from_value(value: u8) -> LeaderState {
        match value {
            0 => LeaderState::Init,
            1 => LeaderState::Idle,
            2 => LeaderState::MoveUp,
            3 => LeaderState::MoveDown,
            4 => LeaderState::MoveLeft,
            5 => LeaderState::MoveRight,
            _ => LeaderState::Init,
        }
    }

    pub fn to_value(&self) -> u8 {
        match self {
            LeaderState::Init => 0,
            LeaderState::Idle => 1,
            LeaderState::MoveUp => 2,
            LeaderState::MoveDown => 3,
            LeaderState::MoveLeft => 4,
            LeaderState::MoveRight => 5,
        }
    }
}

pub fn task_party_leader(ctx: &mut Context, state: &mut WorldState, actor: &mut WorldActor) {
    let current_anim_index = actor.memory.get_u8(0x3E);
    let mut next_anim_index = current_anim_index;

    let mut leader_state = LeaderState::from_value(actor.memory.get_u8(0x02));
    match leader_state {
        LeaderState::Init => {
            actor.x = state.enter_pos.x;
            actor.y = state.enter_pos.y;
            actor.facing = Facing::Down;
            if state.world_index == 5 {
                actor.palette_priority = 0xB8;
            } else {
                actor.palette_priority = 0xA8;
            }
            next_anim_index = 0x0C;
            leader_state = LeaderState::Idle;
        },
        LeaderState::Idle => {
            // take input
            //if no_input {
            let mut idle_counter = actor.memory.get_u8(0x3F);
            if idle_counter == 0xFE {
                next_anim_index = 0xA2;
            } else {
                idle_counter += 1;
                actor.memory.put_u8(0x3F, idle_counter);
            }
            //}
        }
        _ => {},
    }

    // let anim_index = match actor.facing {
    //     Facing::Down => 0,
    //     Facing::Up => 3,
    //     Facing::Right => 6,
    //     Facing::Left => 9,
    // };
    // actor.animation_address = state.animations.get_animation_address(0x0C + anim_index);

    if current_anim_index != next_anim_index {
        actor.animation_address = state.animations.get_animation_address(next_anim_index as usize);
    }
    state.animations.run(ctx, actor);

    actor.memory.put_u8(0x02, leader_state.to_value());
    actor.memory.put_u8(0x3E, next_anim_index);

    // if not initialized, initialize position and facing from destination
    // if movement is disabled, run script ops
    // if movement is enabled, take input, move object, set correct animation
    // set anim from facing if needed, animate when needed with world_state.animations.run(ctx, actor),

    // movement enabled flag is global to world, $000280, movement is disabled by a trigger or scripted exit
    // x and y is kept in actor memory $12 and $16

    // $02B1 tracks time without input, increments whenever input is checked every frame, FE will start idle anim $A2
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
