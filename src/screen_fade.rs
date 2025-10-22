use crate::renderer::Renderer;

pub struct ScreenFade {
    active: bool,
    current: f64,
    last: f64,
    target: f64,
    delay: usize,
    delay_counter: usize,
}

impl ScreenFade {
    pub fn new(current: f64) -> ScreenFade {
        ScreenFade {
            active: false,
            current,
            last: current,
            target: current,
            delay: 2,
            delay_counter: 2,
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
        self.last = self.current;

        if !self.active {
            return;
        }
        if self.current == self.target {
            self.active = false;
            return;
        }

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

    pub fn render(&self, render: &mut Renderer, lerp: f64) {
        let alpha = self.last + (self.current - self.last) * lerp;
        render.set_fade_alpha(255 - (alpha * 255.0) as u8);
    }
}
