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
}
