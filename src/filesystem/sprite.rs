use std::collections::HashMap;
use std::io::{Cursor, Seek};
use std::io::SeekFrom;
use byteorder::LittleEndian;
use byteorder::ReadBytesExt;
use crate::assets::Assets;
use crate::filesystem::filesystem::FileSystem;
use crate::GameMode;
use crate::software_renderer::bitmap::Bitmap;
use crate::software_renderer::palette::Palette;
use crate::sprites::sprite_anim::SpriteAnim;
use crate::sprites::sprite_anim::SpriteAnimFrame;
use crate::sprites::sprite_anim::SpriteAnimSet;
use crate::sprites::sprite_anim::FACING_COUNT;
use crate::sprites::sprite_assembly::SpriteAssembly;
use crate::sprites::sprite_assembly::SpriteAssemblyFrame;
use crate::sprites::sprite_assembly::SpriteAssemblyChip;
use crate::sprites::sprite_assembly::SpriteAssemblyChipFlags;
use crate::sprites::sprite_header::SpriteHeader;
use crate::world_script::world_animation_script::WorldAnimationScript;

impl FileSystem {

    pub fn read_sprite_header(&self, index: usize) -> SpriteHeader {
        let mut data = self.backend.get_sprite_header_data(index);

        let mut header = match self.mode {
            GameMode::Pc => SpriteHeader {
                index,
                bitmap_index: data.read_u8().unwrap() as usize,
                assembly_index: data.read_u8().unwrap() as usize,
                palette_index: data.read_u8().unwrap() as usize,
                size_flags: data.read_u8().unwrap() as u32,
                anim_index: data.read_u8().unwrap() as usize,
                flags: data.read_u8().unwrap() as u32,
                hand_x: 0,
                hand_y: 0,
                enemy_unknown1: 0,
                enemy_unknown2: 0,
                enemy_unknown3: 0,
            },
            GameMode::Snes => SpriteHeader {
                index,
                bitmap_index: data.read_u8().unwrap() as usize,
                assembly_index: data.read_u8().unwrap() as usize,
                palette_index: data.read_u8().unwrap() as usize,
                anim_index: data.read_u8().unwrap() as usize,
                size_flags: data.read_u8().unwrap() as u32,
                flags: 0,
                hand_x: 0,
                hand_y: 0,
                enemy_unknown1: 0,
                enemy_unknown2: 0,
                enemy_unknown3: 0,
            },
        };

        if matches!(self.mode, GameMode::Pc) {
            header.bitmap_index = index;
            header.assembly_index = index;
            header.palette_index = index;
        }
                
        // Enemy sprites have more data.
        if data.get_ref().len() > 6 {
            header.hand_x = data.read_i8().unwrap() as i32;
            header.hand_y = data.read_i8().unwrap() as i32;
            header.enemy_unknown1 = data.read_u8().unwrap() as u32;
            header.enemy_unknown2 = data.read_u8().unwrap() as u32;
            header.enemy_unknown3 = data.read_u8().unwrap() as u32;
        }

        header
    }

    // Reads a sprite's assembly data.
    pub fn read_sprite_assembly(&self, index: usize, size_flags: u32) -> (SpriteAssembly, HashMap<u64, SpriteAssemblyFrame>) {
        let mut data = self.backend.get_sprite_assembly_data(index);

        let (assembly, frames) = match self.mode {
            GameMode::Pc => {
                data.seek(SeekFrom::Start(3)).unwrap();
                parse_pc_sprite_assembly(index, &mut data)
            },
            GameMode::Snes => {
                let (groups_per_frame, tiles_per_group) = match size_flags & 0x3 {
                    0 => (1, 4),
                    1 => (1, 8),
                    2 => (3, 4),
                    3 => (3, 8),
                    _ => (1, 4),
                };
                parse_snes_sprite_assembly(index, groups_per_frame, tiles_per_group, &mut data)
            },
        };

        if frames.len() == 0 {
            panic!("Sprite assembly {} has no frames.", assembly.index);
        }

        (assembly, frames)
    }

