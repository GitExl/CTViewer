use crate::util::vec2df64::Vec2Df64;

#[derive(Default, Copy, Clone)]
pub struct ScrollState {
    speed: Vec2Df64,
    time: f64,
}

impl ScrollState {
    pub fn new() -> Self {
        ScrollState {
            speed: Vec2Df64::default(),
            time: 0.0,
        }
    }

    pub fn start(&mut self, speed: Vec2Df64, time: f64) {
        self.speed = speed;
        self.time = time;
    }

    pub fn tick(&mut self, delta: f64) {
        if self.time <= 0.0 {
            return;
        }

        self.time -= delta;
        if self.time <= 0.0 {
            self.speed.x = 0.0;
            self.speed.y = 0.0;
            self.time = 0.0;
        }
    }

    pub fn set_speed(&mut self, speed: Vec2Df64) {
        self.speed = speed;
        self.time = 0.0;
    }

    pub fn get_speed(&self) -> Vec2Df64 {
        self.speed
    }
}
