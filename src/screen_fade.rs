use crate::Context;

pub struct ScreenFade {
    active: bool,
    current: f64,
    last: f64,
    target: f64,
    speed: f64,
}

impl ScreenFade {
    pub fn new() -> ScreenFade {
        ScreenFade {
            active: false,
            current: 0.0,
            last: 0.0,
            target: 0.0,
            speed: 0.0,
        }
    }

    pub fn start(&mut self, target: f64, speed: f64) {
        self.target = target;
        self.speed = speed;
        self.active = true;
    }

    pub fn set(&mut self, to: f64) {
        self.target = to;
        self.current = to;
        self.active = true;
    }

    pub fn tick(&mut self, _delta: f64) {
        if !self.active {
            return;
        }
        if self.current == self.target {
            self.active = false;
            return;
        }

        self.last = self.current;

        let step = (self.target - self.current).signum() * self.speed;
        self.current += step;

        // End fade, but leave disabling until after another render has happened.
        if (step > 0.0 && self.current >= self.target) || (step < 0.0 && self.current <= self.target) {
            self.current = self.target;
        }
    }

    pub fn render(&self, ctx: &mut Context, lerp: f64) {
        if !self.active {
            return;
        }

        let alpha = self.last + (self.current - self.last) * lerp;
        ctx.render.set_fade_alpha(255 - (alpha * 255.0) as u8);
    }
}
