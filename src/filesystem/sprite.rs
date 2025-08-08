use std::collections::HashMap;
use std::io::{Cursor, Seek};
use std::io::SeekFrom;
use byteorder::LittleEndian;
use byteorder::ReadBytesExt;

use crate::filesystem::filesystem::{FileSystem, ParseMode};
use crate::software_renderer::bitmap::Bitmap;
use crate::software_renderer::palette::Palette;
use crate::sprites::sprite_anim::SpriteAnim;
use crate::sprites::sprite_anim::SpriteAnimFrame;
use crate::sprites::sprite_anim::SpriteAnimSet;
use crate::sprites::sprite_anim::DIRECTION_COUNT;
use crate::sprites::sprite_assembly::SpriteAssembly;
use crate::sprites::sprite_assembly::SpriteAssemblyFrame;
use crate::sprites::sprite_assembly::SpriteAssemblyTile;
use crate::sprites::sprite_assembly::SpriteAssemblyTileFlags;
use crate::sprites::sprite_header::SpriteHeader;
use crate::sprites::sprite_manager::WORLD_ANIM_SET_INDEX;
use crate::sprites::sprite_manager::WORLD_ASSEMBLY_SET_INDEX;

impl FileSystem {

    pub fn read_sprite_header(&self, index: usize) -> SpriteHeader {
        let mut data = self.backend.get_sprite_header_data(index);

        let mut header = match self.parse_mode {
            ParseMode::Pc => SpriteHeader {
                index,
                bitmap_index: data.read_u8().unwrap() as usize,
                assembly_index: data.read_u8().unwrap() as usize,
                palette_index: data.read_u8().unwrap() as usize,
                assembly_set_count: data.read_u8().unwrap() as u32 + 1,
                anim_index: data.read_u8().unwrap() as usize,
                flags: data.read_u8().unwrap() as u32,
                hand_x: 0,
                hand_y: 0,
                enemy_unknown1: 0,
                enemy_unknown2: 0,
                enemy_unknown3: 0,
            },
            ParseMode::Snes => SpriteHeader {
                index,
                bitmap_index: data.read_u8().unwrap() as usize,
                assembly_index: data.read_u8().unwrap() as usize,
                palette_index: data.read_u8().unwrap() as usize,
                anim_index: data.read_u8().unwrap() as usize,
                assembly_set_count: data.read_u8().unwrap() as u32 + 1,
                flags: 0,
                hand_x: 0,
                hand_y: 0,
                enemy_unknown1: 0,
                enemy_unknown2: 0,
                enemy_unknown3: 0,
            },
        };
        
        if matches!(self.parse_mode, ParseMode::Pc) {
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
    pub fn read_sprite_assembly(&self, index: usize, sprite_header: &SpriteHeader) -> SpriteAssembly {
        let mut data = self.backend.get_sprite_assembly_data(index);

        match self.parse_mode {
            ParseMode::Pc => {
                data.seek(SeekFrom::Start(3)).unwrap();
                parse_pc_sprite_assembly(index, &mut data)
            },
            ParseMode::Snes => {
                parse_snes_sprite_assembly(index, sprite_header.assembly_set_count as usize, &mut data)
            },
        }
    }

    // Read all sprite animations.
    //
    // Sprite animations are split into slots (sprite assembly frames) and durations.
    // They also list data for each of the 4 directions a sprite can be facing.
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
            let mut set = SpriteAnimSet {
                index: set_index,
                anims: Vec::new(),
            };

            // Read each animation from 4 byte starts.
            let anim_data_len = slot_ptrs[set_index + 1] - slot_ptrs[set_index];
            let anim_count = (anim_data_len / 4) / DIRECTION_COUNT;
            for anim_index in 0..anim_count {
                set.anims.push(SpriteAnim {
                    frames: parse_sprite_animation_frames(
                        &slot_data,
                        slot_ptrs[set_index],
                        &interval_data,
                        interval_ptrs[set_index],
                        anim_index,
                        anim_count,
                    ),
                });
            }

            anim_sets.insert(set_index, set);
        }

        anim_sets
    }

