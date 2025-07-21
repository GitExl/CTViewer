use super::blit::SurfaceBlendOps;
use super::palette::Color;
use super::surface::Surface;

pub fn draw_box(surface: &mut Surface, mut x: i32, mut y: i32, mut width: i32, mut height: i32, color: Color, blend_op: SurfaceBlendOps) {

    // Clamp to surface.
    if x < 0 {
        width = width + x;
        x = 0;
    } else if x + width >= surface.width as i32 {
        width = surface.width as i32 - x;
    }
    if width <= 0 {
        return;
    }

    if y < 0 {
        height = height + y;
        y = 0;
    } else if y + height >= surface.height as i32 {
        height = surface.height as i32 - y;
    }
    if height <= 0 {
        return;
    }

    for y in y..y + height {
        let mut dest = (x + (y * surface.width as i32)) as usize * 4;

        for _ in 0..width {
            let dest_color = &mut surface.data[dest..dest + 4];
            dest += 4;

            match blend_op {
                SurfaceBlendOps::Blend => {
                    if color[3] == 0 {
                        continue;
                    } else if color[3] == 255 {
                        dest_color.copy_from_slice(&color);
                    } else {
                        // dest[channel] = (source[alpha] * (source[channel] - dest[channel])) / 255 + dest[channel]
                        dest_color[0] = ((color[3] as i32 * color[0].saturating_sub(dest_color[0]) as i32) / 255 + dest_color[0] as i32) as u8;
                        dest_color[1] = ((color[3] as i32 * color[1].saturating_sub(dest_color[1]) as i32) / 255 + dest_color[1] as i32) as u8;
                        dest_color[2] = ((color[3] as i32 * color[2].saturating_sub(dest_color[2]) as i32) / 255 + dest_color[2] as i32) as u8;
                    }
                },
                SurfaceBlendOps::CopyAlpha => {
                    if color[3] == 0 {
                        continue;
                    }
                    dest_color.copy_from_slice(&color);
                }
                SurfaceBlendOps::Copy => {
                    dest_color.copy_from_slice(&color);
                }
            }
        }
    }
}
