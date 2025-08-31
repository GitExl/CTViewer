use crate::sprites::sprite_assets::SpriteAssets;
use crate::sprites::sprite_renderer::SpritePriority;
use super::sprite_anim::SpriteAnimFrame;

#[derive(Clone)]
pub struct SpriteState {
    pub x: f64,
    pub y: f64,

    pub sprite_index: usize,
    pub sprite_frame: usize,
    pub palette_offset: usize,
    pub direction: usize,
    pub priority: SpritePriority,
    pub enabled: bool,

    pub anim_index: usize,
    pub anim_frame: usize,
    pub anim_timer: f64,
    pub animating: bool,
}

impl SpriteState {
    pub fn new() -> SpriteState {
        SpriteState {
            x: 0.0,
            y: 0.0,

            sprite_index: 0,
            sprite_frame: 0,
            palette_offset: 0,
            direction: 0,
            priority: SpritePriority::default(),
            enabled: false,

            anim_index: 0,
            anim_frame: 0,
            anim_timer: 0.0,
            animating: false,
        }
    }

    pub fn dump(&self) {
        println!("Sprite state - {}", if self.enabled { "enabled" } else { "disabled" });
        println!("  Sprite {} frame {}", self.sprite_index, self.sprite_frame);
        println!("  At {} x {}", self.x, self.y);
        println!("  Direction: {}", self.direction);
        println!("  Sprite priority: {:?}", self.priority);
        println!("  Palette offset {}", self.palette_offset);
        println!("  Animation {}, frame {}, {}", self.anim_index, self.anim_frame, if self.animating { "enabled" } else { "disabled" });
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

    pub fn set_animation(&mut self, assets: &SpriteAssets, actor_index: usize, anim_index: usize) {
        let state = &self.states[actor_index];
        let (frame, anim_index, frame_index) = self.get_frame_for_animation(assets, state.sprite_index, anim_index, 0);
        let sprite_frame = frame.sprite_frames[state.direction];

        let state = &mut self.states[actor_index];
        state.sprite_frame = sprite_frame;
        state.anim_index = anim_index;
        state.anim_frame = frame_index;
        state.anim_timer = 0.0;
        state.animating = true;
    }

    pub fn set_direction(&mut self, assets: &SpriteAssets, actor_index: usize, direction: usize) {
        let state = &self.states[actor_index];
        let (frame, _, _) = self.get_frame_for_animation(assets, state.sprite_index, state.anim_index, state.anim_frame);
        let sprite_frame = frame.sprite_frames[direction];

        let state = &mut self.states[actor_index];
        state.direction = direction;
        state.sprite_frame = sprite_frame;
    }

    pub fn set_sprite_frame(&mut self, actor_index: usize, frame_index: usize) {
        let state = &mut self.states[actor_index];
        state.sprite_frame = frame_index;
        state.animating = false;
    }

    fn get_frame_for_animation(&self, assets: &SpriteAssets, sprite_index: usize, anim_index: usize, frame_index: usize) -> (SpriteAnimFrame, usize, usize) {
        let sprite = assets.get(sprite_index);
        let anim_set = assets.get_anim_set(sprite.anim_set_index);

        let real_anim_index = if anim_set.anims.len() <= anim_index {
            println!("Warning: sprite {} does not have animation {}. Using animation 0.", sprite_index, anim_index);
            0
        } else {
            anim_index
        };

        let anim = &anim_set.anims[real_anim_index];
        let real_frame_index = if anim.frames.len() == 0 {
            println!("Warning: sprite {} animation {} does not have frame {}. Using frame 0.", sprite_index, anim_index, frame_index);
            0
        } else {
            frame_index
        };

        (anim.frames[real_frame_index], real_anim_index, real_frame_index)
    }

    // Updates sprite state.
    pub fn tick(&mut self, assets: &SpriteAssets, delta: f64, actor_index: usize) {
        let state = self.states.get_mut(actor_index).unwrap();
        if !state.animating {
            return;
        }

        // Get the current visible animation frame through the sprite's animation set.
        let sprite = assets.get(state.sprite_index);
        let anim_set = assets.get_anim_set(sprite.anim_set_index);
        let anim = &anim_set.anims[state.anim_index];
        let frame = &anim.frames[state.anim_frame];

        // 0-duration frames show indefinitely.
        if frame.duration == 0.0 {
            return;
        }

        // Advance animation time.
        state.anim_timer += delta;
        if state.anim_timer < frame.duration {
            return;
        }

        // Advance to the next frame.
        state.anim_timer -= frame.duration;
        state.anim_frame += 1;
        if state.anim_frame >= anim.frames.len() {
            state.anim_frame = 0;
        }
        state.sprite_frame = anim.frames[state.anim_frame].sprite_frames[state.direction];
    }

}
