use crate::software_renderer::palette::Color;
use crate::software_renderer::palette::Palette;

#[derive(Clone, Copy)]
pub enum PaletteAnimType {
    Sequence,
    CycleForward,
    CycleBackward,
}

impl PaletteAnimType {
    pub fn to_string(&self) -> &str {
        match self {
            PaletteAnimType::Sequence => "Sequence",
            PaletteAnimType::CycleForward => "Cycle forwards",
            PaletteAnimType::CycleBackward => "Cycle backwards",
        }
    }
}

pub struct PaletteAnim {
    pub index: usize,

    pub anim_type: PaletteAnimType,
    pub color_index: usize,
    pub color_count: usize,
    pub delay: f64,

    pub frames: Vec<usize>,
    pub colors: Vec<Color>,
    pub current_frame: usize,
    pub timer: f64,
}

impl PaletteAnim {
    pub fn dump(&self) {
        println!("  Palette animation {}", self.index);
        println!("    Type: {}", self.anim_type.to_string());
        println!("    Color index: {}", self.color_index);
        println!("    Color count: {}", self.color_count);
        println!("    Delay: {:.2} seconds", self.delay);
        if self.frames.len() > 0 {
            println!("    Frames: {:?}", self.frames);
        }
        println!();
    }

    pub fn tick(&mut self, delta: f64, palette: &mut Palette) {
        self.timer += delta;
        if self.timer < self.delay {
            return;
        }
        self.timer -= self.delay;

        // Advance a frame.
        match self.anim_type {
            PaletteAnimType::CycleForward => {
                let last = palette.colors[self.color_index + self.color_count];
                palette.colors.copy_within(self.color_index..self.color_index + self.color_count, self.color_index + 1);
                palette.colors[self.color_index] = last;
            },

            PaletteAnimType::CycleBackward => {
                let first = palette.colors[self.color_index];
                palette.colors.copy_within(self.color_index + 1..self.color_index + self.color_count + 1, self.color_index);
                palette.colors[self.color_index + self.color_count] = first;
            },

            // Advance to the next sequence.
            PaletteAnimType::Sequence => {
                self.current_frame += 1;
                if self.current_frame >= self.frames.len() {
                    self.current_frame = 0;
                }

                // Select the color set for this frame.
                let set_index = self.frames[self.current_frame];
                let src = set_index * self.color_count;
                palette.colors[self.color_index..self.color_index + self.color_count].copy_from_slice(&self.colors[src..src + self.color_count]);
            },
        };
    }
}

pub struct PaletteAnimSet {
    pub index: usize,
    pub anims: Vec<PaletteAnim>,
}

impl PaletteAnimSet {
    pub fn dump(&self) {
        println!("Palette animation set {}", self.index);
        for anim in &self.anims {
            anim.dump();
        }
    }

    pub fn tick(&mut self, delta: f64, palette: &mut Palette) {
        for anim in self.anims.iter_mut() {
            anim.tick(delta, palette);
        }
    }
}