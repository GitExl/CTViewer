use crate::filesystem::filesystem::FileSystem;
use crate::software_renderer::bitmap::Bitmap;
use crate::software_renderer::blit::{blit_bitmap_to_bitmap, BitmapBlitFlags};
use crate::ui_theme::UiTheme;

impl FileSystem {
    pub fn read_ui_theme(&self, ui_theme_index: usize) -> UiTheme {
        if ui_theme_index > 7 {
            panic!("UI theme index must be from 0 to 7.");
        }

        let (ui_src_bitmap, ui_palette) = self.backend.get_ui_theme_window_graphics(ui_theme_index);
        let (cursor_src_bitmap, cursor_palette) = self.backend.get_ui_theme_cursor_graphics();

        let mut window_bitmap = Bitmap::new(96, 128);

        // Top
        blit_bitmap_to_bitmap(&ui_src_bitmap, &mut window_bitmap, 0, 0, 32, 8, 0, 0, BitmapBlitFlags::empty());

        // Sides
        blit_bitmap_to_bitmap(&ui_src_bitmap, &mut window_bitmap, 32, 0, 8, 8, 0, 8, BitmapBlitFlags::empty());
        blit_bitmap_to_bitmap(&ui_src_bitmap, &mut window_bitmap, 48, 0, 8, 8, 0, 16, BitmapBlitFlags::empty());

        blit_bitmap_to_bitmap(&ui_src_bitmap, &mut window_bitmap, 40, 0, 8, 8, 24, 8, BitmapBlitFlags::empty());
        blit_bitmap_to_bitmap(&ui_src_bitmap, &mut window_bitmap, 56, 0, 8, 8, 24, 16, BitmapBlitFlags::empty());

        // Bottom
        blit_bitmap_to_bitmap(&ui_src_bitmap, &mut window_bitmap, 0, 8, 32, 8, 0, 24, BitmapBlitFlags::empty());

        // Fill
        blit_bitmap_to_bitmap(&ui_src_bitmap, &mut window_bitmap, 32, 8, 16, 8, 0, 32, BitmapBlitFlags::empty());
        blit_bitmap_to_bitmap(&ui_src_bitmap, &mut window_bitmap, 48, 8, 16, 8, 0, 40, BitmapBlitFlags::empty());
        blit_bitmap_to_bitmap(&ui_src_bitmap, &mut window_bitmap, 0, 16, 16, 8, 16, 32, BitmapBlitFlags::empty());
        blit_bitmap_to_bitmap(&ui_src_bitmap, &mut window_bitmap, 16, 16, 16, 8, 16, 40, BitmapBlitFlags::empty());

        let mut cursor_bitmap = Bitmap::new(32, 16);

        // Hand
        blit_bitmap_to_bitmap(&cursor_src_bitmap, &mut cursor_bitmap, 0, 0, 16, 8, 0, 0, BitmapBlitFlags::empty());
        blit_bitmap_to_bitmap(&cursor_src_bitmap, &mut cursor_bitmap, 16, 0, 16, 8, 0, 8, BitmapBlitFlags::empty());

        // Arrow
        blit_bitmap_to_bitmap(&cursor_src_bitmap, &mut cursor_bitmap, 32, 0, 16, 8, 16, 0, BitmapBlitFlags::empty());
        blit_bitmap_to_bitmap(&cursor_src_bitmap, &mut cursor_bitmap, 48, 0, 16, 8, 16, 8, BitmapBlitFlags::empty());

        UiTheme::new(window_bitmap, ui_palette, cursor_bitmap, cursor_palette)
    }
}
