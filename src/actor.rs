use bitflags::bitflags;
use crate::Context;
use crate::sprites::sprite_state_list::SpriteState;
use crate::sprites::sprite_renderer::SpritePriority;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ActorClass {
    PC1,
    PC2,
    PC3,
    PCOutOfParty,
    NPC,
    Monster,
    MonsterPeaceful,
}

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub enum Direction {
    Up,
    #[default]
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn to_index(&self) -> usize {
        match self {
            Direction::Up => 0,
            Direction::Down => 1,
            Direction::Left => 2,
            Direction::Right => 3,
        }
    }

    pub fn from_index(index: usize) -> Direction {
        match index {
            0 => Direction::Up,
            1 => Direction::Down,
            2 => Direction::Left,
            3 => Direction::Right,
            _ => Direction::default(),
        }
    }
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
    pub index: usize,
    pub x: f64,
    pub y: f64,
    pub sprite_priority: SpritePriority,
    pub class: Option<ActorClass>,
    pub player_index: Option<usize>,
    pub direction: Direction,
    pub move_speed: f64,
    pub flags: ActorFlags,
}

impl Actor {
    pub fn new(index: usize) -> Self {
        Actor {
            index,
            x: 0.0,
            y: 0.0,
            sprite_priority: SpritePriority::default(),
            class: None,
            player_index: None,
            direction: Direction::default(),
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

    pub fn dump(&self, ctx: &Context) {
        println!("Actor {}", self.index);
        if let Some(class) = self.class {
            println!("  Class {:?}", class);
        }
        if let Some(player_index) = self.player_index {
            println!("  Player {}", player_index);
        }
        println!("  At {} x {}", self.x, self.y);
        println!("  Direction: {:?}", self.direction);
        println!("  Speed: {}", self.move_speed);
        println!("  Sprite priority: {:?}", self.sprite_priority);
        println!("  Flags: {:?}", self.flags);
        println!();

        ctx.sprites_states.get_state(self.index).dump();
    }

}
