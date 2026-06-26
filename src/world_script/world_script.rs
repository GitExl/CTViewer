use crate::Context;
use crate::gamestate::gamestate_world::WorldState;
use crate::world::world_exit::WorldTrigger;
use crate::world_script::task_dispatch::{task_dispatch, WorldActorTask};
use crate::world_script::world_actor::WorldActor;
use crate::world_script::world_script_disassembler::WorldScriptDisassembler;

pub fn world_script_initialize(world_state: &mut WorldState) {
    world_state.actors[0].task = WorldActorTask::RunScript;
}

pub fn world_script_run(ctx: &mut Context, world_state: &mut WorldState) {
    for actor_index in 0..world_state.actors.len() {
        let actor = &world_state.actors[actor_index];
        if matches!(actor.task, WorldActorTask::None) {
            continue;
        }

        // Clone actor, update it, then place it back into the actor list.
        let mut actor_dup = actor.clone();
        task_dispatch(ctx, world_state, &mut actor_dup);
        actor_dup.cycles += 1;
        world_state.actors[actor_index] = actor_dup;
    }
}

pub fn world_script_add_special_actor(world_state: &mut WorldState, source_actor: &WorldActor, task: WorldActorTask, ) -> usize {
    for index in 0..4 {
        let state = &world_state.actors[index];
        if matches!(state.task, WorldActorTask::None) {
            world_state.actors[index] = source_actor.clone();
            world_state.actors[index].task = task;
            world_state.actors[index].cycles = 0;
            return index;
        }
    }

    panic!("Out of special world actor slots!");
}

pub fn world_script_add_actor(world_state: &mut WorldState, source_actor: &WorldActor, task: WorldActorTask) -> usize {
    for index in 4..world_state.actors.len() {
        let actor = &world_state.actors[index];
        if matches!(actor.task, WorldActorTask::None) {
            world_state.actors[index] = source_actor.clone();
            world_state.actors[index].task = task;
            world_state.actors[index].cycles = 0;
            return index;
        }
    }

    panic!("Out of world actor slots!");
}

pub fn world_script_disassemble(ctx: &Context, data: &Vec<u8>, scripted_exits: &Vec<WorldTrigger>, script_addresses: &Vec<u64>) {
    let mut disassembler = WorldScriptDisassembler::new(data, scripted_exits, script_addresses, ctx.mode);
    disassembler.disassemble();
    disassembler.dump();
}
