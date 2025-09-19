use crate::actor::{ActorClass, Direction};
use crate::sprites::sprite_anim::SpriteAnim;
use crate::sprites::sprite_renderer::SpritePriority;

#[derive(Clone,PartialEq,Debug)]
pub enum AnimationMode {
    None,
    LoopForever,
    Loop,
    Static,
}

#[derive(Clone)]
pub struct SpriteState {
    pub x: f64,
    pub y: f64,

    pub sprite_index: usize,
    pub sprite_frame: usize,
    pub palette_offset: usize,
    pub direction: Direction,
    pub priority_top: SpritePriority,
    pub priority_bottom: SpritePriority,
    pub enabled: bool,

    pub anim_delay: u32,
    pub anim_index: usize,
    pub anim_index_loop: usize,
    pub anim_frame: usize,
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
            direction: Direction::default(),
            priority_top: SpritePriority::default(),
            priority_bottom: SpritePriority::default(),
            enabled: false,

            anim_delay: 0,
            anim_index: 0,
            anim_index_loop: 0,
            anim_frame: 0,
            anim_mode: AnimationMode::None,
            anim_loops_remaining: 0,
        }
    }

    pub fn tick_animation(&mut self, anims: &Vec<SpriteAnim>) -> usize {
        // todo: only do this for sprites that are visible

        let anim_index = if self.anim_mode == AnimationMode::Loop {
            self.anim_index_loop
        } else {
            self.anim_index
        };

        if self.anim_delay > 0 {
            self.anim_delay -= 1;
        }
        if self.anim_delay > 0 {
            return anim_index;
        }

        // todo: dead actors should stop here
        // if actor.is_dead {
        //     return
        // }

        self.anim_frame += 1;

        // Advance to next frame if available.
        let anim = &anims[anim_index];
        if self.anim_frame < anim.frames.len() {
            self.anim_delay = anim.frames[self.anim_frame].duration;
            return anim_index;
        }
        // Otherwise loop back to frame 0.
        self.anim_delay = anim.frames[0].duration;

        // Track loop count in mode 2.
        if self.anim_mode == AnimationMode::Loop {
            if self.anim_loops_remaining > 1 {
                self.anim_loops_remaining -= 1;

            // If loop count is reached, undo the advance to the next frame.
            } else {
                self.anim_loops_remaining = 0;
                self.anim_frame -= 1;
            }

            return anim_index;
        }

        self.anim_frame = 0;

        anim_index
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
                self.anim_mode = AnimationMode::LoopForever;
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
        println!("  Direction: {:?}", self.direction);
        println!("  Priority top {:?}", self.priority_top);
        println!("  Priority bottom {:?}", self.priority_bottom);
        println!("  Palette offset {}", self.palette_offset);
        println!("  Animation mode {:?}, animation {}, frame {} at {} ticks, {} loops remaining", self.anim_mode, self.anim_index, self.anim_frame, self.anim_delay, self.anim_loops_remaining);
        println!();
    }
}
