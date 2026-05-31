use std::path::Path;
use crate::filesystem::filesystem::FileSystem;
use crate::software_renderer::bitmap::Bitmap;
use crate::software_renderer::blit::{blit_bitmap_to_surface, BitmapBlitFlags};
use crate::software_renderer::palette::Palette;
use crate::software_renderer::surface::Surface;

pub struct WorldSprites {
    world_index: usize,
    bitmap: Bitmap,
}

impl WorldSprites {

    pub fn new(fs: &FileSystem, world_index: usize, sprite_graphics: [usize; 4]) -> WorldSprites {
        WorldSprites {
            world_index,
            bitmap: fs.read_world_sprite_tiles_all(world_index, sprite_graphics),
        }
    }

    // Replace part of the world sprite tile graphics with new data.
    pub fn replace(&mut self, fs: &FileSystem, tiles_index: usize, offset: usize) {
        fs.read_world_sprite_tiles(self.world_index, tiles_index, offset, &mut self.bitmap.data);
    }

    // Read player character sprites.
    pub fn load_player_sprites(&mut self, fs: &FileSystem, characters: [usize; 3]) {
        let src_pixels = fs.read_world_player_sprite_tiles();

        // Copy player world sprites from external bitmap data into sprite bitmap data.
        // Only 3 characters these are loaded at a time.
        for (index, src_index) in characters.iter().enumerate() {

            // Copy regular sprites to 0x4000 in tile memory + the start of the player character.
            // Each row is 0x80 bytes, 16 rows is 0x800 bytes.
            let src_offset = src_index * 0x800;
            let dest_offset = 0x4000 + (index * 0x800);
            self.bitmap.data[dest_offset..dest_offset + 0x800].copy_from_slice(&src_pixels[src_offset..src_offset + 0x800]);

            // Copy the loose idle sprite.
            for row in 0..16 {
                let src_offset = 0x3800 + src_index * 0x10 + (row * 0x80);
                let dest_offset = 0x5800 + src_index * 0x10 + (row * 0x80);
                self.bitmap.data[dest_offset..dest_offset + 0x10].copy_from_slice(&src_pixels[src_offset..src_offset + 0x10]);
            }
        }
    }

    // Read epoch sprites.
    pub fn load_epoch_sprites(&mut self, fs: &FileSystem, mode: usize) {
        let src_pixels = fs.read_world_epoch_sprite_tiles();

        // Copy walking sprites to 8192 + start of player character.
        let src_offset = mode * 0x800;
        let dest_offset = 0x6000 + (mode * 0x1000);
        self.bitmap.data[dest_offset..dest_offset + 0x1000].copy_from_slice(&src_pixels[src_offset..src_offset + 0x1000]);
    }

    // Dump world sprite tiles to disk.
    pub fn dump(&self, palette: &Palette) {
        let mut surface = Surface::new(128, 256);
        blit_bitmap_to_surface(&self.bitmap, &mut surface, 0, 0, 128, 256, 0, 0, &palette, 0, BitmapBlitFlags::default());
        surface.write_to_bmp(Path::new("debug_output/world_sprite_graphics.bmp"));
    }

}
