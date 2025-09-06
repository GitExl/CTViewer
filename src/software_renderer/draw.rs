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
            blend_pixel(dest_color, color, blend_op);
        }
    }
}

pub fn draw_line(surface: &mut Surface, x1: i32, y1: i32, x2: i32, y2: i32, color: Color, blend_op: SurfaceBlendOps) {
    let clip = surface.clip;

    // Cohen-Sutherland line clipping.
    const INSIDE: u8 = 0;
    const LEFT: u8 = 1;
    const RIGHT: u8 = 2;
    const BOTTOM: u8 = 4;
    const TOP: u8 = 8;

    fn compute_outcode(x: i32, y: i32, clip: &Rect) -> u8 {
        let mut code = INSIDE;
        if x < clip.left {
            code |= LEFT;
        } else if x >= clip.right {
            code |= RIGHT;
        }
        if y < clip.top {
            code |= TOP;
        } else if y >= clip.bottom {
            code |= BOTTOM;
        }
        code
    }

    let mut x1 = x1;
    let mut y1 = y1;
    let mut x2 = x2;
    let mut y2 = y2;
    
    let mut outcode1 = compute_outcode(x1, y1, &clip);
    let mut outcode2 = compute_outcode(x2, y2, &clip);
    
    loop {
        // Both points are inside the clip rectangle.
        if (outcode1 | outcode2) == 0 {
            break;
        // Both points share an outside zone the line is completely outside.
        } else if (outcode1 & outcode2) != 0 {
            return;
        }

        // The line needs clipping.
        let outcode_out = if outcode1 != 0 { outcode1 } else { outcode2 };

        let x: i32;
        let y: i32;
        if (outcode_out & TOP) != 0 {
            x = x1 + (x2 - x1) * (clip.top - y1) / (y2 - y1);
            y = clip.top;
        } else if (outcode_out & BOTTOM) != 0 {
            x = x1 + (x2 - x1) * (clip.bottom - 1 - y1) / (y2 - y1);
            y = clip.bottom - 1;
        } else if (outcode_out & RIGHT) != 0 {
            y = y1 + (y2 - y1) * (clip.right - 1 - x1) / (x2 - x1);
            x = clip.right - 1;
        } else {
            y = y1 + (y2 - y1) * (clip.left - x1) / (x2 - x1);
            x = clip.left;
        }

        if outcode_out == outcode1 {
            x1 = x;
            y1 = y;
            outcode1 = compute_outcode(x1, y1, &clip);
        } else {
            x2 = x;
            y2 = y;
            outcode2 = compute_outcode(x2, y2, &clip);
        }
    }

    // Bresenham line drawing.
    let dx = (x2 - x1).abs();
    let dy = (y2 - y1).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let sy = if y1 < y2 { 1 } else { -1 };
    let mut err = dx - dy;

    let mut x = x1;
    let mut y = y1;
    loop {
        let dest = (x + y * surface.width as i32) as usize * 4;
        let dest_color = &mut surface.data[dest..dest + 4];
        blend_pixel(dest_color, color, blend_op);
        
        if x == x2 && y == y2 {
            break;
        }
        
        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
}

fn blend_pixel (dest_color: &mut [u8], color: Color, blend_op: SurfaceBlendOps) {
    match blend_op {
        SurfaceBlendOps::Blend => {
            if color[3] == 0 {
                return;
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
                return;
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
