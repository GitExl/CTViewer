use std::io::{BufRead, Cursor};
use crate::software_renderer::palette::Palette;

pub trait FileSystemBackendTrait {
    fn read_text_string_list(&self, data: Cursor<Vec<u8>>, start: Option<usize>, end: Option<usize>) -> Vec<String> {
        let mut strings = Vec::<String>::new();
        for line in data.lines().map_while(Result::ok) {
            strings.push(line);
        }

        let real_start = if start.is_some() { start.unwrap() } else { 0 };
        let real_end = if end.is_some() { end.unwrap() } else { strings.len() };

        strings[real_start..real_end].to_vec()
    }

    fn get_world_header_data(&self, world_index: usize) -> Cursor<Vec<u8>>;
    fn get_world_map_tile_data(&self, world_map_index: usize) -> Cursor<Vec<u8>>;
    fn get_world_map_tile_props_data(&self, world_map_props_index: usize) -> Cursor<Vec<u8>>;
    fn get_world_tileset12_graphics(&self, chips_index: usize) -> Option<Vec<u8>>;
    fn get_world_tileset3_graphics(&self, chips_index: usize) -> Option<Vec<u8>>;
    fn get_world_tileset12_assembly_data(&self, assembly_index: usize) -> Cursor<Vec<u8>>;
    fn get_world_tileset3_assembly_data(&self, assembly_index: usize) -> Cursor<Vec<u8>>;
    fn get_world_music_data(&self, music_index: usize) -> Cursor<Vec<u8>>;
    fn get_world_exit_data(&self, exits_index: usize) -> Cursor<Vec<u8>>;
    fn get_world_exit_names(&self, language: &str) -> Vec<String>;
    fn get_world_names(&self, language: &str) -> Vec<String>;
    fn get_world_sprite_data(&self) -> Cursor<Vec<u8>>;
    fn get_world_sprite_graphics(&self, world_index: usize, tiles_index: usize) -> Option<Vec<u8>>;
    fn get_world_player_sprite_graphics(&self) -> Option<Vec<u8>>;
    fn get_world_epoch_sprite_graphics(&self) -> Option<Vec<u8>>;
    fn get_world_palette(&self, world_palette_index: usize) -> Palette;
    fn get_world_palette_anim_data(&self, world_palette_index: usize) -> Cursor<Vec<u8>>;

    fn get_scene_palette_anim_data(&self) -> (Cursor<Vec<u8>>, Cursor<Vec<u8>>, Cursor<Vec<u8>>);
    fn get_scene_palette(&self, scene_palette_index: usize) -> Palette;
    fn get_scene_header_data(&self, scene_index: usize) -> Cursor<Vec<u8>>;
    fn get_scene_map_data(&self, scene_map_index: usize) -> Cursor<Vec<u8>>;
    fn get_scene_layer_priorities(&self, scene_map_index: usize) -> Cursor<Vec<u8>>;
    fn get_scene_tileset_data(&self, tileset_index: usize) -> Cursor<Vec<u8>>;
    fn get_scene_tileset3_graphics(&self, chips_index: usize) -> Option<Vec<u8>>;
    fn get_scene_tileset3_assembly_data(&self, assembly_index: usize) -> Option<Cursor<Vec<u8>>>;
    fn get_scene_tileset12_graphics(&self, chips_index: usize) -> Vec<u8>;
    fn get_scene_tileset12_assembly_data(&self, index_assembly: usize) -> Cursor<Vec<u8>>;
    fn get_scene_tileset12_animation_data(&self, chip_anims_index: usize) -> Option<Cursor<Vec<u8>>>;
    fn get_scene_exit_data(&self, scene_index: usize) -> Cursor<Vec<u8>>;
    fn get_scene_names(&self, language: &str) -> Vec<String>;
    fn get_scene_treasure_data(&self) -> (Vec<u32>, Cursor<Vec<u8>>);
    fn get_scene_script_data(&self, scene_script_index: usize) -> Cursor<Vec<u8>>;

    fn get_sprite_header_data(&self, sprite_index: usize) -> Cursor<Vec<u8>>;
    fn get_sprite_assembly_data(&self, sprite_assembly_index: usize) -> Cursor<Vec<u8>>;
    fn get_sprite_animation_data(&self) -> (Vec<usize>, Cursor<Vec<u8>>, Vec<usize>, Cursor<Vec<u8>>);
    fn get_sprite_palette(&self, sprite_index: usize) -> Option<Palette>;
    fn get_sprite_graphics(&self, sprite_tiles_index: usize, chip_count: usize, compressed: bool) -> Vec<u8>;

    fn get_item_names(&self, language: &str) -> Vec<String>;
}
