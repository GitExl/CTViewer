use bitflags::bitflags;
use crate::Context;
use crate::facing::Facing;
use crate::scene::scene_map::SceneMap;
use crate::scene_script::scene_script::ActorScriptState;
use crate::software_renderer::palette::Palette;
use crate::sprites::sprite_anim::SpriteAnim;
use crate::assets::Assets;
use crate::sprites::sprite_renderer::SpritePriority;
use crate::sprites::sprite_state::{AnimationMode, SpriteState, SpriteStateFlags};
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
pub enum SceneActorClass {
    PC1,
    PC2,
    PC3,
    PCOutOfParty,
    NPC,
    Enemy,
    EnemyPeaceful,
    Undefined,
}

impl SceneActorClass {
    pub fn to_index(&self) -> u8 {
        match self {
            SceneActorClass::PC1 => 0,
            SceneActorClass::PC2 => 1,
            SceneActorClass::PC3 => 2,
            SceneActorClass::PCOutOfParty => 3,
            SceneActorClass::NPC => 4,
            SceneActorClass::Enemy => 5,
            SceneActorClass::EnemyPeaceful => 6,
            SceneActorClass::Undefined => 7,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DrawMode {
    Hidden,  // 0x00
    Draw,    // 0x01
    Removed, // 0x80
}

bitflags! {
    #[derive(Clone, Default, Copy, Debug, PartialEq)]
    pub struct SceneActorFlags: u32 {
        /// Script execution is disabled.
        const SCRIPT_DISABLED = 0x0001;

        /// Actor is solid to other actors for collision detection.
        const SOLID = 0x0002;

        /// Actor can have functions called on it (including touch and activate).
        const CALLS_DISABLED = 0x0004;

        /// Actor collides with solid tiles.
        const COLLISION_WITH_TILES = 0x0008;

        /// Actor avoids collision with players.
        const COLLISION_AVOID_PC = 0x0010;

        /// Actor movement end on tiles.
        const MOVE_ONTO_TILE = 0x0020;

        /// Move onto target actor position.
        const MOVE_ONTO_OBJECT = 0x0040;

        /// Actor is currently in battle.
        const IN_BATTLE = 0x0080;

        /// Actor can be pushed.
        const PUSHABLE = 0x0100;

        /// Actor remains static during battles.
        const BATTLE_STATIC = 0x0200;

        /// Actor is dead.
        const DEAD = 0x0400;

        /// Sprite priority does not change from movement.
        const SPRITE_PRIORITY_LOCKED = 0x0800;

        /// Actor is outputting lines into a textbox.
        const TEXTBOX_ACTIVE = 0x1000;
    }
}

pub enum SceneActorTask {
    None,
    MoveToTile {
        tile_pos: Vec2Di32,
        move_by: Vec2Df64,
        cycles: u32,
    },
    MoveToActor {
        actor_index: usize,
        move_by: Vec2Df64,
        cycles: u32,
    },
    MoveByAngle {
        angle: f64,
        move_by: Vec2Df64,
        cycles: u32,
    },
}

impl SceneActorTask {
    pub fn dump(&self) {
        match self {
            SceneActorTask::None {} => {
                return;
            },
            SceneActorTask::MoveToTile { tile_pos, move_by, cycles } => {
                println!("Moving to tile {}, at {} pixels/s in {} script cycles", tile_pos, move_by, cycles);
            },
            SceneActorTask::MoveToActor { actor_index, move_by, cycles } => {
                println!("Moving to actor {}, at {} pixels/s in {} script cycles", actor_index, move_by, cycles);
            },
            SceneActorTask::MoveByAngle { angle, move_by, cycles } => {
                println!("Moving at angle {}, at {} pixels/s in {} script cycles", angle, move_by, cycles);
            },
        }

    }
}

pub struct SceneActor {
    pub index: usize,

    pub pos: Vec2Df64,
    pub pos_last: Vec2Df64,
    pub pos_lerp: Vec2Df64,

    pub task: SceneActorTask,
    pub debug_sprite: DebugSprite,

    pub sprite_info_key: Option<u64>,
    pub sprite_frame: usize,
    pub sprite_priority_top: SpritePriority,
    pub sprite_priority_bottom: SpritePriority,

    pub palette_key: Option<u64>,
    pub palette_offset: usize,
    pub local_palette: Palette,

    pub class: SceneActorClass,
    pub player_index: Option<usize>,
    pub facing: Facing,
    pub move_speed: f64,
    pub flags: SceneActorFlags,
    pub draw_mode: DrawMode,
    pub result: u32,

    pub anim_set_index: usize,
    pub anim_delay: u32,
    pub anim_index: usize,
    pub anim_index_looped: usize,
    pub anim_frame: usize,
    pub anim_frame_static: usize,
    pub anim_mode: AnimationMode,
    pub anim_loops_remaining: u32,
    pub anim_enabled: bool,

    pub battle_index: usize,
    pub movement_unknown: u32,
}

impl SceneActor {
    pub fn new(index: usize) -> Self {
        SceneActor {
            index,

            pos: Vec2Df64::default(),
            pos_last: Vec2Df64::default(),
            pos_lerp: Vec2Df64::default(),

            task: SceneActorTask::None,
            debug_sprite: DebugSprite::None,
            sprite_priority_top: SpritePriority::default(),
            sprite_priority_bottom: SpritePriority::default(),
            class: SceneActorClass::Undefined,
            player_index: None,
            facing: Facing::default(),
            move_speed: (1.0 / 18.0) * 16.0,
            flags: SceneActorFlags::COLLISION_WITH_TILES | SceneActorFlags::COLLISION_AVOID_PC | SceneActorFlags::DEAD,
            draw_mode: DrawMode::Draw,
            result: 0,

            sprite_info_key: None,
            sprite_frame: 0,

            palette_key: None,
            palette_offset: 0,
            local_palette: Palette::new(256),

            anim_set_index: 0,
            anim_delay: 0,
            anim_index: 0,
            anim_index_looped: 0,
            anim_frame: 0,
            anim_frame_static: 0,
            anim_mode: AnimationMode::None,
            anim_loops_remaining: 0,
            anim_enabled: true,

            battle_index: 0,
            movement_unknown: 0,
        }
    }

    pub fn tick(&mut self, _delta: f64, scene_map: &SceneMap, assets: &Assets) {
        self.pos_last = self.pos;

        self.run_task(scene_map);

        self.anim_enabled = !self.flags.contains(SceneActorFlags::DEAD);

        if self.anim_mode == AnimationMode::Static {
            self.sprite_frame = self.anim_frame_static;
        } else {
            let anim_index = if self.anim_mode == AnimationMode::LoopCount {
                self.anim_index_looped
            } else {
                self.anim_index
            };

            let anim_set = assets.get_anim_set(self.anim_set_index);
            let anim = &anim_set.get_anim(anim_index);
            if let Some(anim) = anim {
                self.tick_animation(&anim);
                let frame = &anim.frames[self.anim_frame];
                self.sprite_frame = frame.sprite_frames[self.facing.to_index()];
            }
        }
    }

    pub fn tick_animation(&mut self, anim: &SpriteAnim) {
        // todo: only do this for sprites that are "visible", C0A88D

        if self.anim_delay > 0 {
            self.anim_delay -= 1;
        }
        if self.anim_delay > 0 {
            return;
        }

        // Process delay, but don't animate.
        if !self.anim_enabled {
            return;
        }

        self.anim_frame += 1;

        // Advance to the next frame if available.
        if self.anim_frame < anim.frames.len() {
            self.anim_delay = anim.frames[self.anim_frame].delay;
            return;
        }

        // Otherwise loop back to frame 0.
        self.anim_delay = anim.frames[0].delay;

        // Track loop count in mode 2.
        if self.anim_mode == AnimationMode::LoopCount {
            if self.anim_loops_remaining == 1 {
                self.anim_frame -= 1;
                return;
            } else if self.anim_loops_remaining == 2 {
                self.anim_loops_remaining = 1;
                self.anim_frame -= 1;
                return;
            }

            self.anim_loops_remaining -= 2;
        }

        self.anim_frame = 0;
    }

    pub fn reset_animation(&mut self) {

        // Return if animation is not running.
        if self.anim_mode != AnimationMode::None {
            return;
        }
        // Return if animation is already 0.
        if self.anim_index == 0 {
            return;
        }

        self.anim_index = 0;
        self.anim_frame = 0;
        self.anim_delay = 0;
    }

    pub fn animate_for_movement(&mut self, actor_class: SceneActorClass, move_by: Vec2Df64) {
        if self.anim_mode == AnimationMode::Static {
            if self.anim_index == 0xFF {
                self.anim_mode = AnimationMode::None;
            } else {
                self.anim_mode = AnimationMode::Loop;
                return;
            }
        }

        if self.anim_mode != AnimationMode::None {
            return;
        }

        let movement_total = move_by.x.abs() + move_by.y.abs();
        let mut anim_index = 0;
        if actor_class != SceneActorClass::NPC && movement_total > 4.0 {
            anim_index = 6;
        } else if movement_total >= 0.01 {
            anim_index = 1;
        }

        if self.anim_index != anim_index {
            self.anim_index = anim_index;
            self.anim_frame = 0;
            self.anim_delay = 0;
        }
    }

    pub fn lerp(&mut self, lerp: f64) {
        self.pos_lerp = Vec2Df64::interpolate(self.pos_last, self.pos, lerp);
    }

    pub fn update_sprite_state(&self, sprite_state: &mut SpriteState, assets: &Assets) {
        sprite_state.pos = self.pos_lerp;
        sprite_state.priority_top = self.sprite_priority_top;
        sprite_state.priority_bottom = self.sprite_priority_bottom;

        if self.draw_mode == DrawMode::Draw && !self.flags.contains(SceneActorFlags::DEAD) && self.class != SceneActorClass::Undefined {
            sprite_state.flags.insert(SpriteStateFlags::ENABLED);
        } else {
            sprite_state.flags.remove(SpriteStateFlags::ENABLED);
        }

        // TODO: something faster than copying the entire thing every frame?
        sprite_state.palette.clone_from(&self.local_palette);
        sprite_state.palette_offset = self.palette_offset;

        if let Some(sprite_info_key) = self.sprite_info_key {
            let sprite = assets.get_sprite_info(sprite_info_key);
            let assembly = assets.get_assembly(sprite.assembly_key);
            sprite_state.assembly_key = assembly.frame_keys[self.sprite_frame];
            sprite_state.bitmap_key = sprite.tiles_bitmap_key;
        }
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
            SceneActorTask::MoveToTile { move_by, ref mut cycles, .. } => {
                if *cycles == 0 {
                    return;
                }

                *cycles -= 1;
                self.pos = self.pos + move_by;
                self.update_sprite_priority(scene_map);
            },
            SceneActorTask::MoveToActor { move_by, ref mut cycles, .. } => {
                if *cycles == 0 {
                    return;
                }

                *cycles -= 1;
                self.pos = self.pos + move_by;
                self.update_sprite_priority(scene_map);
            },
            SceneActorTask::MoveByAngle { move_by, ref mut cycles, .. } => {
                if *cycles == 0 {
                    return;
                }

                *cycles -= 1;
                self.pos = self.pos + move_by;
                self.update_sprite_priority(scene_map);
            },
            SceneActorTask::None {} => return,
        }
    }

    pub fn update_sprite_priority(&mut self, scene_map: &SceneMap) {
        if self.flags.contains(SceneActorFlags::SPRITE_PRIORITY_LOCKED) {
            return;
        }

        let props = scene_map.get_props_at_pixel(self.pos);
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
        println!("  Sprite priority top / bottom: {:?} {:?}", self.sprite_priority_top, self.sprite_priority_bottom);
        println!("  Flags: {:?}", self.flags);
        println!("  Draw mode: {:?}", self.draw_mode);
        println!("  At {}", self.pos);
        if let Some(sprite_info_key) = self.sprite_info_key {
            println!("  Sprite {:X} frame {}", sprite_info_key, self.sprite_frame);
        }
        if let Some(palette_key) = self.palette_key {
            println!("  Palette {:X} offset {}", palette_key, self.palette_offset);
        }
        println!("  Animation mode {:?}", self.anim_mode);
        println!("    Index {}, looped index {}", self.anim_index, self.anim_index_looped);
        println!("    Frame {} at {} ticks", self.anim_frame, self.anim_delay);
        println!("    Loops remaining: {}", self.anim_loops_remaining);
        println!();

        self.task.dump();
        println!();

        ctx.sprite_states.get_state(self.index).dump();
        script_state.dump();
    }

}