    // Read all sprite animations.
    //
    // Sprite animations are split into slots (sprite assembly frames) and durations.
    // They also list data for each of the 4 sprite facings.
    pub fn read_sprite_animations(&self) -> HashMap<usize, SpriteAnimSet> {
        let (
            slot_ptrs,
            mut slot_data,
            interval_ptrs,
            mut interval_data
        ) = self.backend.get_sprite_animation_data();

        let slot_data = slot_data.get_mut();
        let interval_data = interval_data.get_mut();

        // Parse animation frames. Each animation starts on a 4-byte offset in slot and interval
        // data, but ends only if an 0xFF slot byte is encountered.
        let mut anim_sets = HashMap::new();
        for set_index in 0..slot_ptrs.len() - 1 {
            let mut set = SpriteAnimSet::new(set_index);

            // Read each animation from 4 byte starts.
            let anim_data_len = slot_ptrs[set_index + 1] - slot_ptrs[set_index];
            let anim_count = ((anim_data_len / 4) as f64 / FACING_COUNT as f64).ceil() as usize;
            let data_per_facing = anim_data_len / 4;
            for anim_index in 0..anim_count {
                set.add_anim(SpriteAnim {
                    frames: parse_sprite_animation_frames(
                        &slot_data,
                        slot_ptrs[set_index],
                        &interval_data,
                        interval_ptrs[set_index],
                        anim_index,
                        data_per_facing,
                    ),
                });
            }

            anim_sets.insert(set_index, set);
       }

        anim_sets
    }

    // Read a sprite palette.
    pub fn read_sprite_palette(&self, index: usize, offset: usize) -> Option<Palette> {
        let palette_wrap = self.backend.get_sprite_palette(index);
        if palette_wrap.is_none() {
            return None;
        }
        let source_palette = palette_wrap.unwrap();
        let mut palette = Palette::new(0);

        for i in 0..16 {
            if i < source_palette.colors.len() {
                palette.colors.push(source_palette.colors[i + offset]);
            } else {
                palette.colors.push([0, 0, 0, 0xFF]);
            }
        }

        Some(palette)
    }

    // Read a sprite's graphics tiles.
    pub fn read_sprite_tiles(&self, sprite_tiles_index: usize, chip_count: usize) -> Bitmap {
        match self.mode {
            GameMode::Pc => {
                let data = self.backend.get_sprite_graphics(sprite_tiles_index, chip_count, false);
                let bitmap_height = (data.len() as f64 / 256.0).ceil() as u32;
                Bitmap::from_raw_data(256, bitmap_height, data)
            },
            GameMode::Snes => {
                let data = self.backend.get_sprite_graphics(sprite_tiles_index, chip_count, sprite_tiles_index > 6);
                let bitmap_height = (data.len() as f64 / 128.0).ceil() as u32;
                Bitmap::from_raw_data(128, bitmap_height, data)
            },
        }
    }

    pub fn read_world_animation_script(&self) -> WorldAnimationScript {
        let data = self.backend.get_world_sprite_data();
        let count = if self.mode == GameMode::Pc { 168 } else { 166 };
        WorldAnimationScript::new(&data, count)
    }

    // Read all sprite tile graphics for a given list of world graphics.
    pub fn read_world_sprite_tiles_all(&self, world_index: usize, bitmap_indices: [usize; 4]) -> Bitmap {
        let mut bitmap_data = vec![0u8; 0x8000];

        for (index, packet_index) in bitmap_indices.iter().enumerate() {
            let offset = index * 0x2000;
            self.read_world_sprite_tiles(world_index, *packet_index, offset, &mut bitmap_data)
        }

        Bitmap::from_raw_data(128, 256, bitmap_data)
    }

    // Read a single sprite tile graphic set and place its pixel data directly into a bitmap.
    pub fn read_world_sprite_tiles(&self, world_index: usize, tiles_index: usize, offset: usize, bitmap_data: &mut Vec<u8>) {
        if tiles_index & 0x80 > 0 {
            return;
        }

        let tile_pixel_data = self.backend.get_world_sprite_graphics(world_index, tiles_index);
        if tile_pixel_data.is_some() {
            let pixel_data = tile_pixel_data.unwrap();
            let data_len = pixel_data.len();
            bitmap_data[offset..offset + data_len].copy_from_slice(&pixel_data);
        }
    }

