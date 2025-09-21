use std::io::Cursor;
use std::io::Read;

use byteorder::ReadBytesExt;

use crate::filesystem::filesystem::FileSystem;
use crate::map::EffectFlags;
use crate::map::LayerScrollMode;
use crate::map::Map;
use crate::map::MapLayer;
use crate::map::ScreenFlags;
use crate::scene::scene_map::{SceneMap, SceneMoveDirection, ScenePropLayer, SceneTileCollision, SceneTileFlags, SceneTileProps};
use crate::sprites::sprite_renderer::SpritePriority;
use crate::tileset::TileSet;
use crate::util::vec2df64::Vec2Df64;
use crate::world::world_map::{WorldChip, WorldChipFlags, WorldMap};

struct SceneMapHeader {
    // A nibble for the size of layer 1 and 2.
    layer12_size: u8,

    // Layer 3 size nibble and layer 2 & 3 scroll flags.
    bits: u8,

    // Layer 2 & 3 scrolling speed.
    scroll_l2: u8,
    scroll_l3: u8,

    // SNES layer blend flags.
    screen_flags: u8,
    effect_flags: u8,
}

impl FileSystem {

    // Load a world map.
    pub fn read_world_map(&self, index: usize, index_props: usize, index_music: usize, tileset_l12: &TileSet, tileset_l3: &TileSet) -> (WorldMap, Map) {
        let mut data_map = self.backend.get_world_map_tile_data(index);

        let mut layer_1 = MapLayer::new(192, 128);
        let mut layer_2 = MapLayer::new(192, 128);
        let mut layer_3 = MapLayer::new(64, 32);

        // todo fix hardcoded layer animation. Where is this stored or set? World script calling 0x22F0F?
        let mut screen_flags = ScreenFlags::SCREEN_L1_MAIN | ScreenFlags::SCREEN_L2_MAIN | ScreenFlags::SCREEN_L3_SUB | ScreenFlags::SCREEN_SPR_MAIN;
        let effect_flags = EffectFlags::EFFECT_L1 | EffectFlags::EFFECT_L2 | EffectFlags::EFFECT_SPR | EffectFlags::EFFECT_HALF_INTENSITY;
        if index == 0 {
            layer_3.scroll_speed = Vec2Df64::new(-16.0, 8.0);
        } else if index == 1 {
            layer_3.scroll_speed = Vec2Df64::new(-16.0, 8.0);
        } else if index == 2 {
            screen_flags.remove(ScreenFlags::SCREEN_L3_SUB);
            screen_flags.insert(ScreenFlags::SCREEN_L3_MAIN);
        } else if index == 3 {
            layer_3.scroll_speed = Vec2Df64::new(-16.0, 8.0);
        } else if index == 4 {
            screen_flags.remove(ScreenFlags::SCREEN_L3_SUB);
        } else if index == 5 {
            layer_3.scroll_speed = Vec2Df64::new(-16.0, 8.0);
        } else if index == 6 {
            layer_3.scroll_speed = Vec2Df64::new(-16.0, 8.0);
        } else if index == 7 {
            layer_1.scroll_speed = Vec2Df64::new(16.0, 0.0);
        }

        // Read 2x2 chip map tiles and assemble chips from them.
        read_map_layer_tiles(&mut layer_1, &mut data_map, 0);
        read_map_layer_tiles(&mut layer_2, &mut data_map, 256);
        assemble_map_layer(&mut layer_1, &tileset_l12);
        assemble_map_layer(&mut layer_2, &tileset_l12);

        // Assemble layer 3 from the top and bottom halves of the layer 3 tileset.
        for x in 0..16 {
            for y in 0..16 {
                let tile_index = (x + y * layer_3.tile_width) as usize;
                layer_3.tiles[tile_index] = (x + y * 16) as usize;
            }
        }
        for x in 16..32 {
            for y in 0..16 {
                let tile_index = (x + y * layer_3.tile_width) as usize;
                layer_3.tiles[tile_index] = (((x - 16) + y * 16) + 256) as usize;
            }
        }
        assemble_map_layer(&mut layer_3, &tileset_l3);

        // Read tile properties. Each tile in the tileset has fixed properties associated with it.
        let mut chips: Vec<WorldChip> = vec![WorldChip::default(); layer_1.chips.len()];
        let mut data_props = self.backend.get_world_map_tile_props_data(index_props);
        read_world_tile_props(&mut data_props, &layer_2, &mut chips);

        // Read music for each map chip.
        // Each byte contains 2 values that indicate what music should play when the player is on a given
        // tile. The music "tiles" are 8x16 in size. Here we expand them into 8x8 chips for the map to
        // make things easier in the rest of the code.
        let music_data = self.read_world_music_data(index_music, layer_2.tile_width, layer_2.tile_height);
        let chip_width = layer_2.chip_width;
        for (index, music) in music_data.iter().enumerate() {
            let tile_x = (index * 2) as u32 % layer_2.tile_width;
            let tile_y = (index * 2) as u32 / layer_2.tile_width;
            let chip_x = tile_x * 2;
            let chip_y = tile_y * 2;
            let dest_index = (chip_y * chip_width + chip_x) as usize;

            let value = ((music & 0xF0) >> 4) as usize;
            chips[dest_index + 0].music = value;
            chips[dest_index + 1].music = value;
            chips[dest_index + 0 + chip_width as usize].music = value;
            chips[dest_index + 1 + chip_width as usize].music = value;

            let value = (music & 0x0F) as usize;
            chips[dest_index + 2].music = value;
            chips[dest_index + 3].music = value;
            chips[dest_index + 2 + chip_width as usize].music = value;
            chips[dest_index + 3 + chip_width as usize].music = value;
        }

        (
            WorldMap {
                index,
                width: layer_1.chip_width,
                height: layer_1.chip_height,
                chips,
            },
            Map {
                index,
                effect_flags,
                screen_flags,
                layers: [layer_1, layer_2, layer_3],
                layer_priorities: [3, 2, 2, 1],
            }
        )
    }

