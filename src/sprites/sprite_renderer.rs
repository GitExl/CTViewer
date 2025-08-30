use crate::software_renderer::bitmap::Bitmap;
use crate::software_renderer::blit::blit_bitmap_to_surface_and_source;
use crate::software_renderer::blit::BitmapBlitFlags;
use crate::software_renderer::surface::Surface;

use super::sprite_assembly::SpriteAssemblyChipFlags;
use super::sprite_list::Sprite;

pub fn render_sprite(surface: &mut Surface, pixel_source: &mut Bitmap, source_value: u8, sprite: &Sprite, frame: usize, x: i32, y: i32, palette_offset: usize) {
    let frame = &sprite.assembly.frames[frame];

    for tile in frame.chips.iter().rev() {
        if tile.flags.contains(SpriteAssemblyChipFlags::UNUSED) {
            continue;
        }
        if tile.src_x >= sprite.tiles.width as i32 || tile.src_y >= sprite.tiles.height as i32 {
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
            &sprite.tiles,
            surface,
            pixel_source,
            tile.src_x, tile.src_y,
            tile.width, tile.height,
            x + tile.x, y + tile.y,
            &sprite.palette, palette_offset,
            source_value,
            flags,
        );
    }
}
