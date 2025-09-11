use std::f64::consts::PI;
use bitflags::bitflags;
use crate::Context;
use crate::scene::scene_map::SceneMap;
use crate::scene_script::scene_script::ActorScriptState;
use crate::sprites::sprite_state_list::SpriteState;
use crate::sprites::sprite_renderer::SpritePriority;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DebugSprite {
    None,
    Moving,
    Waiting,
    Animating,
}

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
        const SCRIPT_DISABLED = 0x0001;
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

pub enum ActorTask {
    None,
    MoveToTile {
        tile_x: i32,
        tile_y: i32,
        move_x: f64,
        move_y: f64,
        steps: u32,
    },
    MoveByAngle {
        angle: f64,
        move_x: f64,
        move_y: f64,
        steps: u32,
    },
}

impl ActorTask {
    pub fn dump(&self) {
        match self {
            ActorTask::None {} => {
                return;
            },
            ActorTask::MoveToTile { tile_x, tile_y, move_x, move_y, steps } => {
                println!("Moving to tile {}x{}, by {}x{} pixels in {} steps", tile_x, tile_y, move_x, move_y, steps);
            },
            ActorTask::MoveByAngle { angle, move_x, move_y, steps } => {
                println!("Moving at angle {}, by {}x{} pixels in {} steps", angle, move_x, move_y, steps);
            },
        }

    }
}

pub struct Actor {
    pub index: usize,

    pub x: f64,
    pub y: f64,
    last_x: f64,
    last_y: f64,
    pub lerp_x: f64,
    pub lerp_y: f64,

    pub task: ActorTask,
    pub debug_sprite: DebugSprite,
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

            task: ActorTask::None,
            debug_sprite: DebugSprite::None,
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

    pub fn tick(&mut self, _delta: f64, scene_map: &SceneMap) {
        self.last_x = self.x;
        self.last_y = self.y;

        self.run_task(scene_map);
    }

    pub fn lerp(&mut self, lerp: f64) {
        self.lerp_x = self.last_x + (self.x - self.last_x) * lerp;
        self.lerp_y = self.last_y + (self.y - self.last_y) * lerp;
    }

    pub fn update_sprite_state(&self, sprite_state: &mut SpriteState) {
        sprite_state.x = self.lerp_x;
        sprite_state.y = self.lerp_y;
        sprite_state.direction = self.direction;
        sprite_state.priority_top = self.sprite_priority_top;
        sprite_state.priority_bottom = self.sprite_priority_bottom;
        sprite_state.enabled = self.flags.contains(ActorFlags::VISIBLE);
    }

    pub fn face_towards(&mut self, x: f64, y: f64) {
        let diff_x = x - self.x;
        let diff_y = y - self.y;

        let mut angle = (diff_y.atan2(diff_x) * 180.0 / PI) - 45.0;
        if angle < 0.0 {
            angle += 360.0;
        }

        self.direction = match (angle / 90.0).floor() as u32 {
            0 => Direction::Down,
            1 => Direction::Left,
            2 => Direction::Up,
            3 => Direction::Right,
            _ => Direction::Up,
        };
    }

    pub fn move_to(&mut self, x: f64, y: f64, warp: bool, scene_map: &SceneMap) {
        self.x = x;
        self.y = y;

        if warp {
            self.last_x = x;
            self.last_y = y;
        }

        self.update_sprite_priority(scene_map);
    }

    pub fn move_by(&mut self, x: f64, y: f64, scene_map: &SceneMap) {
        self.x += x;
        self.y += y;

        self.update_sprite_priority(scene_map);
    }

    fn run_task(&mut self, scene_map: &SceneMap) {
        match self.task {
            ActorTask::MoveToTile { move_x, move_y, ref mut steps, .. } => {
                if *steps == 0 {
                    return;
                }

                *steps -= 1;
                self.x += move_x;
                self.y += move_y;
                self.update_sprite_priority(scene_map);
            },
            ActorTask::MoveByAngle { move_x, move_y, ref mut steps, .. } => {
                if *steps == 0 {
                    return;
                }

                *steps -= 1;
                self.x += move_x;
                self.y += move_y;
                self.update_sprite_priority(scene_map);
            },
            ActorTask::None {} => return,
        }
    }

    pub fn update_sprite_priority(&mut self, scene_map: &SceneMap) {
        let props = scene_map.get_props_at_coordinates(self.x, self.y - 1.0);
        if let Some(props) = props {
            if let Some(sprite_priority) = props.sprite_priority {
                self.sprite_priority_top = sprite_priority;
                self.sprite_priority_bottom = sprite_priority;
            }
        }
    }

    pub fn dump(&self, ctx: &Context, script_state: &ActorScriptState) {
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

        self.task.dump();
        println!();

        ctx.sprites_states.get_state(self.index).dump();
        script_state.dump();
    }

}
