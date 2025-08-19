use bitflags::bitflags;
use crate::map_renderer::MapSprite;
use crate::scene::scene_script::ActorScriptState;
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

    pub sprite_state: Option<SpriteState>,
    pub script_state: Option<ActorScriptState>,
}

impl Actor {
    pub fn spawn() -> Self {
        Actor {
            x: 0.0,
            y: 0.0,
            sprite_priority: 0,
            class: ActorClass::NPC,
            player_index: 0,
            direction: 0,
            move_speed: 1.0,
            flags: ActorFlags::empty(),
            sprite_state: None,
            script_state: None,
        }
    }

    pub fn tick(&mut self, _delta: f64) {
    }

    pub fn lerp(&mut self, _lerp: f64) {
    }

    pub fn update_map_sprite(&self, sprite: &mut MapSprite) {
        sprite.x = self.x;
        sprite.y = self.y;
        sprite.priority = self.sprite_priority;

        match &self.sprite_state {
            Some(state) => {
                sprite.sprite_index = state.sprite_index;
                sprite.frame = state.sprite_frame;
                sprite.palette_offset = state.palette_offset;
            },
            None => {},
        }
    }

}
