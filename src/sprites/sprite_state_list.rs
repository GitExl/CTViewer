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

    pub fn tick(&mut self, assets: &SpriteAssets, actor_index: usize) {
        let state = self.states.get_mut(actor_index).unwrap();

        if state.anim_index == 0xFF {
            return;
        }

        let anim_set = assets.get_anim_set(state.anim_set_index);
        let anim_index = state.tick_animation(&anim_set);
        let anim = &anim_set.get_anim(anim_index);
        if let Some(anim) = anim {
            let frame = &anim.frames[state.anim_frame];
            state.sprite_frame = frame.sprite_frames[state.facing.to_index()];
        }
    }

}
