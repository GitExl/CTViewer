use std::io::{BufRead, Cursor};
use std::io::Read;

use byteorder::LittleEndian;
use byteorder::ReadBytesExt;

use crate::filesystem::filesystem::{FileSystem, ParseMode};
use crate::map::MapChip;
use crate::map::MapChipFlags;
use crate::software_renderer::bitmap::Bitmap;
use crate::tileset::ChipAnim;
use crate::tileset::ChipAnimFrame;
use crate::tileset::Tile;
use crate::tileset::TileSet;

impl FileSystem {

    // Read a layer 1 or 2 tileset for a world map.
    pub fn read_world_tileset_layer12 (&self, index_chips: [usize; 8], index_assembly: usize) -> TileSet {

        // Read chip bitmap data.
        let mut bitmap_data = vec![0u8; 0x10000];
        for (index, chipset) in index_chips.iter().enumerate() {
            if *chipset == 0x80 {
                continue;
            }

            // Load layer 1/2 chip graphics data referenced by the tileset.
            let chips_data = self.backend.get_world_tileset12_graphics(*chipset);
            if chips_data.is_some() {
                set_chip_bitmap_data(&mut bitmap_data, &mut chips_data.unwrap(), index * 0x2000);
            }
        }

        // Split chip bitmap data into individual chips.
        let chip_bitmaps = split_chip_graphics(&bitmap_data, bitmap_data.len() / 64);

        // Read tile assembly data.
        let mut assembly_data = self.backend.get_world_tileset12_assembly_data(index_assembly);
        let tiles = parse_world_l12_tile_assembly(&mut assembly_data, 512, 16);

        TileSet {
            index: 0,
            index_assembly,
            tiles,
            chip_bitmaps,
            animated_chip_bitmaps: Vec::new(),
            chip_anims: Vec::new(),
        }
    }

    // Read a layer 3 tileset for a world map.
    pub fn read_world_tileset_layer3 (&self, index_chips: usize, index_assembly: usize) -> TileSet {
        let mut chip_bitmaps = Vec::<Bitmap>::new();
        let mut tiles = Vec::<Tile>::new();

        if index_chips != 0x80 {
            let chips_data = self.backend.get_world_tileset3_graphics(index_chips);
            if chips_data.is_some() {
                let data = chips_data.unwrap();
                let chip_count = data.len() / 64;
                chip_bitmaps.extend(split_chip_graphics(&data, chip_count));
            }

            let mut assembly = self.backend.get_world_tileset3_assembly_data(index_assembly);
            tiles.extend(parse_world_l3_tile_assembly(&mut assembly, 512, 4));
        }

        TileSet {
            index: index_chips,
            index_assembly,
            tiles,
            chip_bitmaps,
            animated_chip_bitmaps: Vec::new(),
            chip_anims: Vec::new(),
        }
    }

    // Read a tileset for a scene map layer 1 or 2.
    pub fn read_scene_tileset_layer12 (&self, tileset_index: usize, index_assembly: usize, chip_anims_index: usize) -> TileSet {
        let mut bitmap_data = vec![0u8; 0xE000];
        let mut animated_bitmap_data = vec![0u8; 0x8000];

        let mut chip_bitmaps = Vec::<Bitmap>::new();
        let mut animated_chip_bitmaps = Vec::<Bitmap>::new();
        let mut tiles = Vec::<Tile>::new();
        let mut chip_anims = Vec::<ChipAnim>::new();

        if tileset_index != 0xFF && tileset_index != 0xFFFF {

            // Each tileset refers to 8 sets of pixel data containing 128 8x8 pixel chips. The
            // last 2 sets are for animated tiles.
            let mut data = self.backend.get_scene_tileset_data(tileset_index);
            let mut chipsets = [0u8; 8];
            data.read_exact(&mut chipsets).unwrap();

            for (chipset_index, chipset) in chipsets.iter().enumerate() {
                if *chipset == 0xFF {
                    continue;
                }

                // Load layer 1/2 chip graphics data referenced by the tileset.
                let mut chipset_data = self.backend.get_scene_tileset12_graphics(*chipset as usize);

                // Set 0 to 5 contain regular chips. Set 6 contains animated chips. Set 7 contains
                // regular chips again; are these unique to the PC version?
                if chipset_index == 6 {
                    set_chip_bitmap_data(&mut animated_bitmap_data, &mut chipset_data, 0);
                } else if chipset_index == 7 {
                    set_chip_bitmap_data(&mut bitmap_data, &mut chipset_data, 0xC000);
                } else {
                    set_chip_bitmap_data(&mut bitmap_data, &mut chipset_data, chipset_index * 0x2000);
                }
            }

            chip_bitmaps.extend(split_chip_graphics(&bitmap_data, bitmap_data.len() / 64));
            animated_chip_bitmaps.extend(split_chip_graphics(&animated_bitmap_data, animated_bitmap_data.len() / 64));

            // Using these chips, 16x16 tiles are constructed. Each tile has 4 corners. Each
            // corner refers to a chip, a palette and some flags to describe the tile.
            let mut assembly_data = self.backend.get_scene_tileset12_assembly_data(index_assembly);
            tiles.extend(parse_scene_tile_assembly(&mut assembly_data, 512, 16, self.parse_mode));

            // Read tile animation data.
            let anim_data = self.backend.get_scene_tileset12_animation_data(chip_anims_index);
            if anim_data.is_some() {
                chip_anims.extend(parse_chip_anims(&mut anim_data.unwrap(), self.parse_mode));
            }
        }

        TileSet {
            index: tileset_index,
            index_assembly,

            tiles,
            chip_bitmaps,
            animated_chip_bitmaps,
            chip_anims,
        }
    }

