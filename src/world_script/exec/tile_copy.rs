use crate::map::Map;
use crate::tileset::TileSet;
use crate::world::world_map::{WorldChip, WorldMap};

pub fn exec_tile_copy(map: &mut Map, world_map: &mut WorldMap, tileset: &TileSet, src_layer: usize, src_x: usize, src_y: usize, dest_layer: usize, dest_x: usize, dest_y: usize, width: usize, height: usize) {

    // Copy tiles into intermediate buffer to prevent multiple borrows.
    let layer_src = &map.layers[src_layer];
    let src_len = layer_src.tiles.len();
    let src_tile_width = layer_src.tile_width;
    let mut buffer_tiles = vec![0usize; width * height];
    let mut buffer_chips = vec![WorldChip::default(); width * height * 4];

    for tile_y in 0..height {
        for tile_x in 0..width  {

            // Copy tile.
            let src_tile_x = src_x + tile_x;
            let src_tile_y = src_y + tile_y;
            let src_tile_index = src_tile_x + src_tile_y * src_tile_width as usize;
            if src_tile_index >= src_len {
                continue;
            }
            let dest_tile_index = tile_x + tile_y * width;
            buffer_tiles[dest_tile_index] = layer_src.tiles[src_tile_index];

            // Copy chip properties.
            let src_chip_x = src_tile_x * 2;
            let src_chip_y = src_tile_y * 2;
            let src_chip_index = src_chip_x + src_chip_y * world_map.width as usize;
            let dest_chip_index = (tile_x * 2) + (tile_y * 2) * width * 2;
            buffer_chips[dest_chip_index + 0] = world_map.chips[src_chip_index + 0];
            buffer_chips[dest_chip_index + 1] = world_map.chips[src_chip_index + 1];
            buffer_chips[dest_chip_index + (width * 2) + 0] = world_map.chips[src_chip_index + world_map.width as usize + 0];
            buffer_chips[dest_chip_index + (width * 2) + 1] = world_map.chips[src_chip_index + world_map.width as usize + 1];
        }
    }

    // Copy buffer tiles to destination.
    let layer_dest = &mut map.layers[dest_layer];
    let dest_len = layer_dest.tiles.len();
    let dest_tile_width = layer_dest.tile_width;

    for tile_y in 0..height {
        for tile_x in 0..width  {

            // Copy tile.
            let dest_tile_x = dest_x + tile_x;
            let dest_tile_y = dest_y + tile_y;
            let dest_tile_index = dest_tile_x + dest_tile_y * dest_tile_width as usize;
            if dest_tile_index >= dest_len {
                continue;
            }
            let src_tile_index = tile_x + tile_y * width;
            layer_dest.tiles[dest_tile_index] = buffer_tiles[src_tile_index];

            // Copy chip properties.
            let dest_chip_x = dest_tile_x * 2;
            let dest_chip_y = dest_tile_y * 2;
            let dest_chip_index = dest_chip_x + dest_chip_y * world_map.width as usize;
            let src_chip_index = (tile_x * 2) + (tile_y * 2) * width * 2;
            world_map.chips[dest_chip_index + 0] = buffer_chips[src_chip_index + 0];
            world_map.chips[dest_chip_index + 1] = buffer_chips[src_chip_index + 1];
            world_map.chips[dest_chip_index + world_map.width as usize + 0] = buffer_chips[src_chip_index + (width * 2) + 0];
            world_map.chips[dest_chip_index + world_map.width as usize + 1] = buffer_chips[src_chip_index + (width * 2) + 1];
        }
    }

    layer_dest.assemble_chips(&tileset, dest_x as u32, dest_y as u32, width as u32, height as u32);
}
