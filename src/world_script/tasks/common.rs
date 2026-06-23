use crate::Context;
use crate::gamestate::gamestate_world::WorldState;
use crate::world_script::task_dispatch::WorldActorTask;
use crate::world_script::world_actor::WorldActor;
use crate::world_script::world_script::world_script_add_actor;

pub fn task_palette_load(world_state: &mut WorldState, actor: &mut WorldActor) {
    actor.task = WorldActorTask::None;

    let palette_index = actor.memory.get_u8(0x32);
    let mode = actor.memory.get_u8(0x34);
    let mut address = actor.memory.get_u24(0x35) as usize;
    if address >= 0xC60000 {
        address -= 0xC00000;
    }

    // Mode 0 copies palette data from palette animations.
    if mode == 0 {

        // Gate world palette animation data.
        // Palette data exists at 0xC6F5E0 as well, used during Epoch warp.
        if address < 0x7EC000 {
            println!("Unhandled palette copy mode 0 address 0x{:06X}", address);
        } else {
            let src_start = ((address - 0x7EC000) / 32) * 16;
            let dest_start = palette_index as usize * 16;
            world_state.palette.palette.colors[dest_start..dest_start + 16].copy_from_slice(&world_state.palette_animation.palette.colors[src_start..src_start + 16]);
        }

    // Other modes.
    } else {
        world_script_add_actor(world_state, &actor, WorldActorTask::PaletteLoadModes);
    }
}

pub fn task_layer_animation(world_state: &mut WorldState, world_index: usize) {
    match world_index {
        0 => {
            world_state.map.layers[2].scroll.x -= 0.25;
            world_state.map.layers[2].scroll.y += 0.125;
        }
        1 => {
            world_state.map.layers[2].scroll.x += 0.25;
            world_state.map.layers[2].scroll.y -= 0.25;
        }
        2 => {
            world_state.map.layers[2].scroll.x -= 30414.00006103516;
            world_state.map.layers[2].scroll.y += 2.0;

            // Do not interpolate this because of the fast scrolling effect.
            world_state.map.layers[2].scroll_last = world_state.map.layers[2].scroll;
            world_state.map.layers[2].scroll_lerp = world_state.map.layers[2].scroll;
        }
        4 => {
            world_state.map.layers[2].scroll.x -= 21003.0002746582;
            world_state.map.layers[2].scroll.y -= 10.0;

            // Do not interpolate this because of the fast scrolling effect.
            world_state.map.layers[2].scroll_last = world_state.map.layers[2].scroll;
            world_state.map.layers[2].scroll_lerp = world_state.map.layers[2].scroll;
        }
        5 => {
            world_state.map.layers[0].scroll.x -= 0.25;
        }
        _ => {},
    }
}

pub fn task_palette_load_mode(actor: &mut WorldActor) {
    actor.task = WorldActorTask::None;
    // TODO: full palette_load behaviour is unknown
}

pub fn task_fade_in(ctx: &mut Context, actor: &mut WorldActor) {
    actor.task = WorldActorTask::None;

    let delay = actor.memory.get_u8(0x0A) as usize;
    ctx.screen_fade.start(1.0, delay);
}

pub fn task_fade_out(ctx: &mut Context, actor: &mut WorldActor) {
    actor.task = WorldActorTask::None;

    let delay = actor.memory.get_u8(0x0A) as usize;
    ctx.screen_fade.start(0.0, delay);
}

pub fn task_grp(_ctx: &mut Context, actor: &mut WorldActor) {
    actor.task = WorldActorTask::None;
}
