use std::io::{Seek, SeekFrom};
use byteorder::LittleEndian;
use byteorder::ReadBytesExt;
use crate::Facing;
use crate::filesystem::filesystem::{FileSystem, ParseMode};
use crate::scene::scene::{Scene, SceneExit, SceneTreasure, ScrollMask};

struct SceneHeader {

    // Index of music track to play.
    music_index: usize,

    // Tileset references.
    tileset_l12_index: usize,
    tileset_l12_assembly_index: usize,
    tileset_l3_index: usize,
    tileset_l3_assembly_index: usize,

    // Palette data references.
    palette_index: usize,
    palette_anims_index: usize,

    // Scene map to use.
    map_index: usize,

    // Tileset chip animation reference.
    chip_anims_index: usize,

    // Script to run.
    script_index: usize,

    // Area to constrain the camera to.
    scroll_mask: ScrollMask,
}

impl FileSystem {

    // Read a scene and related data.
    pub fn read_scene(&self, scene_index: usize) -> Scene {
        let mut data = self.backend.get_scene_header_data(scene_index);

        let mut header = match self.parse_mode {
            ParseMode::Snes => SceneHeader {
                music_index: data.read_u8().unwrap() as usize,
                tileset_l12_index: data.read_u8().unwrap() as usize,
                tileset_l12_assembly_index: 0,
                tileset_l3_index: data.read_u8().unwrap() as usize,
                tileset_l3_assembly_index: 0,
                palette_index: data.read_u8().unwrap() as usize,
                map_index: data.read_u16::<LittleEndian>().unwrap() as usize,
                chip_anims_index: data.read_u8().unwrap() as usize,
                palette_anims_index: data.read_u8().unwrap() as usize,
                script_index: data.read_u16::<LittleEndian>().unwrap() as usize,
                scroll_mask: ScrollMask {
                    left: data.read_u8().unwrap() as isize * 16,
                    top: data.read_u8().unwrap() as isize * 16,
                    right: data.read_u8().unwrap() as isize * 16 + 16,
                    bottom: data.read_u8().unwrap() as isize * 16,
                },
            },
            ParseMode::Pc => SceneHeader {
                music_index: data.read_u16::<LittleEndian>().unwrap() as usize,
                tileset_l12_index: data.read_u16::<LittleEndian>().unwrap() as usize,
                tileset_l12_assembly_index: data.read_u16::<LittleEndian>().unwrap() as usize,
                tileset_l3_index: data.read_u16::<LittleEndian>().unwrap() as usize,
                tileset_l3_assembly_index: 0,
                palette_index: data.read_u16::<LittleEndian>().unwrap() as usize,
                palette_anims_index: data.read_u16::<LittleEndian>().unwrap() as usize,
                map_index: data.read_u16::<LittleEndian>().unwrap() as usize,
                chip_anims_index: data.read_u16::<LittleEndian>().unwrap() as usize,
                script_index: data.read_u32::<LittleEndian>().unwrap() as usize,
                scroll_mask: ScrollMask {
                    left: data.read_i8().unwrap() as isize * 16,
                    top: data.read_i8().unwrap() as isize * 16,
                    right: data.read_i8().unwrap() as isize * 16 + 16,
                    bottom: data.read_i8().unwrap() as isize * 16 + 16,
                },
            },
        };

        if matches!(self.parse_mode, ParseMode::Snes) {
            header.palette_anims_index = header.palette_index;
            header.chip_anims_index = header.tileset_l12_index;
            header.tileset_l12_assembly_index = header.tileset_l12_index;
            header.tileset_l3_assembly_index = header.tileset_l3_index;
        }
        if matches!(self.parse_mode, ParseMode::Pc) {
            header.tileset_l3_assembly_index = scene_index;
        }

        let tileset_l12 = self.read_scene_tileset_layer12(header.tileset_l12_index, header.tileset_l12_assembly_index, header.chip_anims_index);
        let tileset_l3 = self.read_scene_tileset_layer3(header.tileset_l3_index, header.tileset_l3_assembly_index);
        let (scene_map, map) = self.read_scene_map(header.map_index, &tileset_l12, &tileset_l3);
        let palette = self.read_scene_palette(header.palette_index);
        let palette_anims = self.read_palette_anim_set(header.palette_anims_index);
        let exits = self.read_scene_exits(scene_index);
        let treasure = self.read_scene_treasure(scene_index);

        if header.scroll_mask.left == 2048 || header.scroll_mask.left == -2048 {
            header.scroll_mask.left = 0;
            header.scroll_mask.top = 0;
            header.scroll_mask.right = map.layers[0].tile_width as isize * 16;
            header.scroll_mask.bottom = map.layers[0].tile_height as isize * 16;
        }

        Scene {
            index: scene_index,

            music_index: header.music_index,
            script_index: header.script_index,

            map,
            scene_map,
            scroll_mask: header.scroll_mask,
            tileset_l12,
            tileset_l3,
            palette,
            palette_anims,
            exits,
            treasure,
            actors: Vec::new(),
            render_sprites: Vec::new(),
        }
    }

