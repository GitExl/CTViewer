use bitflags::bitflags;
use sdl3::ttf::Font;
use sdl3::pixels::Color as SDLColor;
use sdl3::surface::Surface as SDLSurface;
use crate::software_renderer::blit::{blit_surface_to_surface, SurfaceBlendOps};
use crate::software_renderer::palette::Color;
use crate::software_renderer::surface::Surface;

bitflags! {
    #[derive(Clone, Default, Copy)]
    pub struct TextDrawFlags: u8 {
        const SHADOW = 0x01;
    }
}

pub fn text_draw_to_surface(text: &str, sdl_font: &Font, color: Color, flags: TextDrawFlags) -> Surface {
    let sdl_surface = sdl_font
        .render(&text)
        .blended(SDLColor { r: color[2], g: color[1], b: color[0], a: color[3] }).unwrap();

    text_sdl_surface_to_surface(sdl_surface, color, flags)
}

pub fn text_draw_to_surface_wrapped(text: &str, sdl_font: &Font, color: Color, flags: TextDrawFlags, wrap_width: i32) -> Surface {
    let sdl_surface = sdl_font
        .render(&text)
        .blended_wrapped(SDLColor { r: color[2], g: color[1], b: color[0], a: color[3] }, wrap_width).unwrap();

    text_sdl_surface_to_surface(sdl_surface, color, flags)
}

fn text_sdl_surface_to_surface(sdl_surface: SDLSurface, color: Color, flags: TextDrawFlags) -> Surface {
    let mut text;
    unsafe {
        text = Surface::from_data(
            sdl_surface.width() + 1,
            sdl_surface.height() + 1,
            sdl_surface.height(),
            sdl_surface.pitch(),
            sdl_surface.without_lock().unwrap().to_vec(),
        );
    }

    if flags.contains(TextDrawFlags::SHADOW) {

        // Simulate the Chrono Trigger SNES character shadow colors.
        // 47 if the color is 223.
        // 23 if the color is 223.
        let color_mid = [
            (color[0] as f64 * 0.214) as u8,
            (color[1] as f64 * 0.214) as u8,
            (color[2] as f64 * 0.214) as u8,
            color[3],
        ];
        let color_side = [
            (color[0] as f64 * 0.107) as u8,
            (color[1] as f64 * 0.107) as u8,
            (color[2] as f64 * 0.107) as u8,
            color[3],
        ];

        let mut shadow = Surface::new(text.width, text.height);
        for y in 0..text.height - 1 {
            for x in 0..text.width - 1 {
                let src = ((y * text.width + x) * 4) as usize;
                if text.data[src + 3] == 0 {
                    continue;
                }

                let mut dest = src + 4;
                if text.data[dest + 3] == 0 {
                    shadow.data[dest..dest + 4].copy_from_slice(&color_side);
                }

                dest += text.width as usize * 4;
                if text.data[dest + 3] == 0 {
                    shadow.data[dest..dest + 4].copy_from_slice(&color_mid);
                }

                dest -= 4;
                if text.data[dest + 3] == 0 {
                    shadow.data[dest..dest + 4].copy_from_slice(&color_side);
                }
            }
        }

        blit_surface_to_surface(&shadow, &mut text, 0, 0, shadow.width as i32, shadow.height as i32, 0, 0, SurfaceBlendOps::Blend);
    }

    text
}
