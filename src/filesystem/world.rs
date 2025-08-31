use std::io::Seek;
use std::io::SeekFrom;

use byteorder::LittleEndian;
use byteorder::ReadBytesExt;
use crate::destination::{Destination, Facing};
use crate::filesystem::filesystem::{FileSystem, ParseMode};

use crate::world::world::WorldExit;
use crate::world::world::ScriptedWorldExit;
use crate::world::world::World;

#[derive(Default)]
struct WorldHeader {
    // Layer 1, 2 and 3 chip graphics.
    pub chips_l12: [usize; 8],
    pub chips_l3: [usize; 2],

    // Palette index for chip graphics and palette animations.
    pub palette_index: usize,
    pub palette_anim_index: usize,

    // Sprite chip graphics sets.
    pub sprite_sets: [usize; 4],

    // Layer 1 & 2 tile assembly.
    pub assembly_l12: usize,

    // Map chips and chip properties index.
    pub map_index: usize,
    pub map_props_index: usize,

    // Music data.
    pub music_props_index: usize,

    // Layer 3 tile assembly.
    pub assembly_l3: usize,

    // Exit data.
    pub exits_index: usize,

    // Script data.
    pub script_index: usize,
}

#[derive(Default)]
struct ExitData {
    // X Y coordinates and availability flags.
    pub x: u8,
    pub y: u8,

    // Index into names for this exit.
    pub name_index: u8,

    // Destination scene index and facing data.
    pub scene_index: u16,

    // Where in the scene the player starts.
    pub scene_facing: u8,
    pub scene_tile_x: u8,
    pub scene_tile_y: u8,
}

impl FileSystem {

    pub fn read_world(&self, index: usize) -> World {
        let mut header = WorldHeader::default();
        let mut data = self.backend.get_world_header_data(index);

        for chip_l12 in header.chips_l12.iter_mut() {
            *chip_l12 = data.read_u8().unwrap() as usize;
        }
        for chip_l3 in header.chips_l3.iter_mut() {
            *chip_l3 = data.read_u8().unwrap() as usize;
        }

        header.palette_index = data.read_u8().unwrap() as usize;
        header.palette_anim_index = data.read_u8().unwrap() as usize;

        for sprite_set in header.sprite_sets.iter_mut() {
            *sprite_set = data.read_u8().unwrap() as usize;
        }

        header.assembly_l12 = data.read_u8().unwrap() as usize;
        header.map_index = data.read_u8().unwrap() as usize;
        header.map_props_index = data.read_u8().unwrap() as usize;
        header.music_props_index = data.read_u8().unwrap() as usize;
        header.assembly_l3 = data.read_u8().unwrap() as usize;
        header.exits_index = data.read_u8().unwrap() as usize;
        header.script_index = data.read_u8().unwrap() as usize;

        if matches!(self.parse_mode, ParseMode::Pc) {
            header.palette_anim_index = index;
        }

        let tileset_l12 = self.read_world_tileset_layer12(header.chips_l12, header.assembly_l12);
        let tileset_l3 = self.read_world_tileset_layer3(header.chips_l3[0], header.assembly_l3);
        let (world_map, map) = self.read_world_map(header.map_index, header.map_props_index, header.music_props_index, &tileset_l12, &tileset_l3);
        let palette = self.read_world_palette(header.palette_index);
        let palette_anim = self.read_world_palette_anim_data(header.palette_anim_index);
        let (exits, scripted_exits, script_offsets) = self.read_world_exits(header.script_index);

        World {
            index,

            script: header.script_index,

            tileset_l12,
            tileset_l3,

            palette,
            palette_anim,
            palette_anim_index: 0,
            palette_anim_timer: 0.0,

            sprite_graphics: header.sprite_sets,
            map,
            world_map,
            exits,
            scripted_exits,
            script_offsets,

            actors: Vec::new(),
        }
    }

