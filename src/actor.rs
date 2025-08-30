use bitflags::bitflags;
use crate::map_renderer::MapSprite;
use crate::sprites::sprite_list::SpriteState;

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
    pub sprite_priority: u32,
    pub class: ActorClass,
    pub player_index: usize,
    pub direction: usize,
    pub move_speed: f64,
    pub flags: ActorFlags,
}

impl Actor {
    pub fn spawn() -> Self {
        Actor {
            x: 0.0,
            y: 0.0,
            sprite_priority: 0,
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

    pub fn update_map_sprite(&self, sprite_state: &SpriteState, map_sprite: &mut MapSprite) {
        map_sprite.x = self.x;
        map_sprite.y = self.y;
        map_sprite.priority = self.sprite_priority;
        map_sprite.visible = !self.flags.contains(ActorFlags::HIDDEN);

        map_sprite.sprite_index = sprite_state.sprite_index;
        map_sprite.frame = sprite_state.sprite_frame;
        map_sprite.palette_offset = sprite_state.palette_offset;
    }

}
