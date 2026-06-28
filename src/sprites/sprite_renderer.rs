use crate::software_renderer::bitmap::Bitmap;
use crate::software_renderer::blit::blit_bitmap_to_surface_and_source;
use crate::software_renderer::blit::BitmapBlitFlags;
use crate::software_renderer::palette::Palette;
use crate::software_renderer::surface::Surface;
use super::sprite_assembly::{SpriteAssemblyChipFlags, SpriteAssemblyFrame};

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub enum SpritePriority {
    BelowAll,
    BelowL1L2,
    #[default]
    BelowL2AboveL1,
    AboveAll,
}

impl SpritePriority {
    pub fn from_value(value: u8) -> SpritePriority {
        match value {
            0 => SpritePriority::BelowAll,
            1 => SpritePriority::BelowL1L2,
            2 => SpritePriority::BelowL2AboveL1,
            3 => SpritePriority::AboveAll,
            _ => SpritePriority::AboveAll,
        }
    }
}

pub fn render_sprite(surface: &mut Surface, pixel_source: &mut Bitmap, source_value: u8, render_top: bool, render_bottom: bool, assembly_frame: &SpriteAssemblyFrame, bitmap: &Bitmap, tile_offset_x: i32, tile_offset_y: i32, x: i32, y: i32, palette: &Palette, palette_offset: usize) {
    for tile in assembly_frame.chips.iter().rev() {
        if tile.flags.contains(SpriteAssemblyChipFlags::UNUSED) {
            continue;
        }
        if tile.flags.contains(SpriteAssemblyChipFlags::IS_TOP) && !render_top {
            continue;
        }
        if tile.flags.contains(SpriteAssemblyChipFlags::IS_BOTTOM) && !render_bottom {
            continue;
        }

        let src_x = tile.src_x + tile_offset_x;
        let src_y = tile.src_y + tile_offset_y;
        if src_x >= bitmap.width as i32 || src_y >= bitmap.height as i32 {
            continue;
        }

        let mut flags: BitmapBlitFlags = BitmapBlitFlags::SKIP_0;
        if tile.flags.contains(SpriteAssemblyChipFlags::FLIP_X) {
            flags |= BitmapBlitFlags::FLIP_X;
        }
        if tile.flags.contains(SpriteAssemblyChipFlags::FLIP_Y) {
            flags |= BitmapBlitFlags::FLIP_Y;
        }

        blit_bitmap_to_surface_and_source(
            &bitmap,
            surface,
            pixel_source,
            src_x, src_y,
            tile.width, tile.height,
            x + tile.x, y + tile.y,
            palette, palette_offset,
            source_value,
            flags,
        );
    }
}
