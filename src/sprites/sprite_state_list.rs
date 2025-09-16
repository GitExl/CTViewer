use crate::actor::Direction;
use crate::sprites::sprite_anim::{SpriteAnim, SpriteAnimFrame};
use crate::sprites::sprite_assets::SpriteAssets;
use crate::sprites::sprite_renderer::SpritePriority;

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

    pub anim_index: usize,
    pub anim_frame: usize,
    pub anim_timer: f64,
    pub anim_enabled: bool,
    pub anim_loop_count: u32,
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

            anim_index: 0,
            anim_frame: 0,
            anim_timer: 0.0,
            anim_enabled: false,
            anim_loop_count: 0,
        }
    }

    pub fn tick_animation(&mut self, anim: &SpriteAnim, frame: &SpriteAnimFrame, delta: f64) {

        // 0-duration frames show indefinitely?
        if frame.duration == 0.0 {
            return;
        }

        // Advance animation time.
        self.anim_timer += delta;
        if self.anim_timer < frame.duration {
            return;
        }

        // Advance to the next frame.
        self.anim_timer -= frame.duration;
        self.anim_frame += 1;
        if self.anim_frame >= anim.frames.len() {
            self.anim_loop_count += 1;
            self.anim_frame = 0;
        }

        self.sprite_frame = anim.frames[self.anim_frame].sprite_frames[self.direction.to_index()];
    }

    pub fn dump(&self) {
        println!("Sprite state - {}", if self.enabled { "enabled" } else { "disabled" });
        println!("  Sprite {} frame {}", self.sprite_index, self.sprite_frame);
        println!("  At {} x {}", self.x, self.y);
        println!("  Direction: {:?}", self.direction);
        println!("  Priority top {:?}", self.priority_top);
        println!("  Priority bottom {:?}", self.priority_bottom);
        println!("  Palette offset {}", self.palette_offset);
        println!("  Animation {}, frame {}, {}, {} loops", self.anim_index, self.anim_frame, if self.anim_enabled { "enabled" } else { "disabled" }, self.anim_loop_count);
        println!();
    }
}

pub struct SpriteStateList {
    states: Vec<SpriteState>,
}

impl SpriteStateList {
    pub fn new() -> SpriteStateList {
        SpriteStateList {
            states: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.states.clear();
    }

    pub fn add_state(&mut self) -> &mut SpriteState {
        self.states.push(SpriteState::new());
        let index = self.states.len() - 1;
        self.states.get_mut(index).unwrap()
    }

    pub fn get_state(&self, actor_index: usize) -> &SpriteState {
        self.states.get(actor_index).unwrap()
    }

    pub fn get_state_mut(&mut self, actor_index: usize) -> &mut SpriteState {
        self.states.get_mut(actor_index).unwrap()
    }

    pub fn get_all(&self) -> &Vec<SpriteState> {
        &self.states
    }

    pub fn set_animation(&mut self, assets: &SpriteAssets, actor_index: usize, anim_index: usize, animate: bool, direction: Direction) {
        let state = &self.states[actor_index];

        // Only reset playing the animation if a new animation was set.
        let frame_index = if state.anim_index == anim_index {
            state.anim_frame
        } else {
            0
        };

        let (frame, anim_index, frame_index) = assets.get_frame_for_animation(state.sprite_index, anim_index, frame_index);
        let sprite_frame = frame.sprite_frames[direction.to_index()];

        let state = &mut self.states[actor_index];

        // Only reset playing the animation if a new animation was set.
        if state.anim_index != anim_index {
            state.anim_timer = 0.0;
            state.anim_loop_count = 0;
        }

        state.direction = direction;
        state.sprite_frame = sprite_frame;
        state.anim_index = anim_index;
        state.anim_frame = frame_index;
        state.anim_enabled = animate;
}

pub fn set_direction(&mut self, assets: &SpriteAssets, actor_index: usize, direction: Direction) {
        let state = &self.states[actor_index];
        if state.direction == direction {
            return;
        }

        let (frame, _, _) = assets.get_frame_for_animation(state.sprite_index, state.anim_index, state.anim_frame);
        let sprite_frame = frame.sprite_frames[direction.to_index()];

        let state = &mut self.states[actor_index];
        state.direction = direction;
        state.sprite_frame = sprite_frame;
    }

    pub fn set_sprite_frame(&mut self, actor_index: usize, frame_index: usize) {
        let state = &mut self.states[actor_index];
        state.sprite_frame = frame_index;
        state.anim_enabled = false;
    }

    // Updates sprite state.
    pub fn tick(&mut self, assets: &SpriteAssets, delta: f64, actor_index: usize) {
        let state = self.states.get_mut(actor_index).unwrap();

        if state.anim_enabled {
            let sprite = assets.get(state.sprite_index);
            let anim_set = assets.get_anim_set(sprite.anim_set_index);
            let anim = &anim_set.anims[state.anim_index];
            let frame = &anim.frames[state.anim_frame];

            state.tick_animation(anim, frame, delta);
        }
    }

}
