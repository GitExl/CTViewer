use std::io::Cursor;
use byteorder::ReadBytesExt;
use crate::software_renderer::blit::SurfaceBlendOps;
use crate::software_renderer::draw::draw_box;
use crate::software_renderer::surface::Surface;

pub type Color = [u8; 4];

#[derive(Clone)]
pub struct Palette {
    pub colors: Vec<Color>,
}

impl Palette {
    pub fn new(color_count: usize) -> Palette {
        Palette {
            colors: vec![[0, 0, 0, 0]; color_count],
        }
    }

    pub fn from_colors(colors: &Vec<Color>) -> Palette {
        Palette {
            colors: colors.clone(),
        }
    }
}

pub fn render_palette(palette: &Palette, surface: &mut Surface, scale: i32) {
    let mut x = 0;
    let mut y = 0;
    for color in &palette.colors {
        draw_box(surface, x, y, 8, 8, *color, SurfaceBlendOps::Copy);
        x += scale;
        if x >= scale * 16  {
            x = 0;
            y += scale;
        }
    }

}
