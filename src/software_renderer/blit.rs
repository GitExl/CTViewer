use bitflags::bitflags;

use super::bitmap::Bitmap;
use super::palette::Palette;
use super::surface::Surface;

bitflags! {
    #[derive(Clone, Default, Copy)]
    pub struct BitmapBlitFlags: u8 {
        const FLIP_X = 0x01;
        const FLIP_Y = 0x02;
        const SKIP_0 = 0x04;
    }
}

#[derive(Clone, Copy)]
pub enum SurfaceBlendOps {
    Copy,
    CopyAlpha,
    Blend,
    CopyAlphaGreyscale,
}

pub fn blit_bitmap_to_surface_and_source(bitmap: &Bitmap, surface: &mut Surface, dest_bitmap: &mut Bitmap, src_x: i32, src_y: i32, src_width: i32, src_height: i32, dest_x: i32, dest_y: i32, palette: &Palette, color_offset: usize, source_value: u8, flags: BitmapBlitFlags) {
    let delta_x;
    let start_cx;
    if flags.contains(BitmapBlitFlags::FLIP_X) {
        delta_x = -1;
        start_cx = src_width - 1;
    } else {
        delta_x = 1;
        start_cx = 0;
    }

    let delta_y;
    let mut cy;
    if flags.contains(BitmapBlitFlags::FLIP_Y) {
        delta_y = -1;
        cy = src_height - 1;
    } else {
        delta_y = 1;
        cy = 0;
    }

    let skip_0 = flags.contains(BitmapBlitFlags::SKIP_0);
    let surface_data = surface.data.as_mut_ptr();
    let dest_bitmap_data = dest_bitmap.data.as_mut_ptr();

    for y in dest_y..dest_y + src_height {
        if !(y < surface.clip.top || y >= surface.clip.bottom) {

            let mut cx = start_cx;
            let mut src = src_x + cx + ((src_y + cy) * bitmap.width as i32);
            let mut dest = (dest_x + (y * surface.width as i32)) * 4;
            let mut dest_source = dest_x + (y * dest_bitmap.width as i32);

            for x in dest_x..dest_x + src_width {
                if !(x < surface.clip.left || x >= surface.clip.right) {

                    let src_color = bitmap.data[src as usize] as usize;
                    if src_color != 0 || !skip_0 {
                        let col = palette.colors[src_color + color_offset];
                        unsafe {
                            let dest_ptr = surface_data.add(dest as usize);
                            std::ptr::copy_nonoverlapping(col.as_ptr(), dest_ptr, 4);
                            *dest_bitmap_data.add(dest_source as usize) = source_value;
                        }
                        // surface.data[dest as usize..dest as usize + 4].copy_from_slice(&col);
                        // dest_bitmap.data[dest_source as usize] = source_value;
                    }
                }

                src += delta_x;
                dest += 4;
                dest_source += 1;
                cx += delta_x;
            }
        }

        cy += delta_y;
    }
}

pub fn blit_bitmap_to_surface(bitmap: &Bitmap, surface: &mut Surface, src_x: i32, src_y: i32, src_width: i32, src_height: i32, dest_x: i32, dest_y: i32, palette: &Palette, color_offset: usize, flags: BitmapBlitFlags) {
    let delta_x;
    let start_cx;
    if flags.contains(BitmapBlitFlags::FLIP_X) {
        delta_x = -1;
        start_cx = src_width - 1;
    } else {
        delta_x = 1;
        start_cx = 0;
    }

    let delta_y;
    let mut cy;
    if flags.contains(BitmapBlitFlags::FLIP_Y) {
        delta_y = -1;
        cy = src_height - 1;
    } else {
        delta_y = 1;
        cy = 0;
    }

    let skip_0 = flags.contains(BitmapBlitFlags::SKIP_0);

    for y in dest_y..dest_y + src_height {
        if !(y < surface.clip.top || y >= surface.clip.bottom) {

            let mut cx = start_cx;
            let mut src = src_x + cx + ((src_y + cy) * bitmap.width as i32);
            let mut dest = (dest_x + (y * surface.width as i32)) * 4;

            for x in dest_x..dest_x + src_width {
                if !(x < surface.clip.left || x >= surface.clip.right) {
                    let src_color = bitmap.data[src as usize] as usize;
                    if src_color != 0 || !skip_0 {
                        let col = palette.colors[src_color + color_offset];
                        surface.data[dest as usize..dest as usize + 4].copy_from_slice(&col);
                    }
                }

                src += delta_x;
                dest += 4;
                cx += delta_x;
            }
        }

        cy += delta_y;
    }
}

pub fn blit_surface_to_surface(src_surface: &Surface, dest_surface: &mut Surface, mut src_x: i32, mut src_y: i32, mut width: i32, mut height: i32, mut dest_x: i32, mut dest_y: i32, blend_op: SurfaceBlendOps) {

    // Clamp to surface.
    if dest_x < dest_surface.clip.left {
        width = width + dest_x;
        src_x += -dest_x;
        dest_x = dest_surface.clip.left;
    } else if dest_x + width >= dest_surface.clip.right {
        width = dest_surface.clip.right - dest_x;
    }
    if width <= 0 {
        return;
    }

    if dest_y < dest_surface.clip.top {
        height = height + dest_y;
        src_y += -dest_y;
        dest_y = dest_surface.clip.top;
    } else if dest_y + height >= dest_surface.clip.bottom {
        height = dest_surface.clip.bottom - dest_y;
    }
    if height <= 0 {
        return;
    }

    for y in 0..height {
        let mut src = (src_x + ((src_y + y) * src_surface.width as i32)) as usize * 4;
        let mut dest = (dest_x + ((dest_y + y) * dest_surface.width as i32)) as usize * 4;

        for _ in 0..width {
            let src_color = &src_surface.data[src..src + 4];
            let dest_color = &mut dest_surface.data[dest..dest + 4];
            src += 4;
            dest += 4;

            match blend_op {
                SurfaceBlendOps::Blend => {
                    if src_color[3] == 0 {
                        continue;
                    } else if src_color[3] == 255 {
                        dest_color[0] = src_color[0];
                        dest_color[1] = src_color[1];
                        dest_color[2] = src_color[2];
                        dest_color[3] = 0xFF;
                    } else {
                        dest_color[0] = ((src_color[3] as i32 * src_color[0] as i32 + (255 - src_color[3] as i32) * dest_color[0] as i32 + 127) / 255) as u8;
                        dest_color[1] = ((src_color[3] as i32 * src_color[1] as i32 + (255 - src_color[3] as i32) * dest_color[1] as i32 + 127) / 255) as u8;
                        dest_color[2] = ((src_color[3] as i32 * src_color[2] as i32 + (255 - src_color[3] as i32) * dest_color[2] as i32 + 127) / 255) as u8;
                        dest_color[3] = 0xFF;
                    }
                },
                SurfaceBlendOps::CopyAlpha => {
                    if src_color[3] == 0 {
                        continue;
                    }
                    dest_color[0] = src_color[0];
                    dest_color[1] = src_color[1];
                    dest_color[2] = src_color[2];
                    dest_color[3] = 0xFF;
                }
                SurfaceBlendOps::Copy => {
                    dest_color[0] = src_color[0];
                    dest_color[1] = src_color[1];
                    dest_color[2] = src_color[2];
                    dest_color[3] = 0xFF;
                },
                SurfaceBlendOps::CopyAlphaGreyscale => {
                    dest_color[0] = src_color[3];
                    dest_color[1] = src_color[3];
                    dest_color[2] = src_color[3];
                    dest_color[3] = 0xFF;
                }
            }
        }
    }
}
