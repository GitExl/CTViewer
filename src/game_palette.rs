use crate::software_renderer::palette::Palette;

pub struct GamePalette {
    pub index: usize,
    pub palette: Palette,
}

impl GamePalette {
    pub fn dump(&self) {
        println!("Palette {}", self.index);
        println!("  Colors: {}", self.palette.colors.len());
        println!();
    }
}
