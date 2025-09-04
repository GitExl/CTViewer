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
        const VISIBLE = 0x0002;
        const RENDERED = 0x0004;
        const SOLID = 0x0008;
        const TOUCHABLE = 0x0010;
        const COLLISION_TILE = 0x0020;
        const COLLISION_PC = 0x0040;
        const MOVE_ONTO_TILE = 0x0080;
        const MOVE_ONTO_ACTOR = 0x0100;
        const IN_BATTLE = 0x0200;
        const PUSHABLE = 0x0400;
        const BATTLE_STATIC = 0x0800;
    }
}


pub struct Actor {
    pub index: usize,

    pub x: f64,
    pub y: f64,
    last_x: f64,
    last_y: f64,
    lerp_x: f64,
    lerp_y: f64,

    pub sprite_priority_top: SpritePriority,
    pub sprite_priority_bottom: SpritePriority,
    pub class: Option<ActorClass>,
    pub player_index: Option<usize>,
    pub direction: Direction,
    pub move_speed: f64,
    pub flags: ActorFlags,
    pub battle_index: usize,
}

impl Actor {
    pub fn new(index: usize) -> Self {
        Actor {
            index,

            x: 0.0,
            y: 0.0,
            lerp_x: 0.0,
            lerp_y: 0.0,
            last_x: 0.0,
            last_y: 0.0,

            sprite_priority_top: SpritePriority::default(),
            sprite_priority_bottom: SpritePriority::default(),
            class: None,
            player_index: None,
            direction: Direction::default(),
            move_speed: 1.0,
            flags: ActorFlags::empty(),
            battle_index: 0,
        }
    }

    pub fn tick(&mut self, _delta: f64) {
        self.last_x = self.x;
        self.last_y = self.y;
    }

    pub fn lerp(&mut self, lerp: f64) {
        self.lerp_x = self.last_x + (self.x - self.last_x) * lerp;
        self.lerp_y = self.last_y + (self.y - self.last_y) * lerp;
    }

    pub fn update_sprite_state(&self, sprite_state: &mut SpriteState) {
        sprite_state.x = self.x;
        sprite_state.y = self.y;
        sprite_state.direction = self.direction;
        sprite_state.priority_top = self.sprite_priority_top;
        sprite_state.priority_bottom = self.sprite_priority_bottom;
        sprite_state.enabled = self.flags.contains(ActorFlags::VISIBLE);
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
        println!("  Sprite priority top {:?}", self.sprite_priority_top);
        println!("  Sprite priority bottom {:?}", self.sprite_priority_bottom);
        println!("  Flags: {:?}", self.flags);
        println!();

        ctx.sprites_states.get_state(self.index).dump();
    }

}
