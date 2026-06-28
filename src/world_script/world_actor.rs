use crate::facing::Facing;
use crate::memory::MemoryRegion;
use crate::world_script::task_dispatch::WorldActorTask;

#[derive(Clone)]
pub struct WorldActor {
    pub task: WorldActorTask,
    pub cycles: u16,
    pub counter: u8,

    pub script_return_address: u64,
    pub script_current_address: u64,

    pub palette_priority: u8,
    pub animation_address: u64,
    pub animation_counter: u8,
    pub sprite_assembly_key: u64,
    pub sprite_tile_offset: i32,

    pub memory: MemoryRegion,
    pub x: f64,
    pub y: f64,
    pub vector_x: f64,
    pub vector_y: f64,
    pub facing: Facing,
}

impl WorldActor {
    pub fn dump(&self) {
        println!("World actor");
        println!();
    }

    pub fn new() -> Self {
        Self {
            task: WorldActorTask::None,
            cycles: 0,
            counter: 0,

            script_current_address: 0,
            script_return_address: 0,

            palette_priority: 0,
            animation_counter: 0,
            animation_address: 0,
            sprite_assembly_key: 0,
            sprite_tile_offset: 0,

            memory: MemoryRegion::new(64),
            x: 0.0,
            y: 0.0,
            vector_x: 0.0,
            vector_y: 0.0,
            facing: Facing::Down,
        }
    }
}
