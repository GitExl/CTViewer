use crate::{Context, GameMode};
use crate::gamestate::gamestate_world::WorldState;
use crate::world_script::tasks::common::{task_fade_in, task_fade_out, task_layer_animation, task_palette_load, task_palette_load_mode};
use crate::world_script::tasks::run_script::task_run_script;
use crate::world_script::world_actor::WorldActor;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum WorldActorTask {
    None,
    Unknown {
        address: u32
    },

    RunScript,

    FadeIn,
    FadeOut,

    PaletteLoad,
    PaletteLoadModes,

    ScrollLayers {
        world: usize,
    },
}

impl WorldActorTask {
    pub fn from_address(address: u32, mode: GameMode) -> WorldActorTask {
        match mode {
            GameMode::Snes =>
                match address {
                    0x0F63 => WorldActorTask::RunScript,

                    0x2105 => WorldActorTask::FadeIn,
                    0x20A2 => WorldActorTask::FadeOut,

                    0x1DD4 => WorldActorTask::PaletteLoad,
                    0x1E38 => WorldActorTask::PaletteLoadModes,

                    0x75C3 => WorldActorTask::ScrollLayers { world: 0 },
                    0x75FD => WorldActorTask::ScrollLayers { world: 1 },
                    0x7702 => WorldActorTask::ScrollLayers { world: 2 },
                    0x77F2 => WorldActorTask::ScrollLayers { world: 4 },
                    0x7849 => WorldActorTask::ScrollLayers { world: 5 },

                    _ => WorldActorTask::Unknown { address },
                }
            GameMode::Pc =>
                match address {
                    0x0F63 => WorldActorTask::RunScript,

                    0x2105 => WorldActorTask::FadeIn,
                    0x20A2 => WorldActorTask::FadeOut,

                    0x1DD4 => WorldActorTask::PaletteLoad,
                    0x1E38 => WorldActorTask::PaletteLoadModes,

                    0x7517 => WorldActorTask::ScrollLayers { world: 0 },
                    0x7551 => WorldActorTask::ScrollLayers { world: 1 },
                    0x7656 => WorldActorTask::ScrollLayers { world: 2 },
                    0x7746 => WorldActorTask::ScrollLayers { world: 4 },
                    0x779D => WorldActorTask::ScrollLayers { world: 5 },

                    _ => WorldActorTask::Unknown { address },
                }
        }
    }
}

pub fn task_dispatch(ctx: &mut Context, world_state: &mut WorldState, actor: &mut WorldActor) {
    match actor.task {
        WorldActorTask::RunScript => task_run_script(ctx, world_state, actor),
        WorldActorTask::PaletteLoad => task_palette_load(world_state, actor),
        WorldActorTask::PaletteLoadModes => task_palette_load_mode(actor),
        WorldActorTask::ScrollLayers { world } => task_layer_animation(world_state, world),
        WorldActorTask::FadeIn => task_fade_in(ctx, actor),
        WorldActorTask::FadeOut => task_fade_out(ctx, actor),
        _ => {},
    }
}