    // Read exits from scenes.
    // These are triggered when the player walks onto them in the scene, and takes them to a
    // new location (world or scene).
    pub fn read_scene_exits(&self, scene_index: usize) -> Vec<SceneExit> {
        let mut data = self.backend.get_scene_exit_data(scene_index);
        let count = match self.parse_mode {
            ParseMode::Pc => data.get_ref().len() / 8,
            ParseMode::Snes => data.get_ref().len() / 7,
        };

        // Read exits.
        let mut exits = Vec::new();
        for exit_index in 0..count {

            let width;
            let height;
            let x;
            let y;
            let facing;
            let mut dest_x;
            let mut dest_y;
            let dest_index;

            match self.parse_mode {
                ParseMode::Pc => {
                    x = data.read_u8().unwrap() as i32 * 16;
                    y = data.read_u8().unwrap() as i32 * 16;
                    let size_bits = data.read_u8().unwrap();
                    let facing_shift = data.read_u8().unwrap();
                    dest_index = data.read_u16::<LittleEndian>().unwrap() as usize;
                    dest_x = data.read_u8().unwrap() as i32 * 8;
                    dest_y = data.read_u8().unwrap() as i32 * 8;

                    let size = (((size_bits & 0x7F) + 1) * 16) as i32;
                    (width, height) = if size_bits & 0x80 > 0 {
                        (16, size)
                    } else {
                        (size, 16)
                    };

                    facing = match facing_shift & 0x3 {
                        0 => Facing::Up,
                        1 => Facing::Down,
                        2 => Facing::Left,
                        3 => Facing::Right,
                        _ => panic!(),
                    };

                    if dest_index >= 0x1F0 && dest_index <= 0x1FF {
                        dest_x *= 8;
                        dest_y *= 8;
                    } else {
                        dest_x *= 16;
                        dest_y *= 16;
                    }

                    // Shift destination if flags are set.
                    if facing_shift & 0x4 > 0 {
                        dest_x -= 8;
                    }
                    if facing_shift & 0x8 > 0 {
                        dest_y -= 8;
                    }
                },

                // The SNES uses 7 bytes and packs the facing and destination offset
                // into the destination bytes.
                ParseMode::Snes => {
                    x = data.read_u8().unwrap() as i32 * 16;
                    y = data.read_u8().unwrap() as i32 * 16;
                    let size_bits = data.read_u8().unwrap();
                    let dest_index_facing = data.read_u16::<LittleEndian>().unwrap() as usize;
                    dest_x = data.read_u8().unwrap() as i32;
                    dest_y = data.read_u8().unwrap() as i32;

                    dest_index = dest_index_facing & 0x1FF;
                    facing = match (dest_index_facing & 0x600) >> 9 {
                        0 => Facing::Up,
                        1 => Facing::Down,
                        2 => Facing::Left,
                        3 => Facing::Right,
                        _ => panic!(),
                    };

                    let size = ((size_bits & 0x7F) + 1) as i32 * 16;
                    (width, height) = if size_bits & 0x80 > 0 {
                        (16, size)
                    } else {
                        (size, 16)
                    };

                    if dest_index >= 0x1F0 && dest_index <= 0x1FF {
                        dest_x *= 8;
                        dest_y *= 8;
                    } else {
                        dest_x *= 16;
                        dest_y *= 16;
                    }

                    // Shift destination if flags are set.
                    if dest_index_facing & 0x800 > 0 {
                        dest_x -= 8;
                    }
                    if dest_index_facing & 0x1000 > 0 {
                        dest_y -= 8;
                    }
                },
            };

            exits.push(SceneExit {
                index: exit_index,
                x, y,
                width, height,
                destination_index: dest_index,
                destination_x: dest_x,
                destination_y: dest_y,
                facing,
            });
        }

        exits
    }

