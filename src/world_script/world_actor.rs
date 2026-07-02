use crate::facing::Facing;
use crate::memory::MemoryRegion;
use crate::util::vec2df64::Vec2Df64;
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
    pub pos: Vec2Df64,
    pub pos_last: Vec2Df64,
    pub pos_lerp: Vec2Df64,
    pub vec: Vec2Df64,
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

            pos: Vec2Df64::default(),
            pos_last: Vec2Df64::default(),
            pos_lerp: Vec2Df64::default(),
            vec: Vec2Df64::default(),

            memory: MemoryRegion::new(64),
            facing: Facing::Down,
        }
    }
}