    // Read a scene map.
    pub fn read_scene_map(&self, index: usize, tileset_l12: &TileSet, tileset_l3: &TileSet) -> (SceneMap, Map) {
        let mut data = self.backend.get_scene_map_data(index);

        // Read layer data.
        let header = SceneMapHeader {
            layer12_size: data.read_u8().unwrap(),
            bits: data.read_u8().unwrap(),
            scroll_l2: data.read_u8().unwrap(),
            scroll_l3: data.read_u8().unwrap(),
            screen_flags: data.read_u8().unwrap(),
            effect_flags: data.read_u8().unwrap(),
        };
        let scroll_bits = (header.bits & 0x70) >> 4;

        // Read layer 1 tiles.
        let width_l1 = (((header.layer12_size >> 0) & 0x3) * 16 + 16) as u32;
        let height_l1 = (((header.layer12_size >> 2) & 0x3) * 16 + 16) as u32;
        let mut layer_1 = MapLayer::new(width_l1 * 2, height_l1 * 2);
        read_map_layer_tiles(&mut layer_1, &mut data, 0);

        // Read layer 2 tiles.
        let width_l2 = (((header.layer12_size >> 4) & 0x3) * 16 + 16) as u32;
        let height_l2 = (((header.layer12_size >> 6) & 0x3) * 16 + 16) as u32;
        let mut layer_2 = MapLayer::new(width_l2 * 2, height_l2 * 2);
        read_map_layer_tiles(&mut layer_2, &mut data, 0);

        // Read layer 3 tiles if the layer is enabled.
        let width_l3 = (((header.bits >> 0) & 0x3) * 16 + 16) as u32;
        let height_l3 = (((header.bits >> 2) & 0x3) * 16 + 16) as u32;
        let mut layer_3 = MapLayer::new(width_l3 * 2, height_l3 * 2);
        if (header.bits & 0x80) > 0 {
            read_map_layer_tiles(&mut layer_3, &mut data, 0);
        }

        // Read tile properties.
        let scene_width = std::cmp::max(width_l1, width_l2);
        let scene_height = std::cmp::max(height_l1, height_l2);
        let scene_tile_props = read_scene_map_tile_props(scene_width, scene_height, &mut data);

        // Post-process map tiles.
        // If L*_TILE_ADD is set, the tiles refer to the upper bank (index + 256).
        for y in 0..height_l1 {
            for x in 0..width_l1 {
                let prop_index = (y * scene_width + x) as usize;
                let prop = &scene_tile_props[prop_index];
                if prop.flags.contains(SceneTileFlags::L1_TILE_ADD) {
                    let tile_index = (y * width_l1 + x) as usize;
                    layer_1.tiles[tile_index] |= 256;
                }
            }
        }
        for y in 0..height_l2 {
            for x in 0..width_l2 {
                let prop_index = (y * scene_width + x) as usize;
                let prop = &scene_tile_props[prop_index];
                if prop.flags.contains(SceneTileFlags::L2_TILE_ADD) {
                    let tile_index = (y * width_l2 + x) as usize;
                    layer_2.tiles[tile_index] |= 256;
                }
            }
        }

        // Build map chips from a scene map and tilesets.
        // Map layer tiles refer directly to chips, scene map tiles are 2x2 chip references.
        assemble_map_layer(&mut layer_1, &tileset_l12);
        assemble_map_layer(&mut layer_2, &tileset_l12);
        assemble_map_layer(&mut layer_3, &tileset_l3);

        // Set layer 2 scrolling properties.
        let (scroll_speed_x, scroll_speed_y) = decode_scene_layer_scroll_speed(header.scroll_l2);
        let scroll_mode;
        if scroll_speed_x != 0.0 || scroll_speed_y != 0.0 {
            scroll_mode = LayerScrollMode::IgnoreCamera;
        } else {
            scroll_mode = match scroll_bits {
                0x01 => LayerScrollMode::IgnoreCamera,
                0x03 => LayerScrollMode::IgnoreCamera,
                0x04 => LayerScrollMode::Parallax,
                _ => LayerScrollMode::Normal,
            };
        }
        layer_2.scroll_mode = scroll_mode;
        layer_2.scroll_speed = Vec2Df64::new(scroll_speed_x, scroll_speed_y);

        // Set layer 3 scrolling properties.
        let (scroll_speed_x, scroll_speed_y) = decode_scene_layer_scroll_speed(header.scroll_l3);
        let scroll_mode = match scroll_bits {
            0x02 => LayerScrollMode::IgnoreCamera,
            _ => LayerScrollMode::Normal,
        };
        layer_3.scroll_mode = scroll_mode;
        layer_3.scroll_speed = Vec2Df64::new(scroll_speed_x, scroll_speed_y);

        // Read some unknown layer priority data per map.
        // These are unique to the PC version, so might be related to how it renders maps, where it
        // might emulate SNES behaviour only to be able to render them efficiently.
        let mut prio_data = self.backend.get_scene_layer_priorities(index);
        let mut layer_priorities = [0u8; 4];
        prio_data.read_exact(&mut layer_priorities).unwrap();

        (
            SceneMap {
                index,
                props: ScenePropLayer {
                    width: scene_width,
                    height: scene_height,
                    props: scene_tile_props,
                },
            },
            Map {
                index,
                screen_flags: ScreenFlags::from_bits_retain(header.screen_flags as u32),
                effect_flags: EffectFlags::from_bits_retain(header.effect_flags as u32),
                layer_priorities,
                layers: [layer_1, layer_2, layer_3],
            },
        )
    }