    // Read a tileset for layer 3 of scene maps.
    pub fn read_scene_tileset_layer3 (&self, chips_index: usize, index_assembly: usize) -> TileSet {
        let mut chip_bitmaps = Vec::<Bitmap>::new();
        let mut tiles = Vec::<Tile>::new();

        if chips_index != 0xFF && chips_index != 0xFFFF {
            let chips_data = self.backend.get_scene_tileset3_graphics(chips_index);
            if chips_data.is_some() {
                let data = chips_data.unwrap();
                let chip_count = data.len() / 64;
                chip_bitmaps.extend(split_chip_graphics(&data, chip_count));
            }

            let assembly_data = self.backend.get_scene_tileset3_assembly_data(index_assembly);
            if assembly_data.is_some() {
                tiles.extend(parse_scene_tile_assembly(&mut assembly_data.unwrap(), 256, 4, self.parse_mode));
            }
        }

        TileSet {
            index: chips_index,
            index_assembly,
            tiles,
            chip_bitmaps,
            animated_chip_bitmaps: Vec::new(),
            chip_anims: Vec::new(),
        }
    }
}

// Read tile chip animation data.
//
// An animation lists an offset for and animates 4 chips in a tileset.
fn parse_chip_anims(reader: &mut Cursor<Vec<u8>>, parse_mode: ParseMode) -> Vec<ChipAnim> {
    let mut anims = Vec::<ChipAnim>::new();
    while reader.fill_buf().unwrap().len() > 0 {
        let frame_count = reader.read_u8().unwrap() as usize;
        if frame_count == 0 || frame_count == 0x80 {
            break;
        }

        let dest_chip = match parse_mode {
            ParseMode::Snes => (reader.read_u16::<LittleEndian>().unwrap() as usize - 0x2000) / 16,
            ParseMode::Pc => reader.read_u16::<LittleEndian>().unwrap() as usize / 32,
        };

        let mut anim = ChipAnim {
            dest_chip,
            frames: Vec::new(),
            frame: 0,
            timer: 0.0,
        };

        // Frame durations are listed first, then the source animated chip indices.
        for _ in 0..frame_count {
            let duration_bits = (reader.read_u8().unwrap() as usize) >> 4;
            let duration = match duration_bits {
                1 => 16.0,
                2 => 12.0,
                4 => 8.0,
                8 => 4.0,
                _ => 0.0,
            };
            let frame = ChipAnimFrame {
                duration: duration * (1.0 / 60.0),
                src_chip: 0,
            };
            anim.frames.push(frame);
        }
        for frame_index in 0..frame_count {
            let src_chip = match parse_mode {
                ParseMode::Snes => (reader.read_u16::<LittleEndian>().unwrap() - 0x6000) as usize / 32,
                ParseMode::Pc => reader.read_u16::<LittleEndian>().unwrap() as usize / 32,
            };
            anim.frames[frame_index].src_chip = src_chip;
        }

        anims.push(anim);
    }

    anims
}

// Read and copy chip bitmap graphics data.
//
// The number of chips is listed first, then the bitmap data.
fn set_chip_bitmap_data(bitmap_data: &mut Vec<u8>, data: &mut Vec<u8>, offset: usize) {
    bitmap_data[offset..offset + data.len()].copy_from_slice(&data);
}

