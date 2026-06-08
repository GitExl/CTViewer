use crate::{Context, GameMode};
use crate::gamestate::gamestate_world::WorldState;
use crate::world_script::functions::common::{func_actor_is_offscreen, func_seagull_random_pos, func_seagull_random_vector};
use crate::world_script::world_actor::WorldActor;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum WorldActorFunction {
    None,
    Unknown {
        address: u32
    },

    SeagullRandomPosition,
    SeagullRandomVector,
    IsOffscreen,
}

impl WorldActorFunction {
    pub fn from_address(address: u32, mode: GameMode) -> WorldActorFunction {
        match mode {
            GameMode::Snes =>
                match address {
                    0x7575 => WorldActorFunction::SeagullRandomPosition,
                    0x7598 => WorldActorFunction::SeagullRandomVector,
                    0x78A1 => WorldActorFunction::IsOffscreen,

                    _ => WorldActorFunction::Unknown { address },
                }
            GameMode::Pc =>
                match address {
                    0x74C9 => WorldActorFunction::SeagullRandomPosition,
                    0x74EC => WorldActorFunction::SeagullRandomVector,
                    0x77F5 => WorldActorFunction::IsOffscreen,

                    _ => WorldActorFunction::Unknown { address },
                }
        }
    }
}

pub fn function_dispatch(ctx: &mut Context, world_state: &mut WorldState, actor: &mut WorldActor, func: WorldActorFunction) {
    match func {
        WorldActorFunction::SeagullRandomPosition => func_seagull_random_pos(ctx, actor, world_state),
        WorldActorFunction::SeagullRandomVector => func_seagull_random_vector(ctx, actor),
        WorldActorFunction::IsOffscreen => func_actor_is_offscreen(ctx, actor, world_state),
        _ => {},
    }
}