    // Read music data.
    fn read_world_music_data(&self, index_music: usize, tile_width: u32, tile_height: u32) -> Vec<u8> {
        let mut data = self.backend.get_world_music_data(index_music);
        let len = ((tile_width * tile_height) / 2) as usize;
        let mut exact_data = vec![0u8; len];
        data.read_exact(&mut exact_data).unwrap();

        exact_data
    }
}

// Decode layer scroll speed bits.
// The returned values are measured in pixels per second.
fn decode_scene_layer_scroll_speed(bits: u8) -> (f64, f64) {
    let mapping = [
        0.0, 3.75, 7.50, 15.0,
        30.0, 60.0, 120.0, 240.0,
        -0.0, -3.75, -7.50, -15.0,
        -30.0, -60.0, -120.0, -240.0
    ];
    (
        mapping[(bits & 0x0F) as usize],
        mapping[((bits & 0xF0) >> 4) as usize],
    )
}

// Assemble map layer chips from a tileset's tiles.
fn assemble_map_layer(layer: &mut MapLayer, tileset: &TileSet) {

    // Convert each tile into 2x2 chips.
    for (index, tile_index) in layer.tiles.iter().enumerate() {
        let src_x = index as u32 % layer.tile_width;
        let src_y = index as u32 / layer.tile_width;
        let dest_x = src_x * 2;
        let dest_y = src_y * 2;
        let dest_index = (dest_y * layer.chip_width + dest_x) as usize;

        if *tile_index >= tileset.tiles.len() {
            continue;
        }
        let tile = &tileset.tiles[*tile_index];

        layer.chips[dest_index + 0].clone_from(&tile.corners[0]);
        layer.chips[dest_index + 1].clone_from(&tile.corners[1]);
        layer.chips[dest_index + layer.chip_width as usize + 0].clone_from(&tile.corners[2]);
        layer.chips[dest_index + layer.chip_width as usize + 1].clone_from(&tile.corners[3]);
    }
}

