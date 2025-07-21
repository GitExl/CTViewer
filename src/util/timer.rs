use sdl2::TimerSubsystem;

pub struct Timer {
    timer: TimerSubsystem,
    freq: f64,
    start: u64,
    end: u64,
}

impl Timer {
    pub fn new(sdl_timer: TimerSubsystem) -> Self {
        Timer {
            start: sdl_timer.performance_counter(),
            end: 0,
            freq: sdl_timer.performance_frequency() as f64,
            timer: sdl_timer,
        }
    }

    pub fn start(&mut self) {
        self.start = self.timer.performance_counter();
    }

    pub fn stop(&mut self) -> f64 {
        self.end = self.timer.performance_counter();
        (self.end - self.start) as f64 / self.freq
    }

    pub fn elapsed(&self) -> f64 {
        (self.timer.performance_counter() - self.start) as f64 / self.freq
    }
}