    // Return world player sprite data.
    pub fn read_world_player_sprite_tiles(&self) -> Vec<u8> {
        self.backend.get_world_player_sprite_graphics().unwrap()
    }

    // Return Epoch sprite data.
    pub fn read_world_epoch_sprite_tiles(&self) -> Vec<u8> {
        self.backend.get_world_epoch_sprite_graphics().unwrap()
    }
}

// Read an animation's frames directly from slot and interval data.
fn parse_sprite_animation_frames(slot_data: &Vec<u8>, start_slot_offset: usize, interval_data: &Vec<u8>, start_interval_offset: usize, anim_index: usize, data_per_facing: usize) -> Vec<SpriteAnimFrame> {
    let mut frame_index = 0;
    let mut frames: Vec<SpriteAnimFrame> = Vec::new();

    loop {
        // Calculate offsets for each facing.
        let offsets = [
            start_slot_offset + (data_per_facing * 0) + anim_index * 4 + frame_index,
            start_slot_offset + (data_per_facing * 1) + anim_index * 4 + frame_index,
            start_slot_offset + (data_per_facing * 2) + anim_index * 4 + frame_index,
            start_slot_offset + (data_per_facing * 3) + anim_index * 4 + frame_index,
        ];

        // Build a sprite frame from data for all 4 facings.
        let sprite_frames = [
            slot_data[offsets[0]] as usize,
            slot_data[offsets[1]] as usize,
            slot_data[offsets[2]] as usize,
            slot_data[offsets[3]] as usize,
        ];

        // A frame duration is measured in 1/60th of a second (or one SNES frame).
        let interval_offset = start_interval_offset + anim_index * 4 + frame_index;
        let delay = interval_data[interval_offset] as u32;
        if delay == 0 {
            break;
        }

        frames.push(SpriteAnimFrame {
            sprite_frames,
            delay,
        });
        frame_index += 1;
    }

    frames
}

fn parse_pc_sprite_assembly(assembly_index: usize, data: &mut Cursor<Vec<u8>>) -> (SpriteAssembly, HashMap<u64, SpriteAssemblyFrame>) {

    // Number of frames in this assembly.
    let frame_count = data.read_u16::<LittleEndian>().unwrap() as usize;
    data.read_u16::<LittleEndian>().unwrap();
    let mut assembly = SpriteAssembly {
        index: assembly_index,
        chip_max: 0,
        frame_keys: Vec::new(),
    };
    let mut frames: HashMap<u64, SpriteAssemblyFrame> = HashMap::new();

    for frame_index in 0..frame_count {
        let mut frame = SpriteAssemblyFrame::new();

        // Number of tiles in this assembly frame.
        let tile_count = data.read_u8().unwrap();
        for _ in 0..tile_count {
            let value = data.read_u16::<LittleEndian>().unwrap() as usize;

            let x = data.read_i8().unwrap() as i32;
            let y = data.read_i8().unwrap() as i32;
            let flags_value = data.read_u8().unwrap();

            // Tile index is (almost) always missing bit 4. Weird.
            // Is this also the case for the SNES data?
            let weird_bit = (value & 0x8) as u8;
            let chip = (value & 0x07) | (value & 0xFFF0) >> 1;
            assembly.chip_max = assembly.chip_max.max(chip);

            // Map flags to internal ones.
            let mut flags = SpriteAssemblyChipFlags::default();
            if flags_value & 0x01 != 0 {
                flags |= SpriteAssemblyChipFlags::FLIP_X;
            }
            if flags_value & 0x02 != 0 {
                flags |= SpriteAssemblyChipFlags::UNUSED;
            }
            if weird_bit != 0 {
                flags |= SpriteAssemblyChipFlags::UNKNOWN;
            }

            let mut add_flags = SpriteAssemblyChipFlags::empty();
            if y < -24 {
                add_flags |= SpriteAssemblyChipFlags::IS_TOP;
            } else {
                add_flags |= SpriteAssemblyChipFlags::IS_BOTTOM;
            }

            let src_x = ((chip % 32) * 8) as i32;
            let src_y = ((chip / 32) * 16) as i32;

            frame.chips.push(SpriteAssemblyChip {
                src_index: chip,
                x, y,
                src_x, src_y,
                width: 16,
                height: 16,
                flags,
            });
        }

        let key = Assets::asset_key_sprite_assembly_frame_scene(assembly_index, frame_index);
        assembly.frame_keys.push(key);
        frames.insert(key, frame);
    }

    (assembly, frames)
}

