use crate::Context;

pub struct ScreenFade {
    active: bool,
    current: f64,
    last: f64,
    target: f64,
    delay: usize,
    delay_counter: usize,
}

impl ScreenFade {
    pub fn new() -> ScreenFade {
        ScreenFade {
            active: false,
            current: 0.0,
            last: 0.0,
            target: 0.0,
            delay: 0,
            delay_counter: 0,
        }
    }

    pub fn start(&mut self, target: f64, delay: usize) {
        self.target = target;
        self.delay = delay - 1;
        self.delay_counter = delay - 1;
        self.active = true;
    }

    pub fn set(&mut self, to: f64) {
        self.target = to;
        self.current = to;
        self.active = true;
    }

    pub fn is_active(&self) -> bool {
        self.active
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

        if self.delay_counter == 0 {
            self.delay_counter = self.delay;

            let step = (self.target - self.current).signum() * (1.0 / 16.0);
            self.current += step;

            // End fade, but leave disabling until after another render has happened.
            if (step > 0.0 && self.current >= self.target) || (step < 0.0 && self.current <= self.target) {
                self.current = self.target;
            }
        } else {
            self.delay_counter -= 1;
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
