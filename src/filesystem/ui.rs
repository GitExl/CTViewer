use crate::filesystem::filesystem::{FileSystem, ParseMode};
use crate::ui_theme::UiTheme;

impl FileSystem {
    pub fn read_ui_theme(&self, ui_theme_index: usize) -> UiTheme {
        let ui_theme_index = match self.parse_mode {
            ParseMode::Snes => ui_theme_index,
            ParseMode::Pc => 0,
        };

        let (window_bitmap, window_palette) = self.backend.get_ui_theme_window_graphics(ui_theme_index);
        let (cursor_bitmap, cursor_palette) = self.backend.get_ui_theme_cursor_graphics(ui_theme_index);

        UiTheme::new(window_bitmap, window_palette, cursor_bitmap, cursor_palette)
    }
}