    // Read a sprite palette.
    pub fn read_sprite_palette(&self, index: usize) -> Option<Palette> {
        let palette_wrap = self.backend.get_sprite_palette(index);
        if palette_wrap.is_none() {
            return None;
        }
        let mut palette = palette_wrap.unwrap();

        // Extend the palette up to 16 colors if needed.
        let missing_colors = 16 - palette.colors.len();
        for _ in 0..missing_colors {
            palette.colors.push([0, 0, 0, 0xFF]);
        }

        Some(palette)
    }

    // Read a sprite's graphics tiles.
    pub fn read_sprite_tiles(&self, sprite_index: usize, chip_count: usize) -> Bitmap {
        match self.parse_mode {
            ParseMode::Pc => {
                let data = self.backend.get_sprite_graphics(sprite_index, chip_count, false);
                let bitmap_height = (data.len() as f64 / 256.0).ceil() as u32;
                Bitmap::from_raw_data(256, bitmap_height, data)
            },
            ParseMode::Snes => {
                let data = self.backend.get_sprite_graphics(sprite_index, chip_count, sprite_index > 7);
                let bitmap_height = (data.len() as f64 / 128.0).ceil() as u32;
                Bitmap::from_raw_data(128, bitmap_height, data)
            },
        }
    }

    // Read world sprite animation and assembly data.
    //
    // World "sprites" are really just one sprite with all animations and assembly data stored
    // in one location. These are treated internally as one sprite with many animations, because
    // there is no clearly defined boundary between the sprites themselves.
    //
    // The world loads the graphics tile data at the top of VRAM. At some point, the game
    // loads in different graphics data, so animations do not produce their intended sprites
    // in all worlds at all times.
    pub fn read_world_sprites(&self) -> (SpriteAssembly, SpriteAnimSet) {

        // shapeSeqTbl.bin contains both animation frames and sprite tile assemblies for all
        // world sprites.
        let mut data = self.backend.get_world_sprite_data();

        // Read offsets to the data for each animation.
        // The SNES version has two less (the dimensional vortex probably).
        let count = if matches!(self.parse_mode, ParseMode::Pc) { 168 } else { 166 };
        let mut offsets = Vec::new();
        for _ in 0..count {
            offsets.push(data.read_u16::<LittleEndian>().unwrap() as u64 - 0xE000);
        }

        // Create one assembly and animation set for all world sprite animations.
        let mut assembly = SpriteAssembly::new(WORLD_ASSEMBLY_SET_INDEX);
        let mut anim_set = SpriteAnimSet::new(WORLD_ANIM_SET_INDEX);

        // Each world sprite animation consists of a series of instructions and data. The first
        // byte is the instruction followed by 1 or more bytes of data. Animations have no real end
        // except for a 0-duration frame, which will show that frame forever or on an instruction
        // that loops back to an earlier point in the animation.
        //
        // Instructions:
        // 0: Unknown.
        // 1: Unknown.
        // 2: Unknown, not encountered in any data.
        // 3: Change position by n number of bytes (signed, so can loop back). Can move to a
        //    different animation entirely! Not yet implemented.
        // 4: A pointer to frame assembly data, and frame duration. A duration of 0 will show the
        //    frame forever.
        // 5: Unknown.
        // 6: Unknown.
        for (_, offset) in offsets.iter().enumerate() {
            data.seek(SeekFrom::Start(*offset)).unwrap();

            let mut anim = SpriteAnim::empty();

            let mut debug_data: Vec<Vec<isize>> = Vec::new();
            loop {
                let op = data.read_u8().unwrap() as isize;
                let mut op_data: Vec<isize> = Vec::new();
                op_data.push(op);

                // Unknown ops.
                if op == 0x00 {
                    op_data.push(data.read_u8().unwrap() as isize);
                }
                else if op == 0x01 {
                    op_data.push(data.read_u8().unwrap() as isize);
                }

                // Goto another offset relative to the current.
                // 0x03 <signed byte>
                //
                // This is used to loop animations back to a previous frame.
                else if op == 0x03 {
                    op_data.push(data.read_u8().unwrap() as isize);
                    debug_data.push(op_data);
                    break;
                }

                // This frame is assembled from sprite tiles.
                // 0x04 <address of assembly data> <duration>
                //
                // The assembly contains:
                // - A byte with number of tiles in this frame
                // ...then any number of tiles:
                // - A signed byte with the x offset of the tile
                // - A signed byte with the y offset of the tile
                // - An unsigned 16-bit integer that contains the SNES VRAM tile index and flags
                //   for this tile
                else if op == 0x04 {
                    let ptr = data.read_u16::<LittleEndian>().unwrap() as u64;
                    let duration = data.read_u8().unwrap() as usize;

                    // Read frame assembly data from the position, but keep track
                    // of the current position so we can return here later.
                    let old_pos = data.stream_position().unwrap();
                    data.seek(SeekFrom::Start(ptr - 0xE000)).unwrap();
                    let tile_count = data.read_u8().unwrap();
                    let mut frame = SpriteAssemblyFrame {
                        tiles: Vec::new(),
                    };
                    for _ in 0..tile_count {
                        let x = data.read_i8().unwrap() as i32;
                        let y = data.read_i8().unwrap() as i32;
                        let mut chip_index = data.read_u16::<LittleEndian>().unwrap() as usize;

                        let mut flags: SpriteAssemblyTileFlags = SpriteAssemblyTileFlags::default();
                        if chip_index & 0x2000 > 0 {
                            flags |= SpriteAssemblyTileFlags::UNKNOWN;
                        }
                        if chip_index & 0x4000 > 0 {
                            flags |= SpriteAssemblyTileFlags::FLIP_X;
                        }
                        if chip_index & 0x8000 > 0 {
                            flags |= SpriteAssemblyTileFlags::FLIP_Y;
                        }

                        chip_index &= 0x1FFF;
                        let src_x = ((chip_index % 32) * 8) as i32;
                        let src_y = ((chip_index / 32) * 16) as i32;

                        assembly.chip_max = std::cmp::max(assembly.chip_max, chip_index);
                        frame.tiles.push(SpriteAssemblyTile {
                            x, y,
                            width: 16, height: 16,
                            src_x, src_y,
                            chip: chip_index,
                            flags,
                        });
                    }
                    assembly.frames.push(frame);

                    // Add the new frame to the current animation.
                    let frame_index = assembly.frames.len() - 1;
                    anim.frames.push(SpriteAnimFrame {
                        sprite_frames: [frame_index, frame_index, frame_index, frame_index],
                        duration: duration as f64 * (1.0 / 60.0),
                    });

                    data.seek(SeekFrom::Start(old_pos)).unwrap();
                    if duration == 0 {
                        break;
                    }
                }

                // Unknown ops.
                else if op == 0x05 {
                    op_data.push(data.read_u8().unwrap() as isize);
                    debug_data.push(op_data);
                    break;
                }
                else if op == 0x06 {
                    op_data.push(data.read_u8().unwrap() as isize);
                    op_data.push(data.read_u8().unwrap() as isize);
                    op_data.push(data.read_u8().unwrap() as isize);
                    op_data.push(data.read_u8().unwrap() as isize);
                    op_data.push(data.read_u8().unwrap() as isize);
                    op_data.push(data.read_u8().unwrap() as isize);
                    op_data.push(data.read_u8().unwrap() as isize);
                }
                else {
                    panic!("Unknown op {}", op);
                }

                debug_data.push(op_data);
            }

            anim_set.anims.push(anim);

            // println!("{:>04}: {:>3}: {:02X?}", offset, index, debug_data);
        }

        // assembly.dump();
        // anim_set.dump();

        (assembly, anim_set)
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
fn parse_sprite_animation_frames(slot_data: &Vec<u8>, start_slot_offset: usize, interval_data: &Vec<u8>, start_interval_offset: usize, anim_index: usize, anim_count: usize) -> Vec<SpriteAnimFrame> {
    let mut frame_index = 0;
    let mut frames: Vec<SpriteAnimFrame> = Vec::new();
    loop {
        // Calculate offsets for each direction.
        let offsets = [
            start_slot_offset + ((anim_count * 0) * 4) + anim_index * 4 + frame_index,
            start_slot_offset + ((anim_count * 1) * 4) + anim_index * 4 + frame_index,
            start_slot_offset + ((anim_count * 2) * 4) + anim_index * 4 + frame_index,
            start_slot_offset + ((anim_count * 3) * 4) + anim_index * 4 + frame_index,
        ];

        // Build a sprite frame from data for all 4 directions.
        let sprite_frames = [
            slot_data[offsets[0]] as usize,
            slot_data[offsets[1]] as usize,
            slot_data[offsets[2]] as usize,
            slot_data[offsets[3]] as usize,
        ];
        if sprite_frames[0] == 0xFF {
            break;
        }

        // A frame duration is measured in 1/60th of a second (or one SNES frame).
        let interval_offset = start_interval_offset + anim_index * 4 + frame_index;
        let interval = interval_data[interval_offset] as usize;
        let duration = interval as f64 * (1.0 / 60.0);

        frames.push(SpriteAnimFrame {
            sprite_frames,
            duration,
        });
        frame_index += 1;
    }

    frames
}

fn parse_pc_sprite_assembly(assembly_index: usize, data: &mut Cursor<Vec<u8>>) -> SpriteAssembly {

    // Number of frames in this assembly.
    let frame_count = data.read_u16::<LittleEndian>().unwrap();
    data.read_u16::<LittleEndian>().unwrap();
    let mut assembly = SpriteAssembly {
        index: assembly_index,
        chip_max: 0,
        frames: vec![SpriteAssemblyFrame::new(); frame_count as usize],
    };

    for frame in assembly.frames.iter_mut() {

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
            let mut flags = SpriteAssemblyTileFlags::default();
            if flags_value & 0x01 != 0 {
                flags |= SpriteAssemblyTileFlags::FLIP_X;
            }
            if flags_value & 0x02 != 0 {
                flags |= SpriteAssemblyTileFlags::UNUSED;
            }
            if weird_bit != 0 {
                flags |= SpriteAssemblyTileFlags::UNKNOWN;
            }

            let src_x = ((chip % 32) * 8) as i32;
            let src_y = ((chip / 32) * 16) as i32;

            frame.tiles.push(SpriteAssemblyTile {
                chip,
                x, y,
                src_x, src_y,
                width: 16,
                height: 16,
                flags,
            });
        }
    }

    assembly
}

fn parse_snes_sprite_assembly(assembly_index: usize, set_count: usize, data: &mut Cursor<Vec<u8>>) -> SpriteAssembly {
    let frame_count = data.get_ref().len() / (set_count * 4 * 10);
    let mut assembly = SpriteAssembly {
        index: assembly_index,
        chip_max: 0,
        frames: vec![SpriteAssemblyFrame::new(); frame_count],
    };

    for frame in assembly.frames.iter_mut() {
        frame.tiles.resize(set_count * 16, SpriteAssemblyTile::default());

        for set in 0..set_count {

            // Upper row.
            for tile in 0..4 {
                frame.tiles[set * 16 + tile * 4 + 0] = parse_snes_sprite_assembly_tile(data, 0, 0);
                frame.tiles[set * 16 + tile * 4 + 1] = parse_snes_sprite_assembly_tile(data, 8, 0);
            }

            // Bottom row.
            for tile in 0..4 {
                frame.tiles[set * 16 + tile * 4 + 2] = parse_snes_sprite_assembly_tile(data, 0, 8);
                frame.tiles[set * 16 + tile * 4 + 3] = parse_snes_sprite_assembly_tile(data, 8, 8);
            }
        }

        // Offsets.
        for set in 0..set_count {
            for tile in 0..4 {
                let ox = data.read_i8().unwrap() as i32;
                let oy = data.read_i8().unwrap() as i32;
                for chip in 0..4 {
                    frame.tiles[set * 16 + tile * 4 + chip].x += ox;
                    frame.tiles[set * 16 + tile * 4 + chip].y += oy;
                }
            }
        }

        // Track highest chip index.
        for tile in frame.tiles.iter() {
            assembly.chip_max = assembly.chip_max.max(tile.chip);
        }
    }

    assembly
}

fn parse_snes_sprite_assembly_tile(data: &mut Cursor<Vec<u8>>, x: i32, y: i32) -> SpriteAssemblyTile {
    let value = data.read_u16::<LittleEndian>().unwrap();

    let chip = (value & 0x3FF) as usize;
    let src_x = ((chip % 16) * 8) as i32;
    let src_y = ((chip / 16) * 8) as i32;

    let mut flags = SpriteAssemblyTileFlags::default();
    if value & 0x4000 != 0 {
        flags |= SpriteAssemblyTileFlags::FLIP_X;
    }
    if value & 0x8000 != 0 {
        flags |= SpriteAssemblyTileFlags::FLIP_Y;
    }

    SpriteAssemblyTile {
        chip,
        x, y,
        src_x, src_y,
        width: 8,
        height: 8,
        flags,
    }
}