// Read raw map tile data.
fn read_map_layer_tiles(layer: &mut MapLayer, data: &mut Cursor<Vec<u8>>, tile_offset: usize) {
    let mut tiles_raw = vec![0u8; layer.tiles.len()];
    data.read_exact(&mut tiles_raw).unwrap();

    for (index, tile) in layer.tiles.iter_mut().enumerate() {
        *tile = tiles_raw[index] as usize + tile_offset;
    }
}

// Read tile properties for a world map.
fn read_world_tile_props(data: &mut Cursor<Vec<u8>>, layer: &MapLayer, world_chips: &mut Vec<WorldChip>) {
    let mut props_raw = vec![0u8; 0x200];
    data.read_exact(&mut props_raw).unwrap();

    for (index, map_tile) in layer.tiles.iter().enumerate() {
        let src_x = index as u32 % layer.tile_width;
        let src_y = index as u32 / layer.tile_width;
        let dest_x = src_x * 2;
        let dest_y = src_y * 2;
        let dest_index = (dest_y * layer.chip_width + dest_x) as usize;

        // Properties are loaded for the tiles in layer 2, whose tiles start at 256.
        let src_index = (map_tile - 256) * 2;

        // The game stores each property per tileset tile, but that is more complex than we need.
        // So the tile properties are decoded from the raw data and assigned to the map chips.
        for corner in 0..4 {

            // Get a reference to chip in the world map.
            let world_chip = match corner {
                0 => &mut world_chips[dest_index + 0],
                1 => &mut world_chips[dest_index + 1],
                2 => &mut world_chips[dest_index + 0 + layer.chip_width as usize],
                3 => &mut world_chips[dest_index + 1 + layer.chip_width as usize],
                _ => continue,
            };

            // Decode the tile properties into flags.
            let value = match corner {
                0 => (props_raw[src_index + 0] & 0xF0) >> 4,
                1 => (props_raw[src_index + 0] & 0x0F) >> 0,
                2 => (props_raw[src_index + 1] & 0xF0) >> 4,
                3 => (props_raw[src_index + 1] & 0x0F) >> 0,
                _ => continue,
            };
            if value == 4 {
                world_chip.flags |= WorldChipFlags::HAS_EXIT;
            }
            world_chip.flags |= match value {
                1 => WorldChipFlags::BLOCK_WALK,
                2 => WorldChipFlags::BLOCK_HOVER,
                3 => WorldChipFlags::BLOCK_FLYING,
                _ => continue,
            }
        }
    }
}

// Read Scene map tile properties.
fn read_scene_map_tile_props(width: u32, height: u32, data: &mut Cursor<Vec<u8>>) -> Vec<SceneTileProps> {

    // Properties are stored with basic RLE compression. Each prop is 3 bytes.
    let mut tile_props = Vec::<SceneTileProps>::new();
    loop {
        let mut props_raw = [0u8; 3];
        match data.read_exact(&mut props_raw) {
            Err(_) => break,
            Ok(v) => v,
        }

        let props = parse_scene_tile_props(props_raw);

        // Duplicate n times.
        if props.flags.contains(SceneTileFlags::RLE_COMPRESSED) {
            let repeat = data.read_u8().unwrap() as u32;
            let repeat = if repeat > 0 { repeat } else { 256 };
            for _ in 0..repeat {
                tile_props.push(props.clone());
            }

        // Single use.
        } else {
            tile_props.push(props);
        }
    }

    // Sometimes the tile count does not match the tile count of the largest layer, so pad
    // or trim them here.
    let count_expected = (width * height) as usize;
    if tile_props.len() < count_expected {
        println!("Warning: tile properties are too short, padding");
        for _ in 0..count_expected - tile_props.len() {
            tile_props.push(SceneTileProps::default());
        }
    } else if tile_props.len() > count_expected {
        println!("Warning: tile properties are too long, truncating");
        tile_props.truncate(count_expected);
    }

    tile_props
}

