use sdl3::timer;

pub struct Timer {
    freq: f64,
    start: u64,
    end: u64,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            start: timer::performance_counter(),
            end: 0,
            freq: timer::performance_frequency() as f64,
        }
    }

    pub fn start(&mut self) {
        self.start = timer::performance_counter();
    }

    pub fn stop(&mut self) -> f64 {
        self.end = timer::performance_counter();
        (self.end - self.start) as f64 / self.freq
    }

    pub fn elapsed(&self) -> f64 {
        (timer::performance_counter() - self.start) as f64 / self.freq
    }
}
