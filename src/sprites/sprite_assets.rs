use std::collections::HashMap;
use std::path::Path;
use crate::filesystem::filesystem::FileSystem;
use crate::game_palette::GamePalette;
use crate::software_renderer::bitmap::Bitmap;
use crate::software_renderer::blit::blit_bitmap_to_surface;
use crate::software_renderer::blit::BitmapBlitFlags;
use crate::software_renderer::palette::Palette;
use crate::software_renderer::surface::Surface;
use crate::util::bmp_reader::Bmp;
use super::sprite_anim::{SpriteAnim, SpriteAnimFrame, SpriteAnimSet};
use super::sprite_assembly::{SpriteAssembly, SpriteAssemblyChip, SpriteAssemblyChipFlags, SpriteAssemblyFrame};

// Keys for null sprite.
const NULL_SPRITE_INDEX: usize = 0xFFFF;
const NULL_ANIM_SET_INDEX: usize = 0xFFFF;
pub const NULL_ASSEMBLY_SET_INDEX: usize = 0xFFFF;
pub const NULL_PALETTE_INDEX: usize = 0xFFFF;

pub struct SpriteInfo {
    pub index: usize,
    pub tiles_bitmap_index: usize,
    pub assembly_index: usize,
    pub palette_key: u64,
    pub anim_set_index: usize,
}

pub struct Assets {
    sprite_info: HashMap<usize, SpriteInfo>,
    anim_sets: HashMap<usize, SpriteAnimSet>,
    assembly_frames: HashMap<u64, SpriteAssemblyFrame>,
    assemblies: Vec<SpriteAssembly>,
    palettes: HashMap<u64, Palette>,
    bitmaps: Vec<Bitmap>,

    // null_sprite: SpriteInfo,
}

impl Assets {
    pub fn new(fs: &FileSystem) -> Assets {
        // anim_sets.insert(NULL_ANIM_SET_INDEX, generate_null_sprite_anim_set());

        Assets {
            sprite_info: HashMap::new(),
            anim_sets: fs.read_sprite_animations(),
            assembly_frames: HashMap::new(),
            assemblies: Vec::new(),
            palettes: HashMap::new(),
            bitmaps: Vec::new(),

            // null_sprite: self.generate_null_sprite_asset(),
        }
    }

    pub fn get_sprite_info(&self, sprite_index: usize) -> &SpriteInfo {
        //self.sprite_info.get(&sprite_index).unwrap_or(&self.null_sprite)
        self.sprite_info.get(&sprite_index).unwrap()
    }

    pub fn get_anim_set(&self, anim_set_index: usize) -> &SpriteAnimSet {
        self.anim_sets.get(&anim_set_index).unwrap()
    }

    pub fn get_assembly_frame(&self, assembly_frame_key: u64) -> &SpriteAssemblyFrame {
        self.assembly_frames.get(&assembly_frame_key).unwrap()
    }

    pub fn get_assembly(&self, assembly_index: usize) -> &SpriteAssembly {
        self.assemblies.get(assembly_index).unwrap()
    }

    pub fn get_bitmap(&self, bitmap_index: usize) -> &Bitmap {
        self.bitmaps.get(bitmap_index).unwrap()
    }

    pub fn get_palette(&self, palette_key: u64) -> &Palette {
        self.palettes.get(&palette_key).unwrap()
    }

    pub fn get_palette_mut(&mut self, palette_key: u64) -> &mut Palette {
        self.palettes.get_mut(&palette_key).unwrap()
    }

