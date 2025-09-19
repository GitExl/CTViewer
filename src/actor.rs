use std::f64::consts::PI;
use bitflags::bitflags;
use crate::Context;
use crate::scene::scene_map::SceneMap;
use crate::scene_script::scene_script::ActorScriptState;
use crate::sprites::sprite_renderer::SpritePriority;
use crate::sprites::sprite_state::SpriteState;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DebugSprite {
    None,
    Moving,
    Waiting,
    Animating,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ActorClass {
    None,
    PC1,
    PC2,
    PC3,
    PCOutOfParty,
    NPC,
    Monster,
    MonsterPeaceful,
}

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub enum Facing {
    Up,
    #[default]
    Down,
    Left,
    Right,
}

impl Facing {
    pub fn to_index(&self) -> usize {
        match self {
            Facing::Up => 0,
            Facing::Down => 1,
            Facing::Left => 2,
            Facing::Right => 3,
        }
    }

    pub fn from_index(index: usize) -> Facing {
        match index {
            0 => Facing::Up,
            1 => Facing::Down,
            2 => Facing::Left,
            3 => Facing::Right,
            _ => Facing::default(),
        }
    }
}

bitflags! {
    #[derive(Clone, Default, Copy, Debug, PartialEq)]
    pub struct ActorFlags: u32 {
        /// Script execution is disabled.
        const SCRIPT_DISABLED = 0x0001;

        /// Actor is visible.
        const VISIBLE = 0x0002;

        /// Actor is processed by the sprite renderer.
        const RENDERED = 0x0004;

        /// Actor is solid to other actors for collision detection.
        const SOLID = 0x0008;

        /// Actor can have its touch and activate functions triggered.
        const INTERACTABLE = 0x0010;

        /// Actor collides with solid tiles.
        const COLLISION_WITH_TILES = 0x0020;

        /// Actor avoids collision with players.
        const COLLISION_AVOID_PC = 0x0040;

        /// Actor movement end on tiles.
        const MOVE_ONTO_TILE = 0x0080;

        /// Actor movement ends on other actors (?).
        const MOVE_ONTO_ACTOR = 0x0100;

        /// Actor is currently in battle.
        const IN_BATTLE = 0x0200;

        /// Actor can be pushed.
        const PUSHABLE = 0x0400;

        /// Actor remains static during battles.
        const BATTLE_STATIC = 0x0800;

        /// Actor is dead.
        const DEAD = 0x1000;
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
    pub class: ActorClass,
    pub player_index: Option<usize>,
    pub facing: Facing,
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
            class: ActorClass::None,
            player_index: None,
            facing: Facing::default(),
            move_speed: 1.0,
            flags: ActorFlags::COLLISION_WITH_TILES | ActorFlags::COLLISION_AVOID_PC | ActorFlags::INTERACTABLE,
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
        sprite_state.facing = self.facing;
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

        self.facing = match (angle / 90.0).floor() as u32 {
            0 => Facing::Down,
            1 => Facing::Left,
            2 => Facing::Up,
            3 => Facing::Right,
            _ => Facing::Up,
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
        println!("  Class {:?}", self.class);
        if let Some(player_index) = self.player_index {
            println!("  Player {}", player_index);
        }
        println!("  At {} x {}", self.x, self.y);
        println!("  Facing: {:?}", self.facing);
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
