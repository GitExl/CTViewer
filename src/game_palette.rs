use crate::software_renderer::palette::Palette;

#[derive(Clone)]
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

    pub fn key_for_sprite_palette(palette_index: usize) -> u64 {
        0x10000000 | (palette_index as u64)
    }
}
