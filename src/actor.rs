use crate::sprites::sprite_manager::SpriteState;

pub struct Actor {
    pub x: f64,
    pub y: f64,
    pub priority: u32,

    pub sprite_state: SpriteState,
}

impl Actor {
    pub fn spawn(x: f64, y: f64, sprite_index: usize, direction: usize) -> Self {
        Actor {
            x,
            y,
            priority: 2,
            sprite_state: SpriteState::new(sprite_index, direction),
        }
    }

    pub fn tick(&mut self, _delta: f64) {
    }
}
