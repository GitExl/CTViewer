use std::collections::HashMap;
use std::path::Path;
use crate::filesystem::filesystem::FileSystem;
use crate::software_renderer::bitmap::Bitmap;
use crate::software_renderer::blit::blit_bitmap_to_surface;
use crate::software_renderer::blit::BitmapBlitFlags;
use crate::software_renderer::palette::Palette;
use crate::software_renderer::surface::Surface;
use crate::util::bmp_reader::Bmp;
use super::sprite_anim::{SpriteAnim, SpriteAnimFrame, SpriteAnimSet};
use super::sprite_assembly::{SpriteAssembly, SpriteAssemblyChip, SpriteAssemblyChipFlags, SpriteAssemblyFrame};

// Keys for null sprite.
const NULL_SPRITE_INDEX: usize = 0xFFFFFFFF;
const NULL_ANIM_SET_INDEX: usize = 0xFFFFFFFF;
pub const NULL_ASSEMBLY_SET_INDEX: usize = 0xFFFFFFFF;

pub struct SpriteAsset {
    pub index: usize,
    pub tiles: Bitmap,
    pub assembly: SpriteAssembly,
    pub palette: Palette,
    pub palette_index: usize,
    pub anim_set_index: usize,
}

pub struct SpriteAssets {
    assets: HashMap<usize, SpriteAsset>,
    anim_sets: HashMap<usize, SpriteAnimSet>,
    null_sprite: SpriteAsset,
}

impl SpriteAssets {
    pub fn new(fs: &FileSystem) -> SpriteAssets {
        let mut anim_sets = fs.read_sprite_animations();
        anim_sets.insert(NULL_ANIM_SET_INDEX, generate_null_sprite_anim_set());

        SpriteAssets {
            assets: HashMap::new(),
            anim_sets,
            null_sprite: generate_null_sprite_asset(),
        }
    }

    pub fn get(&self, sprite_index: usize) -> &SpriteAsset {
        self.assets.get(&sprite_index).unwrap_or(&self.null_sprite)
    }

    pub fn get_anim_set(&self, anim_set_index: usize) -> &SpriteAnimSet {
        self.anim_sets.get(&anim_set_index).unwrap()
    }

    // Load a sprite for future use.
    pub fn load(&mut self, fs: &FileSystem, sprite_index: usize) -> &SpriteAsset {
        if !self.assets.contains_key(&sprite_index) {
            let info = fs.read_sprite_header(sprite_index);
            let assembly = fs.read_sprite_assembly(info.assembly_index, &info);
            let palette = fs.read_sprite_palette(info.palette_index, 0).unwrap();
            let tiles = fs.read_sprite_tiles(info.bitmap_index, assembly.chip_max);

            let sprite = SpriteAsset {
                index: sprite_index,
                tiles,
                assembly,
                palette,
                palette_index: info.palette_index,
                anim_set_index: info.anim_index,
            };
            self.assets.insert(sprite_index, sprite);
        }

        self.assets.get(&sprite_index).unwrap()
    }

    pub fn dump(&self) {
        for (index, sprite) in self.assets.iter() {
            let mut surface = Surface::new(sprite.tiles.width, sprite.tiles.height);
            blit_bitmap_to_surface(&sprite.tiles, &mut surface, 0, 0, sprite.tiles.width as i32, sprite.tiles.height as i32, 0, 0, &sprite.palette, 0, BitmapBlitFlags::default());
            surface.write_to_bmp(Path::new(&format!("debug_output/sprite_{:03}.bmp", index)));
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

fn generate_null_sprite_asset() -> SpriteAsset {
    let null_sprite_bmp = Bmp::from_path(Path::new("data/null_sprite.bmp"));
    let null_palette = null_sprite_bmp.get_raw_palette();
    let null_sprite_bitmap = Bitmap::from_raw_data(null_sprite_bmp.width, null_sprite_bmp.height, null_sprite_bmp.pixels);

    SpriteAsset {
        index: NULL_SPRITE_INDEX,
        anim_set_index: NULL_ANIM_SET_INDEX,
        tiles: null_sprite_bitmap,
        palette: Palette::from_colors(&null_palette),
        palette_index: 0,
        assembly: SpriteAssembly {
            index: NULL_ASSEMBLY_SET_INDEX,
            frames: vec![
                SpriteAssemblyFrame {
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
                },
            ],
            chip_max: 1,
        },
    }
}
