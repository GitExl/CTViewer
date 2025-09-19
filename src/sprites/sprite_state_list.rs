use crate::actor::Direction;
use crate::sprites::sprite_assets::SpriteAssets;
use crate::sprites::sprite_state::SpriteState;

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

    pub fn set_direction(&mut self, assets: &SpriteAssets, actor_index: usize, direction: Direction) {
        let state = &self.states[actor_index];
        if state.direction == direction {
            return;
        }

        let frame = assets.get_frame_for_animation(state.sprite_index, state.anim_index, state.anim_frame);
        let sprite_frame = if let Some(frame) = frame {
            frame.sprite_frames[direction.to_index()]
        } else {
            0
        };

        let state = &mut self.states[actor_index];
        state.direction = direction;
        state.sprite_frame = sprite_frame;
    }

    pub fn set_sprite_frame(&mut self, actor_index: usize, frame_index: usize) {
        let state = &mut self.states[actor_index];
        state.sprite_frame = frame_index;
    }

    // Updates sprite state.
    pub fn tick(&mut self, assets: &SpriteAssets, actor_index: usize) {
        let state = self.states.get_mut(actor_index).unwrap();

        if state.anim_index != 0xFF {
            let sprite = assets.get(state.sprite_index);
            let anim_set = assets.get_anim_set(sprite.anim_set_index);

            let anim_index = state.tick_animation(&anim_set.anims);

            let anim = &anim_set.anims[anim_index];
            let frame = &anim.frames[state.anim_frame];
            state.sprite_frame = frame.sprite_frames[state.direction.to_index()];
        }
    }

}