    pub fn read_scene_treasure(&self, scene_index: usize) -> Vec<SceneTreasure> {
        let (pointers, mut data) = self.backend.get_scene_treasure_data();
        let treasure_count = match self.parse_mode {
            ParseMode::Pc => (pointers[scene_index + 1] - pointers[scene_index]) / 6,
            ParseMode::Snes => (pointers[scene_index + 1] - pointers[scene_index]) / 4,
        };

        let mut treasure: Vec<SceneTreasure> = Vec::new();
        data.seek(SeekFrom::Start(pointers[scene_index] as u64)).unwrap();
        for index in 0..treasure_count {
            let id = format!("{}_{}", scene_index, index);
            let x = data.read_u8().unwrap();
            let y = data.read_u8().unwrap();
            let contents = data.read_u16::<LittleEndian>().unwrap();

            // Pointer to other location chest data.
            if x == 0 && y == 0 {
                return self.read_scene_treasure(contents as usize);
            }

            let item = match self.parse_mode {
                ParseMode::Snes => parse_snes_treasure(id, x, y, contents),
                ParseMode::Pc => parse_pc_treasure(id, x, y, contents),
            };
            treasure.push(item);

            if matches!(self.parse_mode, ParseMode::Pc) {
                data.read_u16::<LittleEndian>().unwrap();
            }
        }

        treasure
    }
}

fn parse_pc_treasure(id: String, x: u8, y: u8, contents: u16) -> SceneTreasure {

    let mut gold = 0;
    let mut item = 0;

    // Gold.
    if contents & 0x8000 > 0 {
        gold = (contents & 0x7FFF) as u32 * 2;
    }
    // Items.
    else if contents & 0xFF00 == 0x5000 {
        item = (contents & 0x1FF) as usize + 302;
    // Consumables.
    } else if contents & 0xFF00 == 0x4000 {
        item = (contents & 0x1FF) as usize + 259;
    // Accessories.
    } else if contents & 0xFF00 == 0x3000 {
        item = (contents & 0x1FF) as usize + 200;
    // Helmet.
    } else if contents & 0xFF00 == 0x2000 {
        item = (contents & 0x1FF) as usize + 161;
    // Armor.
    } else if contents & 0xFF00 == 0x1000 {
        item = (contents & 0x1FF) as usize + 111;
    // Weapon.
    } else if contents & 0xFF00 == 0 {
        item = (contents & 0x1FF) as usize;
    }

    SceneTreasure {
        id,
        tile_x: x as i32,
        tile_y: y as i32,
        gold,
        item,
    }
}

fn parse_snes_treasure(id: String, x: u8, y: u8, contents: u16) -> SceneTreasure {
    let mut gold = 0;
    let mut item = 0;

    if contents & 0x8000 > 0 {
        gold = (contents & 0x7FFF) as u32 * 2;
    } else if contents & 0x4000 == 0 {
        item = (contents & 0x1FF) as usize
    }

    SceneTreasure {
        id,
        tile_x: x as i32,
        tile_y: y as i32,
        gold,
        item,
    }
}
