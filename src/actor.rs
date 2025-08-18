use bitflags::bitflags;
use crate::scene::scene_script::SceneActorScriptState;
use crate::sprites::sprite_manager::SpriteState;

pub enum ActorClass {
    PC1,
    PC2,
    PC3,
    PCOutOfParty,
    NPC,
    Monster,
    MonsterPeaceful,
}

bitflags! {
    #[derive(Clone, Default)]
    pub struct ActorFlags: u32 {
        const ENABLED = 0x0001;
        const HIDDEN = 0x0002;
        const SOLID = 0x0004;
    }
}


pub struct Actor {
    pub x: f64,
    pub y: f64,
    pub sprite_priority: u32,
    pub class: ActorClass,
    pub player_index: usize,
    pub direction: usize,
    pub move_speed: f64,
    pub flags: ActorFlags,

    pub sprite_state: SpriteState,
    pub script_state: SceneActorScriptState,
}

impl Actor {
    pub fn spawn(x: f64, y: f64, sprite_index: usize, direction: usize) -> Self {
        Actor {
            x, y,
            sprite_priority: 2,
            class: ActorClass::NPC,
            player_index: 0,
            direction,
            move_speed: 1.0,
            flags: ActorFlags::empty(),
            sprite_state: SpriteState::new(sprite_index, direction),
            script_state: SceneActorScriptState::new(),
        }
    }

    pub fn tick(&mut self, _delta: f64) {
    }
}
