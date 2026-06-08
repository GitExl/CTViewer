use crate::Context;
use crate::gamestate::gamestate_world::WorldState;
use crate::world_script::task_dispatch::{task_dispatch, WorldActorTask};
use crate::world_script::world_script_disassembler::WorldScriptDisassembler;

pub fn world_script_initialize(world_state: &mut WorldState) {
    world_script_add_actor(world_state, 0, WorldActorTask::RunScript);
}

pub fn world_script_run(ctx: &mut Context, world_state: &mut WorldState) {
    for actor_index in 0..world_state.actors.len() {
        let actor = &world_state.actors[actor_index];
        if matches!(actor.task, WorldActorTask::None) {
            continue;
        }

        let cycles = actor.cycles.wrapping_add(1);
        task_dispatch(ctx, world_state, actor_index, actor.task);
        world_state.actors.get_mut(actor_index).unwrap().cycles = cycles;
    }
}

pub fn world_script_add_special_actor(world_state: &mut WorldState, source_actor_index: usize, task: WorldActorTask, ) -> usize {
    for index in 0..4 {
        let state = &world_state.actors[index];
        if matches!(state.task, WorldActorTask::None) {
            let source_actor = &world_state.actors[source_actor_index];
            world_state.actors[index] = source_actor.clone();
            world_state.actors[index].task = task;
            // Clear what is in $02 once we know what it is.
            world_state.actors[index].cycles = 0;
            return index;
        }
    }

    panic!("Out of world special actor slots!");
}

pub fn world_script_add_actor(world_state: &mut WorldState, source_actor_index: usize, task: WorldActorTask) -> usize {
    for index in 4..world_state.actors.len() {
        let actor = &world_state.actors[index];
        if matches!(actor.task, WorldActorTask::None) {
            let source_actor = &world_state.actors[source_actor_index];
            world_state.actors[index] = source_actor.clone();
            world_state.actors[index].task = task;
            // Clear what is in $02 once we know what it is.
            world_state.actors[index].cycles = 0;
            return index;
        }
    }

    panic!("Out of world actor slots!");
}

pub fn world_script_disassemble(ctx: &Context, data: &Vec<u8>) {
    let mut disassembler = WorldScriptDisassembler::new(data, ctx.mode);
    disassembler.disassemble();
    disassembler.dump();
}
