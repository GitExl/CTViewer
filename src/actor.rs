use bitflags::bitflags;
use crate::sprites::sprite_state_list::SpriteState;
use crate::sprites::sprite_renderer::SpritePriority;

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
    #[derive(Clone, Default, Copy, Debug, PartialEq)]
    pub struct ActorFlags: u32 {
        const DISABLED = 0x0001;
        const HIDDEN = 0x0002;
        const RENDERED = 0x0004;
        const SOLID = 0x0008;
        const TOUCHABLE = 0x0010;
        const COLLISION_TILE = 0x0020;
        const COLLISION_PC = 0x0040;
        const MOVE_ONTO_TILE = 0x0080;
        const MOVE_ONTO_OBJECT = 0x0100;
        const IN_BATTLE = 0x0200;
        const PUSHABLE = 0x0400;
    }
}


pub struct Actor {
    pub x: f64,
    pub y: f64,
    pub sprite_priority: SpritePriority,
    pub class: ActorClass,
    pub player_index: usize,
    pub direction: usize,
    pub move_speed: f64,
    pub flags: ActorFlags,
}

impl Actor {
    pub fn new() -> Self {
        Actor {
            x: 0.0,
            y: 0.0,
            sprite_priority: SpritePriority::AboveAll,
            class: ActorClass::NPC,
            player_index: 0,
            direction: 1,
            move_speed: 1.0,
            flags: ActorFlags::empty(),
        }
    }

    pub fn tick(&mut self, _delta: f64) {
    }

    pub fn lerp(&mut self, _lerp: f64) {
    }

    pub fn update_sprite_state(&self, sprite_state: &mut SpriteState) {
        sprite_state.x = self.x;
        sprite_state.y = self.y;
        sprite_state.direction = self.direction;
        sprite_state.priority = self.sprite_priority;
        sprite_state.enabled = !self.flags.contains(ActorFlags::HIDDEN);
    }

}
