use std::collections::HashMap;
use std::path::Path;
use crate::filesystem::filesystem::FileSystem;
use crate::software_renderer::bitmap::Bitmap;
use crate::software_renderer::blit::blit_bitmap_to_surface;
use crate::software_renderer::blit::BitmapBlitFlags;
use crate::software_renderer::palette::Palette;
use crate::software_renderer::surface::Surface;
use super::sprite_anim::{SpriteAnimFrame, SpriteAnimSet};
use super::sprite_assembly::SpriteAssembly;

// Keys for world sprite data.
pub const WORLD_SPRITE_INDEX: usize = 0xFFFFFF;
pub const WORLD_ANIM_SET_INDEX: usize = 0xFFFFFF;
pub const WORLD_ASSEMBLY_SET_INDEX: usize = 0xFFFFFF;

pub struct SpriteAsset {
    pub index: usize,
    pub tiles: Bitmap,
    pub assembly: SpriteAssembly,
    pub palette: Palette,
    pub anim_set_index: usize,
}

pub struct SpriteAssets {
    assets: HashMap<usize, SpriteAsset>,
    anim_sets: HashMap<usize, SpriteAnimSet>,
}

impl SpriteAssets {
    pub fn new(fs: &FileSystem) -> SpriteAssets {
        SpriteAssets {
            assets: HashMap::new(),
            anim_sets: fs.read_sprite_animations(),
        }
    }

    pub fn get(&self, sprite_index: usize) -> &SpriteAsset {
        self.assets.get(&sprite_index).unwrap()
    }

    pub fn get_anim_set(&self, anim_set_index: usize) -> &SpriteAnimSet {
        self.anim_sets.get(&anim_set_index).unwrap()
    }

    pub fn get_frame_for_animation(&self, sprite_index: usize, anim_index: usize, frame_index: usize) -> (SpriteAnimFrame, usize, usize) {
        let sprite = self.assets.get(&sprite_index).unwrap();
        let anim_set = self.anim_sets.get(&sprite.anim_set_index).unwrap();

        let real_anim_index = if anim_set.anims.len() <= anim_index {
            println!("Warning: sprite {} does not have animation {}. Using animation 0.", sprite_index, anim_index);
            0
        } else {
            anim_index
        };

        let anim = &anim_set.anims[real_anim_index];
        let real_frame_index = if anim.frames.len() == 0 {
            println!("Warning: sprite {} animation {} does not have frame {}. Using frame 0.", sprite_index, anim_index, frame_index);
            0
        } else {
            frame_index
        };

        (anim.frames[real_frame_index], real_anim_index, real_frame_index)
    }

    // Load a sprite for future use.
    pub fn load(&mut self, fs: &FileSystem, sprite_index: usize) {
        if self.assets.contains_key(&sprite_index) {
            return;
        }

        let info = fs.read_sprite_header(sprite_index);
        let assembly = fs.read_sprite_assembly(info.assembly_index, &info);
        let palette = fs.read_sprite_palette(info.palette_index).unwrap();
        let tiles = fs.read_sprite_tiles(info.bitmap_index, assembly.chip_max);

        let sprite = SpriteAsset {
            index: sprite_index,
            tiles,
            assembly,
            palette,
            anim_set_index: info.anim_index,
        };
        self.assets.insert(sprite_index, sprite);
    }

    // Loads the generic world sprite used by world maps.
    pub fn load_world_sprite_asset(&mut self, fs: &FileSystem, world_index: usize, sprite_graphics: [usize; 4], palette: &Palette) {
        if self.assets.contains_key(&WORLD_SPRITE_INDEX) {
            return;
        }

        // Read animation, assembly and graphics data.
        let (assembly, anim_set) = fs.read_world_sprites();
        self.anim_sets.insert(WORLD_ANIM_SET_INDEX, anim_set);
        let tiles = fs.read_world_sprite_tiles_all(world_index, sprite_graphics);

        self.assets.insert(WORLD_SPRITE_INDEX, SpriteAsset {
            index: WORLD_SPRITE_INDEX,
            tiles,
            assembly,
            palette: palette.clone(),
            anim_set_index: WORLD_ANIM_SET_INDEX,
        });
    }

    // Replace part of the world sprite tile graphics with new data.
    pub fn replace_world_sprite_tiles(&mut self, fs: &FileSystem, world_index: usize, tiles_index: usize, offset: usize) {
        let sprite = self.assets.get_mut(&WORLD_SPRITE_INDEX).unwrap();
        fs.read_world_sprite_tiles(world_index, tiles_index, offset, &mut sprite.tiles.data);
    }

    // Read player character sprites.
    pub fn load_world_player_sprites_asset(&mut self, fs: &FileSystem, characters: [usize; 3]) {
        let sprite = self.assets.get_mut(&WORLD_SPRITE_INDEX).unwrap();
        let src_pixels = fs.read_world_player_sprite_tiles();
        let dest_pixels = &mut sprite.tiles.data;

        // Copy player world sprites from external bitmap data into sprite bitmap data.
        // Only 3 characters these are loaded at a time.
        for (index, src_index) in characters.iter().enumerate() {

            // Copy regular sprites to 0x4000 in tile memory + the start of the player character.
            // Each row is 0x80 bytes, 16 rows is 0x800 bytes.
            let src_offset = src_index * 0x800;
            let dest_offset = 0x4000 + (index * 0x800);
            dest_pixels[dest_offset..dest_offset + 0x800].copy_from_slice(&src_pixels[src_offset..src_offset + 0x800]);

            // Copy the loose idle sprite.
            for row in 0..16 {
                let src_offset = 0x3800 + src_index * 0x10 + (row * 0x80);
                let dest_offset = 0x5800 + src_index * 0x10 + (row * 0x80);
                dest_pixels[dest_offset..dest_offset + 0x10].copy_from_slice(&src_pixels[src_offset..src_offset + 0x10]);
            }
        }
    }

    // Read epoch sprites.
    pub fn load_world_epoch_sprites_asset(&mut self, fs: &FileSystem, mode: usize) {
        let sprite = self.assets.get_mut(&WORLD_SPRITE_INDEX).unwrap();
        let src_pixels = fs.read_world_epoch_sprite_tiles();
        let dest_pixels = &mut sprite.tiles.data;

        // Copy walking sprites to 8192 + start of player character.
        let src_offset = mode * 0x800;
        let dest_offset = 0x6000 + (mode * 0x1000);
        dest_pixels[dest_offset..dest_offset + 0x1000].copy_from_slice(&src_pixels[src_offset..src_offset + 0x1000]);
    }

    // Dump world sprite tiles to disk.
    pub fn dump_world_sprite_graphics(&self) {
        let sprite = self.assets.get(&WORLD_SPRITE_INDEX).unwrap();
        let mut surface = Surface::new(128, 256);
        blit_bitmap_to_surface(&sprite.tiles, &mut surface, 0, 0, 128, 256, 0, 0, &sprite.palette, 0, BitmapBlitFlags::default());
        surface.write_to_bmp(Path::new("debug_output/world_sprite_graphics.bmp"));
    }

    pub fn dump(&self) {
        for (index, sprite) in self.assets.iter() {
            let mut surface = Surface::new(sprite.tiles.width, sprite.tiles.height);
            blit_bitmap_to_surface(&sprite.tiles, &mut surface, 0, 0, sprite.tiles.width as i32, sprite.tiles.height as i32, 0, 0, &sprite.palette, 0, BitmapBlitFlags::default());
            surface.write_to_bmp(Path::new(&format!("debug_output/sprite_{:03}.bmp", index)));
        }
    }
}
