use crate::{Context, GameMode};
use crate::gamestate::gamestate_world::WorldState;
use crate::world_script::tasks::common::{task_fade_in, task_fade_out, task_grp, task_layer_animation, task_palette_load, task_palette_load_mode, task_unknown_string, task_update_music};
use crate::world_script::tasks::epoch::{task_epoch_unk1, task_epoch_unk2};
use crate::world_script::tasks::party::{task_party_followers, task_party_leader, task_party_process_movement_1, task_party_process_movement_2};
use crate::world_script::tasks::run_script::task_run_script;
use crate::world_script::world_actor::WorldActor;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum WorldActorTask {
    None,
    Unknown {
        address: u32
    },
    UnknownGrp,

    RunScript,

    FadeIn,
    FadeOut,

    PaletteLoad,
    PaletteLoadModes,

    ScrollWorldLayers {
        world: usize,
    },

    PartyLeader,
    PartyFollowers,

    EpochUnk1,
    EpochUnk2,

    ProcessMovementUnk1, // 309E
    ProcessMovementUnk2, // 3264

    UpdateMusic, // 2F94

    UnkStringRelated, // 5628
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

                    0x75C3 => WorldActorTask::ScrollWorldLayers { world: 0 },
                    0x75FD => WorldActorTask::ScrollWorldLayers { world: 1 },
                    0x7702 => WorldActorTask::ScrollWorldLayers { world: 2 },
                    0x77F2 => WorldActorTask::ScrollWorldLayers { world: 4 },
                    0x7849 => WorldActorTask::ScrollWorldLayers { world: 5 },

                    0x1CF5 => WorldActorTask::UnknownGrp,

                    0x3404 => WorldActorTask::PartyLeader,
                    0x3AE2 => WorldActorTask::PartyFollowers,

                    0x42DD => WorldActorTask::EpochUnk1,
                    0x495E => WorldActorTask::EpochUnk2,

                    0x309E => WorldActorTask::ProcessMovementUnk1,
                    0x3264 => WorldActorTask::ProcessMovementUnk2,

                    0x2F94 => WorldActorTask::UpdateMusic,

                    0x5628 => WorldActorTask::UnkStringRelated,

                    _ => WorldActorTask::Unknown { address },
                }
            GameMode::Pc =>
                match address {
                    0x0F63 => WorldActorTask::RunScript,

                    0x2105 => WorldActorTask::FadeIn,
                    0x20A2 => WorldActorTask::FadeOut,

                    0x1DD4 => WorldActorTask::PaletteLoad,
                    0x1E38 => WorldActorTask::PaletteLoadModes,

                    0x7517 => WorldActorTask::ScrollWorldLayers { world: 0 },
                    0x7551 => WorldActorTask::ScrollWorldLayers { world: 1 },
                    0x7656 => WorldActorTask::ScrollWorldLayers { world: 2 },
                    0x7746 => WorldActorTask::ScrollWorldLayers { world: 4 },
                    0x779D => WorldActorTask::ScrollWorldLayers { world: 5 },

                    0x1DA1 => WorldActorTask::UnknownGrp,

                    0x3404 => WorldActorTask::PartyLeader,  // TODO verify
                    0x3AE2 => WorldActorTask::PartyFollowers,  // TODO verify

                    0x42DD => WorldActorTask::EpochUnk1, // TODO verify
                    0x495E => WorldActorTask::EpochUnk2, // TODO verify

                    0x309E => WorldActorTask::ProcessMovementUnk1, // TODO verify
                    0x3264 => WorldActorTask::ProcessMovementUnk2, // TODO verify

                    0x2F94 => WorldActorTask::UpdateMusic, // TODO verify

                    0x5628 => WorldActorTask::UnkStringRelated, // TODO verify

                    _ => WorldActorTask::Unknown { address },
                }
        }
    }

    pub fn as_string(&self) -> String {
        match self {
            WorldActorTask::None => "NULL".to_string(),
            WorldActorTask::Unknown { address } => format!("Unknown{:04X}", address),
            WorldActorTask::UnknownGrp => "UnknownGrp".to_string(),

            WorldActorTask::RunScript => "RunScript".to_string(),

            WorldActorTask::FadeIn => "FadeIn".to_string(),
            WorldActorTask::FadeOut => "FadeOut".to_string(),

            WorldActorTask::PaletteLoad => "PaletteLoad".to_string(),
            WorldActorTask::PaletteLoadModes => "PaletteLoadModes".to_string(),

            WorldActorTask::ScrollWorldLayers { world } => format!("ScrollWorldLayers({})", world),

            WorldActorTask::PartyLeader => "PartyLeader".to_string(),
            WorldActorTask::PartyFollowers => "PartyFollowers".to_string(),

            WorldActorTask::EpochUnk1 => "EpochUnknown1".to_string(),
            WorldActorTask::EpochUnk2 => "EpochUnknown2".to_string(),

            WorldActorTask::ProcessMovementUnk1 => "ProcessMovementUnknown1".to_string(),
            WorldActorTask::ProcessMovementUnk2 => "ProcessMovementUnknown2".to_string(),

            WorldActorTask::UpdateMusic => "UpdateMusic".to_string(),

            WorldActorTask::UnkStringRelated => "UnknownStringRelated".to_string(),
        }
    }
}

pub fn task_dispatch(ctx: &mut Context, world_state: &mut WorldState, actor: &mut WorldActor) {
    match actor.task {
        WorldActorTask::RunScript => task_run_script(ctx, world_state, actor),
        WorldActorTask::PaletteLoad => task_palette_load(world_state, actor),
        WorldActorTask::PaletteLoadModes => task_palette_load_mode(actor),
        WorldActorTask::ScrollWorldLayers { world } => task_layer_animation(world_state, world),
        WorldActorTask::FadeIn => task_fade_in(ctx, actor),
        WorldActorTask::FadeOut => task_fade_out(ctx, actor),
        WorldActorTask::UnknownGrp => task_grp(ctx, actor),
        WorldActorTask::PartyLeader => task_party_leader(ctx, world_state, actor),
        WorldActorTask::PartyFollowers => task_party_followers(ctx, world_state, actor),
        WorldActorTask::EpochUnk1 => task_epoch_unk1(ctx, actor),
        WorldActorTask::EpochUnk2 => task_epoch_unk2(ctx, actor),
        WorldActorTask::ProcessMovementUnk1 => task_party_process_movement_1(ctx, actor),
        WorldActorTask::ProcessMovementUnk2 => task_party_process_movement_2(ctx, actor),
        WorldActorTask::UpdateMusic => task_update_music(ctx, actor),
        WorldActorTask::UnkStringRelated => task_unknown_string(ctx, actor),

        WorldActorTask::Unknown { address: _ } | WorldActorTask::None => {},
    }
}
