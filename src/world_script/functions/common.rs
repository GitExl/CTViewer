use crate::Context;
use crate::gamestate::gamestate_world::WorldState;
use crate::util::rect::Rect;

pub fn func_seagull_random_pos(ctx: &mut Context, actor_index: usize, world_state: &mut WorldState) {
    let actor = world_state.actors.get_mut(actor_index).unwrap();

    actor.x = 376.0 + ctx.random.get_u8() as f64;
    actor.y = world_state.camera.pos.y + 256.0;
}

pub fn func_seagull_random_vector(ctx: &mut Context, actor_index: usize, world_state: &mut WorldState) {
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

    let actor = world_state.actors.get_mut(actor_index).unwrap();
    let index = (ctx.random.get_u8() & 0x0F) as usize;
    actor.vector_x = VECTORS[index][0];
    actor.vector_y = VECTORS[index][1];
}

pub fn func_actor_is_offscreen(ctx: &mut Context, actor_index: usize, world_state: &mut WorldState) {
    let actor = &world_state.actors[actor_index];

    let camera_rect = Rect::new(
        world_state.camera.pos.x as i32,
        world_state.camera.pos.y as i32,
        world_state.camera.pos.x as i32 + world_state.camera.size.x as i32,
        world_state.camera.pos.y as i32 + world_state.camera.size.y as i32,
    );
    let actor_rect = Rect::new(
        actor.x as i32 - 32,
        actor.y as i32 - 32,
        actor.x as i32 + 32,
        actor.y as i32 + 32,
    );

    if actor_rect.intersects(&camera_rect) {
        ctx.memory.write_u8(0x7E0000, 1);
    } else {
        ctx.memory.write_u8(0x7E0000, 0);
    }
}