    // Read world exit data.
    fn read_world_exits(&self, exits_index: usize) -> (Vec<WorldExit>, Vec<ScriptedWorldExit>, Vec<usize>) {
        let mut data = self.backend.get_world_exit_data(exits_index);

        // Exits to other locations.
        let mut exits = Vec::<WorldExit>::new();
        let exit_count = data.read_u8().unwrap() as usize;
        for exit_index in 0..exit_count {
            let mut exit_data = ExitData::default();


            // The full 16 bits are used by the PC version. The SNES version has facing
            // data in the last 7 bits.
            match self.parse_mode {
                ParseMode::Snes => {
                    exit_data.x = data.read_u8().unwrap();
                    exit_data.y = data.read_u8().unwrap();
                    exit_data.name_index = data.read_u8().unwrap();
                    exit_data.scene_index = data.read_u16::<LittleEndian>().unwrap();
                    exit_data.scene_tile_x = data.read_u8().unwrap();
                    exit_data.scene_tile_y = data.read_u8().unwrap();
                },
                ParseMode::Pc => {
                    exit_data.x = data.read_u8().unwrap();
                    exit_data.y = data.read_u8().unwrap();
                    exit_data.name_index = data.read_u8().unwrap();
                    exit_data.scene_index = data.read_u16::<LittleEndian>().unwrap();
                    exit_data.scene_facing = data.read_u8().unwrap();
                    exit_data.scene_tile_x = data.read_u8().unwrap();
                    exit_data.scene_tile_y = data.read_u8().unwrap();
                }
            }

            let tile_x = exit_data.x & 0x7F;
            let is_available = exit_data.x & 0x80 > 0;
            let tile_y = exit_data.y & 0x3F;
            let unknown = (exit_data.y & 0xC0) as u32;

            let scene_index;
            let facing;
            let shift_left;
            let shift_up;
            match self.parse_mode {
                ParseMode::Pc => {
                    scene_index = exit_data.scene_index as usize;
                    facing = (exit_data.scene_facing & 0x6) >> 1;
                    shift_left = exit_data.scene_facing & 0x8 > 0;
                    shift_up = exit_data.scene_facing & 0x10 > 0;
                },
                ParseMode::Snes => {
                    scene_index = (exit_data.scene_index & 0x1FF) as usize;
                    facing = ((exit_data.scene_index & 0x600) >> 9) as u8;
                    shift_left = exit_data.scene_index & 0x800 > 0;
                    shift_up = exit_data.scene_index & 0x1000 > 0;
                },
            };

            let destination = Destination::Scene {
                index: scene_index,
                x: (exit_data.scene_tile_x as i32 * 16) - if shift_left { 8 } else { 0 },
                y: (exit_data.scene_tile_y as i32 * 16) - if shift_up { 8 } else { 0 },
                facing: match facing {
                    0 => Facing::Up,
                    1 => Facing::Down,
                    2 => Facing::Left,
                    3 => Facing::Right,
                    _ => panic!("Unknown world exit facing."),
                },
            };

            exits.push(WorldExit {
                index: exit_index,

                x: tile_x as i32 * 16,
                y: tile_y as i32 * 16 - 8,
                is_available,
                name_index: exit_data.name_index as usize,

                destination,
                unknown,
            });
        }

        // Scripted exits. These are associated with a world script address, purpose still unknown.
        // For example, the Vortex Point in 1000 AD uses this.
        let mut scripted_exits = Vec::<ScriptedWorldExit>::new();
        let scripted_exit_count = data.read_u8().unwrap() as usize;
        for scripted_exit_index in 0..scripted_exit_count {
            let x = data.read_u8().unwrap() as usize;
            let y = data.read_u8().unwrap() as usize;
            let script_offset_index = data.read_u8().unwrap() as usize;

            // End at null entry.
            if x == 0 && y == 0 && script_offset_index == 0 {
                break;
            }

            scripted_exits.push(ScriptedWorldExit {
                index: scripted_exit_index,
                x: (x * 16) as i32,
                y: (y * 16) as i32 - 8,
                script_offset_index,
            });
        }

        // Unknown data, always 0x000001.
        let unknown_count = data.read_u8().unwrap() as i64;
        data.seek(SeekFrom::Current(unknown_count * 3)).unwrap();

        // World script offsets. The world script can refer to these.
        let mut script_offsets = Vec::<usize>::new();
        let script_offset_count = data.read_u8().unwrap() as usize;
        for _ in 0..script_offset_count {
            script_offsets.push(data.read_u16::<LittleEndian>().unwrap() as usize);
        }

        (exits, scripted_exits, script_offsets)
    }
}
