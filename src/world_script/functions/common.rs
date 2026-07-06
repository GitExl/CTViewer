use crate::Context;
use crate::gamestate::gamestate_world::WorldState;
use crate::util::rect::Rect;
use crate::world_script::world_actor::WorldActor;

pub fn func_seagull_random_pos(ctx: &mut Context, actor: &mut WorldActor, world_state: &mut WorldState) {
    actor.pos.x = world_state.camera.pos.x + world_state.camera.size.x / 2.0 - 128.0 + ctx.random.get_u8() as f64;
    actor.pos.y = world_state.camera.pos.y + world_state.camera.size.y + 32.0;
}

pub fn func_seagull_random_vector(ctx: &mut Context, actor: &mut WorldActor) {
    static VECTORS: [[f64; 2]; 16] = [
        [0.0, -1.0],
        [0.0, -1.5],
        [0.0, -1.0],
        [0.7071075439453125, -0.7071075439453125],
        [1.414215087890625, -1.414215087890625],
        [0.7071075439453125, -0.7071075439453125],
        [0.5, -0.86602783203125],
        [0.75, -1.299041748046875],
        [0.5, -0.86602783203125],
        [0.5, -0.86602783203125],
        [0.75, -1.299041748046875],
        [0.5, -0.86602783203125],
        [-0.5, -0.86602783203125],
        [-0.25, -1.299041748046875],
        [-0.5, -0.86602783203125],
        [0.0, -1.0],
    ];

    let index = (ctx.random.get_u8() & 0x0F) as usize;
    actor.vec.x = VECTORS[index][0];
    actor.vec.y = VECTORS[index][1];
}

pub fn func_actor_is_offscreen(ctx: &mut Context, actor: &mut WorldActor, world_state: &mut WorldState) {
    let camera_rect = Rect::new(
        world_state.camera.pos.x as i32,
        world_state.camera.pos.y as i32,
        world_state.camera.pos.x as i32 + world_state.camera.size.x as i32,
        world_state.camera.pos.y as i32 + world_state.camera.size.y as i32,
    );
    let actor_rect = Rect::new(
        actor.pos.x as i32 - 32,
        actor.pos.y as i32 - 32,
        actor.pos.x as i32 + 32,
        actor.pos.y as i32 + 32,
    );

    if actor_rect.intersects(&camera_rect) {
        ctx.memory.put_u8(0x7E0000, 1);
    } else {
        ctx.memory.put_u8(0x7E0000, 0);
    }
}

pub fn func_copy_party_indices(ctx: &mut Context) {
    for (index, slot) in ctx.party.get_active_party_slots().enumerate() {
        let value = if slot.disabled {
            slot.character_id as u8 | 0x80
        } else {
            slot.character_id as u8
        };
        ctx.memory.put_u8(0x7E1BA3 + index, value);
    }
}