fn parse_snes_sprite_assembly(assembly_index: usize, groups_per_frame: usize, tiles_per_group: usize, data: &mut Cursor<Vec<u8>>) -> (SpriteAssembly, HashMap<u64, SpriteAssemblyFrame>) {
    let tiles_per_frame = groups_per_frame * tiles_per_group;
    let chips_per_group = tiles_per_group * 4;
    let frame_count = data.get_ref().len() / (tiles_per_frame * 10);
    let mut assembly = SpriteAssembly {
        index: assembly_index,
        chip_max: 0,
        frame_keys: Vec::new(),
    };
    let mut frames: HashMap<u64, SpriteAssemblyFrame> = HashMap::new();

    // Read chip data.
    for frame_index in 0..frame_count {
        let mut frame = SpriteAssemblyFrame::new();
        frame.chips.resize(tiles_per_frame * 4, SpriteAssemblyChip::default());

        for group in 0..groups_per_frame {
            let group_start = group * chips_per_group;

            // Upper row chips.
            for tile in 0..tiles_per_group {
                frame.chips[group_start + tile * 4 + 0] = parse_snes_sprite_assembly_chip(data, 0, 0);
                frame.chips[group_start + tile * 4 + 1] = parse_snes_sprite_assembly_chip(data, 8, 0);
            }

            // Bottom row chips.
            for tile in 0..tiles_per_group {
                frame.chips[group_start + tile * 4 + 2] = parse_snes_sprite_assembly_chip(data, 0, 8);
                frame.chips[group_start + tile * 4 + 3] = parse_snes_sprite_assembly_chip(data, 8, 8);
            }
        }

        // Set tile offsets for each chip.
        // Offsets are stored per tile, but we expand them to each chip internally.
        for group in 0..groups_per_frame {
            let group_start = group * chips_per_group;

            for tile in 0..tiles_per_group {
                let tile_start = group_start + tile * 4;

                let ox = data.read_i8().unwrap() as i32;
                let oy = data.read_i8().unwrap() as i32;

                // Tiles above -24 are considered to be the top of the sprite.
                let is_top = oy < -24;

                for chip in 0..4 {
                    frame.chips[tile_start + chip].x += ox;
                    frame.chips[tile_start + chip].y += oy;
                    if is_top {
                        frame.chips[tile_start + chip].flags |= SpriteAssemblyChipFlags::IS_TOP;
                    } else {
                        frame.chips[tile_start + chip].flags |= SpriteAssemblyChipFlags::IS_BOTTOM;
                    }
                }
            }
        }

        // Track the highest chip index.
        for tile in frame.chips.iter() {
            assembly.chip_max = assembly.chip_max.max(tile.src_index);
        }

        let key = Assets::asset_key_sprite_assembly_frame_scene(assembly_index, frame_index);
        assembly.frame_keys.push(key);
        frames.insert(key, frame);
    }

    (assembly, frames)
}

fn parse_snes_sprite_assembly_chip(data: &mut Cursor<Vec<u8>>, x: i32, y: i32) -> SpriteAssemblyChip {
    let value = data.read_u16::<LittleEndian>().unwrap();

    let chip = (value & 0x3FF) as usize;
    let src_x = ((chip % 16) * 8) as i32;
    let src_y = ((chip / 16) * 8) as i32;

    let mut flags = SpriteAssemblyChipFlags::default();
    if value & 0x4000 != 0 {
        flags |= SpriteAssemblyChipFlags::FLIP_X;
    }
    if value & 0x8000 != 0 {
        flags |= SpriteAssemblyChipFlags::FLIP_Y;
    }

    SpriteAssemblyChip {
        src_index: chip,
        x, y,
        src_x, src_y,
        width: 8,
        height: 8,
        flags,
    }
}