// Reads tile assembly data for a scene tileset.
fn parse_scene_tile_assembly(reader: &mut Cursor<Vec<u8>>, tile_count: usize, palette_size: usize, parse_mode: ParseMode) -> Vec<Tile> {
    let mut tiles = vec![Tile::default(); tile_count];

    // Read data for each tile corner (chip).
    for tile in tiles.iter_mut() {
        for corner in 0..4 {
            let mut flags = MapChipFlags::default();
            let chip;
            let palette;
            
            match parse_mode {

                // byte 1 & 2
                // chip: 10 bits
                // flip x: 1 bit
                // flip y: 1 bit
                // palette: 4 bits
                //
                // byte 2
                // priority: 1 bit
                ParseMode::Pc => {
                    let data1 = reader.read_u16::<LittleEndian>().unwrap();
                    chip = (data1 & 0x3FF) as usize;

                    if data1 & 0x400 > 0 {
                        flags |= MapChipFlags::FLIP_X;
                    }
                    if data1 & 0x800 > 0 {
                        flags |= MapChipFlags::FLIP_Y;
                    }
                    palette = ((data1 >> 12) & 0xF) as usize;

                    let data2 = reader.read_u8().unwrap();
                    if data2 & 0x01 > 0 {
                        flags |= MapChipFlags::PRIORITY;
                    }
                },

                // chip: 10 bits, offset by 256 for location assemblies
                // palette: 3 bits
                // priority: 1 bit
                // flip x: 1 bit
                // flip y: 1 bit
                ParseMode::Snes => {
                    let data1 = reader.read_u16::<LittleEndian>().unwrap();
                    if palette_size == 4 {
                        chip = (data1 & 0x3FF) as usize;
                    } else {
                        chip = (data1 & 0x3FF).saturating_sub(256) as usize;
                    }
                    palette = (data1 & 0x1C00) as usize >> 10;
                    if data1 & 0x2000 > 0 {
                        flags |= MapChipFlags::PRIORITY;
                    }
                    if data1 & 0x4000 > 0 {
                        flags |= MapChipFlags::FLIP_X;
                    }
                    if data1 & 0x8000 > 0 {
                        flags |= MapChipFlags::FLIP_Y;
                    }
                }
            };

            tile.corners[corner] = MapChip {
                chip,
                palette: palette * palette_size,
                flags,
            };
        }
    }

    tiles
}

// Read tile assembly data for world layer 1 or 2 tiles.
fn parse_world_l12_tile_assembly(data: &mut Cursor<Vec<u8>>, tile_count: usize, palette_size: usize) -> Vec<Tile> {
    let mut tiles = vec![Tile::default(); tile_count];

    for tile in tiles.iter_mut() {
        for corner in 0..4 {
            let mut flags = MapChipFlags::default();

            // chip: 10 bits
            // palette: 3 bits
            // priority: 1 bit
            // flip x: 1 bit
            // flip y: 1 bit
            let data1 = data.read_u16::<LittleEndian>().unwrap();
            if data1 & 0x2000 > 0 {
                flags |= MapChipFlags::PRIORITY;
            }
            if data1 & 0x4000 > 0 {
                flags |= MapChipFlags::FLIP_X;
            }
            if data1 & 0x8000 > 0 {
                flags |= MapChipFlags::FLIP_Y;
            }

            tile.corners[corner] = MapChip {
                chip: (data1 & 0x3FF) as usize,
                palette: ((data1 >> 10) & 0x7) as usize * palette_size,
                flags,
            };
        }
    }

    tiles
}

// Read tile assembly data for world layer 3 tiles.
//
// These are the same as the layer 1 or 2 assembly, except they are stored in order from left to
// right, top to bottom.
fn parse_world_l3_tile_assembly(data: &mut Cursor<Vec<u8>>, tile_count: usize, palette_size: usize) -> Vec<Tile> {
    let mut tiles = vec![Tile::default(); tile_count];
    for i in 0..tile_count * 4 {
        let mut flags = MapChipFlags::default();

        let data1 = data.read_u16::<LittleEndian>().unwrap();
        if data1 & 0x2000 > 0 {
            flags |= MapChipFlags::PRIORITY;
        }
        if data1 & 0x4000 > 0 {
            flags |= MapChipFlags::FLIP_X;
        }
        if data1 & 0x8000 > 0 {
            flags |= MapChipFlags::FLIP_Y;
        }

        let x = i % 32;
        let y = i / 32;
        let tile_x = x / 2;
        let tile_y = y / 2;
        let cx = x % 2;
        let cy = y % 2;
        let tile = &mut tiles[tile_x + (tile_y * 16)];
        tile.corners[cx + cy * 2] = MapChip {
            chip: (data1 & 0x3FF) as usize,
            palette: ((data1 >> 10) & 0x7) as usize * palette_size,
            flags,
        };
    }

    tiles
}

// Unpack linear chip graphics data into 8x8 chip bitmaps.
fn split_chip_graphics(chip_data: &Vec<u8>, chip_count: usize) -> Vec<Bitmap> {
    let mut chips =  Vec::<Bitmap>::new();

    let mut x = 0;
    let mut y = 0;
    for _ in 0..chip_count {
        let mut pixels = vec![0u8; 64];

        let mut dest = 0;
        for row in 0..8 {
            let src = (y + row) * 128 + x;
            pixels[dest..dest + 8].copy_from_slice(&chip_data[src..src + 8]);
            dest += 8;
        }
        chips.push(Bitmap::from_raw_data(8, 8, pixels));

        // 16 chips (128 pixels) are stored per row.
        x += 8;
        if x >= 128 {
            x = 0;
            y += 8;
        }
    }

    chips
}
