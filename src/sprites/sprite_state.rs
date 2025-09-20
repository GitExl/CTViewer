use crate::actor::{ActorClass, Facing};
use crate::sprites::sprite_anim::SpriteAnim;
use crate::sprites::sprite_renderer::SpritePriority;

#[derive(Clone,PartialEq,Debug)]
pub enum AnimationMode {
    None,
    Loop,
    LoopCount,
    Static,
}

#[derive(Clone)]
pub struct SpriteState {
    pub x: f64,
    pub y: f64,

    pub sprite_index: usize,
    pub sprite_frame: usize,
    pub palette_offset: usize,
    pub facing: Facing,
    pub priority_top: SpritePriority,
    pub priority_bottom: SpritePriority,
    pub enabled: bool,

    pub anim_set_index: usize,
    pub anim_delay: u32,
    pub anim_index: usize,
    pub anim_index_looped: usize,
    pub anim_frame: usize,
    pub anim_frame_static: usize,
    pub anim_mode: AnimationMode,
    pub anim_loops_remaining: u32,
}

impl SpriteState {
    pub fn new() -> SpriteState {
        SpriteState {
            x: 0.0,
            y: 0.0,

            sprite_index: 0,
            sprite_frame: 0,
            palette_offset: 0,
            facing: Facing::default(),
            priority_top: SpritePriority::default(),
            priority_bottom: SpritePriority::default(),
            enabled: false,

            anim_set_index: 0,
            anim_delay: 0,
            anim_index: 0,
            anim_index_looped: 0,
            anim_frame: 0,
            anim_frame_static: 0,
            anim_mode: AnimationMode::None,
            anim_loops_remaining: 0,
        }
    }

    pub fn tick_animation(&mut self, anim: &SpriteAnim) {
        // todo: only do this for sprites that are visible

        if self.anim_delay > 0 {
            self.anim_delay -= 1;
        }
        if self.anim_delay > 0 {
            return;
        }

        // todo: dead actors should stop here
        // if actor.is_dead {
        //     return
        // }

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

    pub fn animate_for_movement(&mut self, actor_class: ActorClass, movement_x: f64, movement_y: f64) {
        if self.anim_mode == AnimationMode::Static {
            if self.anim_index != 0xFF {
                self.anim_mode = AnimationMode::Loop;
                return;
            } else {
                self.anim_mode = AnimationMode::None;
            }
        }

        if self.anim_mode != AnimationMode::None {
            return;
        }

        let movement_total = movement_x.abs() + movement_y.abs();
        let mut anim_index = 0;
        if actor_class != ActorClass::NPC && movement_total > 4.0 {
            anim_index = 6;
        } else if movement_total >= 0.01 {
            anim_index = 1;
        }

        if self.anim_index != anim_index {
            self.anim_index = anim_index;
            self.anim_frame = 0;
            self.anim_delay = 0;
            return;
        }
    }

    pub fn dump(&self) {
        println!("Sprite state - {}", if self.enabled { "enabled" } else { "disabled" });
        println!("  Sprite {} frame {}", self.sprite_index, self.sprite_frame);
        println!("  At {} x {}", self.x, self.y);
        println!("  Facing: {:?}", self.facing);
        println!("  Priority top {:?}", self.priority_top);
        println!("  Priority bottom {:?}", self.priority_bottom);
        println!("  Palette offset {}", self.palette_offset);
        println!("  Animation mode {:?}", self.anim_mode);
        println!("    Index {}, looped index {}", self.anim_index, self.anim_index_looped);
        println!("    Frame {} at {} ticks", self.anim_frame, self.anim_delay);
        println!("    Loops remaining: {}", self.anim_loops_remaining);
        println!();
    }
}
