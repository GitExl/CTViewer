use std::io::{BufRead, Cursor};
use crate::filesystem::filesystem::FileSystem;
use crate::software_renderer::bitmap::Bitmap;
use crate::software_renderer::palette::{Color, Palette};

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

    fn read_palette(&self, mut data: Cursor<Vec<u8>>, skip: usize, set_size: usize, set_count: usize, set_start: usize, set_pad: usize) -> Palette {
        let mut colors = Vec::<Color>::new();
        for _ in 0..skip {
            colors.push([0, 0, 0, 0xFF]);
        }

        for _ in 0..set_count {
            for _ in 0..set_start {
                colors.push([0, 0, 0, 0xFF]);
            }

            for _ in 0..set_size {
                colors.push(FileSystem::read_color(&mut data));
            }

            for _ in 0..set_pad {
                colors.push([0, 0, 0, 0xFF]);
            }
        }

        Palette::from_colors(&colors)
    }

    fn convert_planar_chips_to_linear(&self, data: Vec<u8>, width: usize, bitplanes: usize) -> Vec<u8> {
        let chip_count = data.len() / (bitplanes * 8);
        let chips_per_row = width / 8;
        let height = (chip_count as f64 / chips_per_row as f64).ceil() as usize * 8;
        let mut pixels = vec![0u8; width * height];

        let mut src_byte: usize = 0;
        for chip in 0..chip_count {
            let chip_x = chip % chips_per_row;
            let chip_y = chip / chips_per_row;

            let mut dest = (chip_y * 8) * width + (chip_x * 8);
            for _ in 0..8 {

                let mut bit = 0b10000000;
                for _ in 0..8 {
                    if data[src_byte + 0] & bit != 0 {
                        pixels[dest] |= 1;
                    }
                    if data[src_byte + 1] & bit != 0 {
                        pixels[dest] |= 2;
                    }

                    dest += 1;
                    bit >>= 1;
                }

                dest += width - 8;
                src_byte += 2;
            }

            if bitplanes == 4 {
                let mut dest = (chip_y * 8) * width + (chip_x * 8);
                for _ in 0..8 {

                    let mut bit = 0b10000000;
                    for _ in 0..8 {
                        if data[src_byte + 0] & bit != 0 {
                            pixels[dest] |= 4;
                        }
                        if data[src_byte + 1] & bit != 0 {
                            pixels[dest] |= 8;
                        }

                        dest += 1;
                        bit >>= 1;
                    }

                    dest += width - 8;
                    src_byte += 2;
                }
            }
        }

        pixels
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

    fn get_textbox_string_table(&self, address: usize, language: &str) -> Vec<String>;

    fn get_ui_theme_cursor_graphics(&self) -> (Bitmap, Palette);
    fn get_ui_theme_window_graphics(&self, ui_theme_index: usize) -> (Bitmap, Palette);
}