// Parse 3 bytes worth of scene tile property data.
fn parse_scene_tile_props(data: [u8; 3]) -> SceneTileProps {
    let mut sprite_priority_top = SpritePriority::BelowL2AboveL1;
    let mut sprite_priority_bottom = SpritePriority::BelowL2AboveL1;

    let mut flags = SceneTileFlags::default();
    if data[0] & 0x01 != 0 {
        flags |= SceneTileFlags::L1_TILE_ADD;
    }
    if data[0] & 0x02 != 0 {
        flags |= SceneTileFlags::L2_TILE_ADD;
    }
    if data[0] & 0x80 != 0 {
        flags |= SceneTileFlags::RLE_COMPRESSED;
    }

    if data[1] & 0x10 != 0 {
        flags |= SceneTileFlags::DOOR_TRIGGER;
    }
    if data[1] & 0x20 != 0 {
        flags |= SceneTileFlags::UNKNOWN_1;
    }
    if data[1] & 0x40 != 0 {
        sprite_priority_top = SpritePriority::AboveAll;
    }
    if data[1] & 0x80 != 0 {
        flags |= SceneTileFlags::NPC_COLLISION_BATTLE;
    }

    if data[2] & 0x04 != 0 {
        flags |= SceneTileFlags::COLLISION_IGNORE_Z;
    }
    if data[2] & 0x08 != 0 {
        flags |= SceneTileFlags::COLLISION_INVERTED;
    }
    if data[2] & 0x10 != 0 {
        flags |= SceneTileFlags::UNKNOWN_2;
    }
    if data[2] & 0x20 != 0 {
        flags |= SceneTileFlags::Z_NEUTRAL;
    }
    if data[2] & 0x40 != 0 {
        sprite_priority_bottom = SpritePriority::AboveAll;
    }
    if data[2] & 0x80 != 0 {
        flags |= SceneTileFlags::NPC_COLLISION;
    }

    SceneTileProps {
        flags,
        sprite_priority_top,
        sprite_priority_bottom,
        z_plane: (data[2] & 0x3) as u32,
        move_speed: ((data[1] >> 2) & 0x03) as u32,
        move_direction: match data[1] & 0x03 {
            0 => SceneMoveDirection::North,
            1 => SceneMoveDirection::South,
            2 => SceneMoveDirection::East,
            3 => SceneMoveDirection::West,
            _ => panic!(),
        },
        collision: match (data[0] >> 2) & 0x1F {
            0 => SceneTileCollision::None,
            1 => SceneTileCollision::Full,
            2 => SceneTileCollision::Corner45NW,
            3 => SceneTileCollision::Corner45NE,
            4 => SceneTileCollision::Corner45SW,
            5 => SceneTileCollision::Corner45SE,
            6 => SceneTileCollision::Corner30NW,
            7 => SceneTileCollision::Corner30NE,
            8 => SceneTileCollision::Corner30SW,
            9 => SceneTileCollision::Corner30SE,
            10 => SceneTileCollision::Corner22NW,
            11 => SceneTileCollision::Corner22NE,
            12 => SceneTileCollision::Corner22SW,
            13 => SceneTileCollision::Corner22SE,
            14 => SceneTileCollision::Corner75NW,
            15 => SceneTileCollision::Corner75NE,
            16 => SceneTileCollision::Corner75SW,
            17 => SceneTileCollision::Corner75SE,
            18 => SceneTileCollision::Corner75NWDup,
            19 => SceneTileCollision::Corner75NEDup,
            20 => SceneTileCollision::Corner75SWDup,
            21 => SceneTileCollision::Corner75SEDup,
            22 => SceneTileCollision::StairsSWNE,
            23 => SceneTileCollision::StairsSENW,
            24 => SceneTileCollision::LeftHalf,
            25 => SceneTileCollision::TopHalf,
            26 => SceneTileCollision::SW,
            27 => SceneTileCollision::SE,
            28 => SceneTileCollision::NE,
            29 => SceneTileCollision::NW,
            30 => SceneTileCollision::Ladder,
            _ => SceneTileCollision::Invalid,
        },
    }
}
