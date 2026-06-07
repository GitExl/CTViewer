use std::collections::HashMap;
use std::path::Path;
use crate::filesystem::filesystem::FileSystem;
use crate::software_renderer::bitmap::Bitmap;
use crate::software_renderer::blit::blit_bitmap_to_surface;
use crate::software_renderer::blit::BitmapBlitFlags;
use crate::software_renderer::palette::Palette;
use crate::software_renderer::surface::Surface;
use crate::sprites::sprite_anim::SpriteAnimSet;
use crate::sprites::sprite_assembly::{SpriteAssembly, SpriteAssemblyFrame};

pub struct SpriteInfo {
    pub index: usize,
    pub tiles_bitmap_key: u64,
    pub assembly_key: u64,
    pub palette_key: u64,
    pub anim_set_index: usize,
}

pub struct Assets {
    sprite_info: HashMap<u64, SpriteInfo>,
    assembly_frames: HashMap<u64, SpriteAssemblyFrame>,
    assemblies: HashMap<u64, SpriteAssembly>,
    palettes: HashMap<u64, Palette>,
    bitmaps: HashMap<u64, Bitmap>,

    anim_sets: HashMap<usize, SpriteAnimSet>,
}

impl Assets {
    pub fn new(fs: &FileSystem) -> Assets {

        Assets {
            sprite_info: HashMap::new(),
            assembly_frames: HashMap::new(),
            assemblies: HashMap::new(),
            palettes: HashMap::new(),
            bitmaps: HashMap::new(),

            anim_sets: fs.read_sprite_animations(),
        }
    }

    pub fn get_sprite_info(&self, sprite_info_key: u64) -> &SpriteInfo {
        self.sprite_info.get(&sprite_info_key).unwrap()
    }

    pub fn get_anim_set(&self, anim_set_index: usize) -> &SpriteAnimSet {
        self.anim_sets.get(&anim_set_index).unwrap()
    }

    pub fn get_assembly_frame(&self, assembly_frame_key: u64) -> &SpriteAssemblyFrame {
        self.assembly_frames.get(&assembly_frame_key).unwrap()
    }

    pub fn get_assembly(&self, assembly_key: u64) -> &SpriteAssembly {
        self.assemblies.get(&assembly_key).unwrap()
    }

    pub fn get_bitmap(&self, bitmap_key: u64) -> &Bitmap {
        self.bitmaps.get(&bitmap_key).unwrap()
    }

    pub fn get_palette(&self, palette_key: u64) -> &Palette {
        self.palettes.get(&palette_key).unwrap()
    }

    pub fn get_palette_mut(&mut self, palette_key: u64) -> &mut Palette {
        self.palettes.get_mut(&palette_key).unwrap()
    }

    pub fn load_sprite_assembly(&mut self, fs: &FileSystem, assembly_index: usize, size_flags: u32) -> u64 {
        let assembly_key = Assets::asset_key_sprite_assembly(assembly_index);
        if self.assemblies.contains_key(&assembly_key) {
            return assembly_key;
        }

        let (assembly, frames) = fs.read_sprite_assembly(assembly_index, size_flags);
        self.assembly_frames.extend(frames);
        self.assemblies.insert(assembly_key, assembly);

        assembly_key
    }

    pub fn load_sprite_palette(&mut self, fs: &FileSystem, palette_index: usize) -> u64 {
        let palette_key = Assets::asset_key_palette_sprite_scene(palette_index);
        if self.palettes.contains_key(&palette_key) {
            return palette_key;
        }

        let palette = fs.read_sprite_palette(palette_index, 0).unwrap();
        self.palettes.insert(palette_key, palette);

        palette_key
    }

    pub fn load_sprite_tiles(&mut self, fs: &FileSystem, sprite_index: usize, max_chip_count: usize) -> u64 {
        let bitmap_key = Assets::asset_key_bitmap_sprite_tiles(sprite_index);
        if self.bitmaps.contains_key(&bitmap_key) {
            return bitmap_key;
        }

        let bitmap = fs.read_sprite_tiles(sprite_index, max_chip_count);
        self.bitmaps.insert(bitmap_key, bitmap);

        bitmap_key
    }

    pub fn load_sprite_info(&mut self, fs: &FileSystem, sprite_index: usize) -> u64 {
        let sprite_info_key = Assets::asset_key_sprite_info(sprite_index);
        if self.sprite_info.contains_key(&sprite_info_key) {
            return sprite_info_key;
        }

        let info = fs.read_sprite_header(sprite_index);
        let assembly_key = self.load_sprite_assembly(&fs, info.assembly_index, info.size_flags);
        let assembly = self.get_assembly(assembly_key);

        let sprite = SpriteInfo {
            index: sprite_index,
            tiles_bitmap_key: self.load_sprite_tiles(&fs, info.bitmap_index, assembly.chip_max),
            assembly_key,
            palette_key: self.load_sprite_palette(&fs, info.palette_index),
            anim_set_index: info.anim_index,
        };
        self.sprite_info.insert(sprite_info_key, sprite);

        sprite_info_key
    }

    pub fn dump(&self) {
        for (index, sprite) in self.sprite_info.iter() {
            let tiles = self.bitmaps.get(&sprite.tiles_bitmap_key).unwrap();
            let palette = self.palettes.get(&sprite.palette_key).unwrap();

            let mut surface = Surface::new(tiles.width, tiles.height);
            blit_bitmap_to_surface(&tiles, &mut surface, 0, 0, tiles.width as i32, tiles.height as i32, 0, 0, &palette, 0, BitmapBlitFlags::default());
            surface.write_to_bmp(Path::new(&format!("debug_output/sprite_{:03}.bmp", index)));
        }
    }

    pub fn asset_key_sprite_assembly_frame_scene(assembly_index: usize, frame_index: usize) -> u64 {
        0x1000000000000000 | (frame_index as u64) | ((assembly_index as u64) << 32)
    }

    pub fn asset_key_sprite_assembly_frame_world(frame_address: u64) -> u64 {
        0x2000000000000000 | frame_address
    }

    pub fn asset_key_palette_sprite_scene(palette_index: usize) -> u64 {
        0x1000000000000000 | (palette_index as u64)
    }

    pub fn asset_key_sprite_assembly(assembly: usize) -> u64 {
        0x1000000000000000 | (assembly as u64)
    }

    pub fn asset_key_sprite_info(sprite_index: usize) -> u64 {
        0x1000000000000000 | (sprite_index as u64)
    }

    pub fn asset_key_bitmap_sprite_tiles(sprite_index: usize) -> u64 {
        0x1000000000000000 | (sprite_index as u64)
    }
}
