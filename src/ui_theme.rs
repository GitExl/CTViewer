use crate::software_renderer::bitmap::Bitmap;
use crate::software_renderer::blit::{blit_bitmap_to_surface, BitmapBlitFlags, SurfaceBlendOps};
use crate::util::rect::Rect;
use crate::software_renderer::draw::draw_box_gradient_vertical;
use crate::software_renderer::palette::Palette;
use crate::software_renderer::surface::Surface;

pub struct UiTheme {
    pub window_bitmap: Bitmap,
    pub window_palette: Palette,

    pub cursor_bitmap: Bitmap,
    pub cursor_palette: Palette,
}

impl UiTheme {
    pub fn new(window_bitmap: Bitmap, window_palette: Palette, cursor_bitmap: Bitmap, cursor_palette: Palette) -> UiTheme {
        UiTheme {
            window_bitmap,
            window_palette,
            cursor_bitmap,
            cursor_palette,
        }
    }

    pub fn render_window(&self, surface: &mut Surface, x: i32, y: i32, chip_width: i32, chip_height: i32) {
        let pixel_width = chip_width * 8;
        let pixel_height = chip_height * 8;

        // Draw top.
        let dest_y = y;
        for chip_x in 1..chip_width - 1 {
            let dest_x = x + chip_x * 8;
            let ci = (chip_x % 2) * 8;
            blit_bitmap_to_surface(&self.window_bitmap, surface, 8 + ci, 0, 8, 8, dest_x, dest_y, &self.window_palette, 0, BitmapBlitFlags::empty());
        }

        // Draw bottom.
        let dest_y = y + chip_height * 8 - 8;
        for chip_x in 1..chip_width - 1 {
            let dest_x = x + chip_x * 8;
            let ci = (chip_x % 2) * 8;
            blit_bitmap_to_surface(&self.window_bitmap, surface, 8 + ci, 24, 8, 8, dest_x, dest_y, &self.window_palette, 0, BitmapBlitFlags::empty());
        }

        // Draw left side.
        let dest_x = x;
        for chip_y in 1..chip_height - 1 {
            let dest_y = y + chip_y * 8;
            let ci = (chip_y % 2) * 8;
            blit_bitmap_to_surface(&self.window_bitmap, surface, 0, 8 + ci, 8, 8, dest_x, dest_y, &self.window_palette, 0, BitmapBlitFlags::empty());
        }

        // Draw right side.
        let dest_x = x + chip_width * 8 - 8;
        for chip_y in 1..chip_height - 1 {
            let dest_y = y + chip_y * 8;
            let ci = (chip_y % 2) * 8;
            blit_bitmap_to_surface(&self.window_bitmap, surface, 24, 8 + ci, 8, 8, dest_x, dest_y, &self.window_palette, 0, BitmapBlitFlags::empty());
        }

        // Draw corners.
        blit_bitmap_to_surface(&self.window_bitmap, surface, 0, 0, 8, 8, x, y, &self.window_palette, 0, BitmapBlitFlags::empty());
        blit_bitmap_to_surface(&self.window_bitmap, surface, 24, 0, 8, 8, x + pixel_width - 8, y, &self.window_palette, 0, BitmapBlitFlags::empty());

        blit_bitmap_to_surface(&self.window_bitmap, surface, 0, 24, 8, 8, x, y + pixel_height - 8, &self.window_palette, 0, BitmapBlitFlags::empty());
        blit_bitmap_to_surface(&self.window_bitmap, surface, 24, 24, 8, 8, x + pixel_width - 8, y + pixel_height - 8, &self.window_palette, 0, BitmapBlitFlags::empty());

        // Fill inside.
        for chip_y in 1..chip_height - 1 {
            for chip_x in 1..chip_width - 1 {

                // Haha what
                let sy = 32 + ((chip_y - 1) % 2) * 8;
                let sx = if ((chip_y - 1) / 2) % 2 == 0 {
                    ((chip_x - 1) % 4) * 8
                } else {
                    ((chip_x + 1) % 4) * 8
                };

                blit_bitmap_to_surface(&self.window_bitmap, surface, sx, sy, 8, 8, x + chip_x * 8, y + chip_y * 8, &self.window_palette, 0, BitmapBlitFlags::empty());
            }
        }

        draw_box_gradient_vertical(surface, Rect::new(x, y, x + pixel_width, y + pixel_height / 2), [66, 66, 66, 255], [0, 0, 0, 255], 5, SurfaceBlendOps::Add);
        draw_box_gradient_vertical(surface, Rect::new(x, y + pixel_height / 2, x + pixel_width, y + pixel_height), [0, 0, 0, 255], [66, 66, 66, 255], 5, SurfaceBlendOps::Subtract);
    }

    pub fn render_to_surface(&self) -> Surface {
        let mut surface = Surface::new(32, 64);
        surface.fill(self.cursor_palette.colors[0]);

        blit_bitmap_to_surface(&self.window_bitmap, &mut surface, 0, 0, 32, 48, 0, 0, &self.window_palette, 0, BitmapBlitFlags::empty());
        blit_bitmap_to_surface(&self.cursor_bitmap, &mut surface, 0, 0, 32, 16, 0, 48, &self.cursor_palette, 0, BitmapBlitFlags::empty());

        surface
    }
}
