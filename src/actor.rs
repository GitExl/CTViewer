use bitflags::bitflags;
use crate::Context;
use crate::facing::Facing;
use crate::scene::scene_map::SceneMap;
use crate::scene_script::scene_script::ActorScriptState;
use crate::sprites::sprite_renderer::SpritePriority;
use crate::sprites::sprite_state::SpriteState;
use crate::util::vec2df64::Vec2Df64;
use crate::util::vec2di32::Vec2Di32;

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

        /// Actor can have functions called on it (including touch and activate).
        const CALLS_ENABLED = 0x0010;

        /// Actor collides with solid tiles.
        const COLLISION_WITH_TILES = 0x0020;

        /// Actor avoids collision with players.
        const COLLISION_AVOID_PC = 0x0040;

        /// Actor movement end on tiles.
        const MOVE_ONTO_TILE = 0x0080;

        /// Actor movements do not avoid other solid actors.
        const MOVE_AROUND_SOLID_ACTORS = 0x0100;

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
        tile_pos: Vec2Di32,
        move_by: Vec2Df64,
        steps: u32,
    },
    MoveByAngle {
        angle: f64,
        move_by: Vec2Df64,
        steps: u32,
    },
}

impl ActorTask {
    pub fn dump(&self) {
        match self {
            ActorTask::None {} => {
                return;
            },
            ActorTask::MoveToTile { tile_pos, move_by, steps } => {
                println!("Moving to tile {}, at {} pixels/s in {} steps", tile_pos, move_by, steps);
            },
            ActorTask::MoveByAngle { angle, move_by, steps } => {
                println!("Moving at angle {}, at {} pixels/s in {} steps", angle, move_by, steps);
            },
        }

    }
}

pub struct Actor {
    pub index: usize,

    pub pos: Vec2Df64,
    pub pos_last: Vec2Df64,
    pub pos_lerp: Vec2Df64,

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
    pub result: u32,
}

impl Actor {
    pub fn new(index: usize) -> Self {
        Actor {
            index,

            pos: Vec2Df64::default(),
            pos_last: Vec2Df64::default(),
            pos_lerp: Vec2Df64::default(),

            task: ActorTask::None,
            debug_sprite: DebugSprite::None,
            sprite_priority_top: SpritePriority::default(),
            sprite_priority_bottom: SpritePriority::default(),
            class: ActorClass::None,
            player_index: None,
            facing: Facing::default(),
            move_speed: 1.0,
            flags: ActorFlags::COLLISION_WITH_TILES | ActorFlags::COLLISION_AVOID_PC | ActorFlags::CALLS_ENABLED,
            battle_index: 0,
            result: 0,
        }
    }

    pub fn tick(&mut self, _delta: f64, scene_map: &SceneMap) {
        self.pos_last = self.pos;

        self.run_task(scene_map);
    }

    pub fn lerp(&mut self, lerp: f64) {
        self.pos_lerp = Vec2Df64::interpolate(self.pos_last, self.pos, lerp);
    }

    pub fn update_sprite_state(&self, sprite_state: &mut SpriteState) {
        sprite_state.pos = self.pos_lerp;
        sprite_state.priority_top = self.sprite_priority_top;
        sprite_state.priority_bottom = self.sprite_priority_bottom;
        sprite_state.enabled = self.flags.contains(ActorFlags::VISIBLE);
    }

    pub fn face_towards(&mut self, pos: Vec2Df64) {
        let mut angle = Vec2Df64::angle_deg_between(self.pos, pos) - 45.0;
        if angle < 0.0 {
            angle += 360.0;
        }
        self.facing = Facing::from_angle(angle);
    }

    pub fn move_to(&mut self, pos: Vec2Df64, warp: bool, scene_map: &SceneMap) {
        self.pos = pos;
        if warp {
            self.pos_last = pos;
        }

        self.update_sprite_priority(scene_map);
    }

    pub fn move_by(&mut self, movement: Vec2Df64, scene_map: &SceneMap) {
        self.pos = self.pos + movement;

        self.update_sprite_priority(scene_map);
    }

    fn run_task(&mut self, scene_map: &SceneMap) {
        match self.task {
            ActorTask::MoveToTile { move_by, ref mut steps, .. } => {
                if *steps == 0 {
                    return;
                }

                *steps -= 1;
                self.pos = self.pos + move_by;
                self.update_sprite_priority(scene_map);
            },
            ActorTask::MoveByAngle { move_by, ref mut steps, .. } => {
                if *steps == 0 {
                    return;
                }

                *steps -= 1;
                self.pos = self.pos + move_by;
                self.update_sprite_priority(scene_map);
            },
            ActorTask::None {} => return,
        }
    }

    pub fn update_sprite_priority(&mut self, scene_map: &SceneMap) {
        let props = scene_map.get_props_at_pixel(self.pos + Vec2Df64::new(0.0, 1.0));
        if let Some(props) = props {
            self.sprite_priority_top = props.sprite_priority_top;
            self.sprite_priority_bottom = props.sprite_priority_bottom;
        }
    }

    pub fn dump(&self, ctx: &Context, script_state: &ActorScriptState) {
        println!("Actor {}", self.index);
        println!("  Class {:?}", self.class);
        if let Some(player_index) = self.player_index {
            println!("  Player {}", player_index);
        }
        println!("  At {}", self.pos);
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
