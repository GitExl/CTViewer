use crate::software_renderer::clip::Rect;
use super::blit::SurfaceBlendOps;
use super::palette::Color;
use super::surface::Surface;

pub fn draw_box(surface: &mut Surface, rect: Rect, color: Color, blend_op: SurfaceBlendOps) {
    let draw = rect.clip_to(&surface.clip);

    for y in draw.top..draw.bottom {
        let mut dest = (draw.left + (y * surface.width as i32)) as usize * 4;

        for _ in draw.left..draw.right {
            let dest_color = &mut surface.data[dest..dest + 4];
            dest += 4;

            match blend_op {
                SurfaceBlendOps::Blend => {
                    if color[3] == 0 {
                        continue;
                    } else if color[3] == 255 {
                        dest_color[0] = color[0];
                        dest_color[1] = color[1];
                        dest_color[2] = color[2];
                        dest_color[3] = 0xFF;
                    } else {
                        dest_color[0] = ((color[3] as i32 * color[0] as i32 + (255 - color[3] as i32) * dest_color[0] as i32 + 127) / 255) as u8;
                        dest_color[1] = ((color[3] as i32 * color[1] as i32 + (255 - color[3] as i32) * dest_color[1] as i32 + 127) / 255) as u8;
                        dest_color[2] = ((color[3] as i32 * color[2] as i32 + (255 - color[3] as i32) * dest_color[2] as i32 + 127) / 255) as u8;
                        dest_color[3] = 0xFF;
                    }
                },
                SurfaceBlendOps::CopyAlpha => {
                    if color[3] == 0 {
                        continue;
                    }
                    dest_color[0] = color[0];
                    dest_color[1] = color[1];
                    dest_color[2] = color[2];
                    dest_color[3] = 0xFF;
                }
                SurfaceBlendOps::Copy => {
                    dest_color[0] = color[0];
                    dest_color[1] = color[1];
                    dest_color[2] = color[2];
                    dest_color[3] = 0xFF;
                },
                SurfaceBlendOps::CopyAlphaGreyscale => {
                    dest_color[0] = color[3];
                    dest_color[1] = color[3];
                    dest_color[2] = color[3];
                    dest_color[3] = 0xFF;
                }
            }
        }
    }
}