    // Load a sprite for future use.
    pub fn load_sprite_info(&mut self, fs: &FileSystem, sprite_index: usize) -> &SpriteInfo {
        if !self.sprite_info.contains_key(&sprite_index) {
            let info = fs.read_sprite_header(sprite_index);
            let (assembly, frames) = fs.read_sprite_assembly(info.assembly_index, &info);
            let palette = fs.read_sprite_palette(info.palette_index, 0).unwrap();
            let tiles = fs.read_sprite_tiles(info.bitmap_index, assembly.chip_max);

            self.bitmaps.push(tiles);
            let tiles_bitmap_index = self.bitmaps.len() - 1;

            let palette_key = GamePalette::key_for_sprite_palette(info.palette_index);
            self.palettes.insert(palette_key, palette);

            self.assemblies.push(assembly);
            let assembly_index = self.assemblies.len() - 1;

            self.assembly_frames.extend(frames);

            let sprite = SpriteInfo {
                index: sprite_index,
                tiles_bitmap_index,
                assembly_index,
                palette_key,
                anim_set_index: info.anim_index,
            };
            self.sprite_info.insert(sprite_index, sprite);

        }

        self.sprite_info.get(&sprite_index).unwrap()
    }

    pub fn load_palette(&mut self, fs: &FileSystem, palette_index: usize) -> u64 {
        let palette_key = GamePalette::key_for_sprite_palette(palette_index);

        if !self.palettes.contains_key(&palette_key) {
            let palette = fs.read_sprite_palette(palette_index, 0).unwrap();
            self.palettes.insert(palette_key, palette);
        }

        palette_key
    }

    pub fn dump(&self) {
        for (index, sprite) in self.sprite_info.iter() {
            let tiles = self.bitmaps.get(sprite.tiles_bitmap_index).unwrap();
            let palette = self.palettes.get(&sprite.palette_key).unwrap();

            let mut surface = Surface::new(tiles.width, tiles.height);
            blit_bitmap_to_surface(&tiles, &mut surface, 0, 0, tiles.width as i32, tiles.height as i32, 0, 0, &palette, 0, BitmapBlitFlags::default());
            surface.write_to_bmp(Path::new(&format!("debug_output/sprite_{:03}.bmp", index)));
        }
    }

    fn generate_null_sprite_asset(&mut self) -> SpriteInfo {
        let null_sprite_bmp = Bmp::from_path(Path::new("data/null_sprite.bmp"));

        let null_raw_palette = null_sprite_bmp.get_raw_palette();
        let null_palette = Palette::from_colors(&null_raw_palette);
        let null_palette_key = GamePalette::key_for_sprite_palette(NULL_PALETTE_INDEX);
        self.palettes.insert(null_palette_key, null_palette);

        let null_sprite_bitmap = Bitmap::from_raw_data(null_sprite_bmp.width, null_sprite_bmp.height, null_sprite_bmp.pixels);
        self.bitmaps.push(null_sprite_bitmap);

        let mut null_frames: HashMap<u64, SpriteAssemblyFrame> = HashMap::new();
        let null_frame_key = SpriteAssemblyFrame::key_for_scene_frame(NULL_ASSEMBLY_SET_INDEX, 0);
        null_frames.insert(null_frame_key, SpriteAssemblyFrame {
            chips: vec![
                SpriteAssemblyChip {
                    x: 0,
                    y: 0,
                    width: 32,
                    height: 32,
                    src_index: 0,
                    src_x: 0,
                    src_y: 0,
                    flags: SpriteAssemblyChipFlags::empty(),
                }
            ],
        });
        self.assembly_frames.extend(null_frames);

        let null_assembly = SpriteAssembly {
            index: NULL_ASSEMBLY_SET_INDEX,
            frame_keys: vec![null_frame_key],
            chip_max: 1,
        };
        self.assemblies.push(null_assembly);

        SpriteInfo {
            index: NULL_SPRITE_INDEX,
            anim_set_index: NULL_ANIM_SET_INDEX,
            tiles_bitmap_index: self.bitmaps.len() - 1,
            palette_key: null_palette_key,
            assembly_index: self.assemblies.len() - 1,
        }
    }

}

fn generate_null_sprite_anim_set() -> SpriteAnimSet {
    let mut set = SpriteAnimSet::new(NULL_ANIM_SET_INDEX);
    set.add_anim(SpriteAnim {
        frames: vec![
            SpriteAnimFrame {
                sprite_frames: [0, 0, 0, 0],
                delay: 60,
            }
        ],
    });
    
    set
}
