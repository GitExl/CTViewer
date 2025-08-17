use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::path::Path;
use byteorder::{LittleEndian, ReadBytesExt};
use crate::filesystem::backend::FileSystemBackendTrait;
use crate::filesystem::filesystem::{FileSystem};
use crate::filesystem::text_decoder::TextDecoder;
use crate::software_renderer::palette::{Color, Palette};
use crate::util::lz_decompress::lz_decompress;

#[derive(Default, Clone)]
struct Entry {
    address: usize,
    length: usize,
}

pub struct FileSystemBackendSnes {
    data: Vec<u8>,
    text_decoder: TextDecoder,

    world_header_entries: Vec<Entry>,
    world_map_tile_entries: Vec<Entry>,
    world_map_props_entries: Vec<Entry>,
    world_tileset_entries: Vec<Entry>,
    world_tileset12_assembly_entries: Vec<Entry>,
    world_tileset3_assembly_entries: Vec<Entry>,
    world_music_entries: Vec<Entry>,
    world_exit_entries: Vec<Entry>,
    world_sprite_entries: Vec<Entry>,
    world_palette_entries: Vec<Entry>,

    scene_map_entries: Vec<Entry>,
    scene_tileset_entries: Vec<Entry>,
    scene_tileset12_assembly_entries: Vec<Entry>,
    scene_tileset12_animation_entries: Vec<Entry>,
    scene_tileset3_assembly_entries: Vec<Entry>,
    scene_exit_entries: Vec<Entry>,

    sprite_entries: Vec<Entry>,
    sprite_assembly_entries: Vec<Entry>,
    sprite_anim_frame_entries: Vec<Entry>,
    sprite_anim_duration_entries: Vec<Entry>,
    sprite_palette_entries: Vec<Entry>,
    sprite_anim_frame_data_entry: Entry,
    sprite_anim_duration_data_entry: Entry,
}

impl FileSystemBackendSnes {
    pub fn new(rom_path: &Path) -> Self {
        let mut file = File::open(&rom_path).expect(&format!("Could not open ROM '{}'.", rom_path.display()));
        if file.metadata().unwrap().len() > 6 * 1024 * 1024 {
            panic!("ROM is too large.");
        }

        let mut data = vec![0u8; file.metadata().unwrap().len() as usize];
        file.read_exact(&mut data).unwrap();

        // Find ROM header and strip file format header if present.
        let mut header = 0;
        let id = str::from_utf8(&data[0x101C0..0x101D5]);
        if id.is_ok() {
            if id.unwrap() == "CHRONO TRIGGER       " {
                header = 0x101C0;
                data = data[0x200..].to_vec();
            }
        } else {
            let id = str::from_utf8(&data[0xFFC0..0xFFD5]);
            if id.is_ok() {
                if id.unwrap() == "CHRONO TRIGGER       " {
                    header = 0xFFC0;
                }
            }
        }
        if header == 0 {
            panic!("ROM is not a Chrono Trigger ROM.");
        }

        println!("Using SNES data backend from {}.", rom_path.display());

        let text_decoder = TextDecoder::from_cursor(&mut Cursor::new(data[0x1EFA00..0x1EFF00].to_vec()), 128, 0xFA00);

        let world_header_entries = get_local_entries(&data, 0x6FD00, 8, 0x6FDF0, false);
        let world_map_tile_entries = get_entries(&data, 0x6FF20, 8, 0x6C7F7);
        let world_map_props_entries = get_entries(&data, 0x6FF80, 8, 0x6DA76);
        let world_tileset_entries = get_entries(&data, 0x6FE20, 42, 0x59A56);
        let world_tileset12_assembly_entries = get_entries(&data, 0x6FF00, 6, 0x66A39);
        let world_tileset3_assembly_entries = get_entries(&data, 0x6FF40, 6, 0x6D410);
        let world_music_entries = get_entries(&data, 0x6FFA0, 3, 0x6DD06);
        let world_exit_entries = get_entries(&data, 0x6FFC0, 8, 0x629EC);
        let world_sprite_entries = get_entries(&data, 0x6FDF0, 16, 0x61E73);
        let world_palette_entries = get_entries(&data, 0x6FEA0, 32, 0x3E000);

        let scene_map_entries = get_entries(&data, 0x361E00, 202, 0x35EE02);
        let scene_tileset_entries = get_entries(&data, 0x362220, 204, 0x3D8E64);
        let scene_tileset12_assembly_entries = get_entries(&data, 0x362100, 64, 0x2F7F41);
        let scene_tileset12_animation_entries = get_relative_entries(&data, 0x3DF290, 64, 0x3DF310, 0x3DF9CC);
        let scene_tileset3_assembly_entries = get_entries(&data, 0x3621C0, 19, 0x2FB168);
        let scene_exit_entries = get_local_entries(&data, 0x250000, 512, 0x251A44, true);

        let sprite_pointer_entries = get_local_entries(&data, 0x24FFE0, 7, 0x24FFEE, false);
        let sprite_entries = get_entries(&data, sprite_pointer_entries[0].address, 248, 0x21DDB2);
        let sprite_assembly_entries = get_entries(&data, sprite_pointer_entries[1].address, 231, 0x23F8C0);
        let sprite_anim_frame_entries = get_local_entries(&data, sprite_pointer_entries[2].address, 194, 0x24A800, false);
        let sprite_anim_duration_entries = get_local_entries(&data, sprite_pointer_entries[3].address, 194, 0x24F000, false);
        let sprite_palette_entries = get_local_entries(&data, sprite_pointer_entries[4].address, 253, 0x243000, false);
        let sprite_anim_frame_data_entry = Entry {
            address: sprite_pointer_entries[5].address,
            length: 30208,
        };
        let sprite_anim_duration_data_entry = Entry {
            address: sprite_pointer_entries[6].address,
            length: 9312,
        };

        FileSystemBackendSnes {
            data,

            text_decoder,

            world_header_entries,
            world_map_tile_entries,
            world_map_props_entries,
            world_tileset_entries,
            world_tileset12_assembly_entries,
            world_tileset3_assembly_entries,
            world_music_entries,
            world_exit_entries,
            world_sprite_entries,
            world_palette_entries,

            scene_map_entries,
            scene_tileset_entries,
            scene_tileset12_assembly_entries,
            scene_tileset12_animation_entries,
            scene_tileset3_assembly_entries,
            scene_exit_entries,

            sprite_entries,
            sprite_assembly_entries,
            sprite_anim_frame_entries,
            sprite_anim_duration_entries,
            sprite_palette_entries,
            sprite_anim_frame_data_entry,
            sprite_anim_duration_data_entry,
        }
    }

    fn get_bytes_cursor(&self, offset: usize, len: usize) -> Cursor<Vec<u8>> {
        Cursor::new(self.data[offset..(offset + len)].to_vec())
    }

    fn get_bytes_lz(&self, offset: usize) -> Vec<u8> {
        let mut decompressed_data = vec![0u8; 0x10000];
        let decompressed_len = lz_decompress(&self.data, &mut decompressed_data, offset);
        decompressed_data.resize(decompressed_len, 0);

        decompressed_data
    }

    fn get_bytes_cursor_lz(&self, offset: usize) -> Cursor<Vec<u8>> {
        let decompressed_data = self.get_bytes_lz(offset);

        Cursor::new(decompressed_data)
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

    fn convert_planar_chips_to_linear(&self, data: Vec<u8>, bitplanes: usize) -> Vec<u8> {
        let chip_count = data.len() / (bitplanes * 8);
        let height = (chip_count as f64 / 16.0).ceil() as usize * 8;
        let mut pixels = vec![0u8; 128 * height];

        let mut src_byte: usize = 0;
        for chip in 0..chip_count {
            let chip_x = chip % 16;
            let chip_y = chip / 16;

            let mut dest = (chip_y * 8) * 128 + (chip_x * 8);
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

                dest += 120;
                src_byte += 2;
            }

            if bitplanes == 4 {
                let mut dest = (chip_y * 8) * 128 + (chip_x * 8);
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

                    dest += 120;
                    src_byte += 2;
                }
            }
        }

        pixels
    }

    fn read_string_list(&self, pointers_start: usize, count: usize) -> Vec<String> {
        let segment = pointers_start & 0xFF0000;
        let segment_data = self.data[segment..segment + 0x10000].to_vec();
        let mut cursor = Cursor::new(segment_data);

        let entries = get_relative_entries(&self.data, pointers_start, count, 0, 0x10000);
        let mut strings = Vec::<String>::with_capacity(count);
        for entry in entries.iter() {
            cursor.seek(SeekFrom::Start(entry.address as u64)).unwrap();
            strings.push(self.text_decoder.decode_huffman_string(&mut cursor));
        }

        strings
    }
}

impl FileSystemBackendTrait for FileSystemBackendSnes {
    fn get_world_header_data(&self, world_index: usize) -> Cursor<Vec<u8>> {
        self.get_bytes_cursor(self.world_header_entries[world_index].address, self.world_header_entries[world_index].length)
    }

    fn get_world_map_tile_data(&self, world_map_index: usize) -> Cursor<Vec<u8>> {
        self.get_bytes_cursor_lz(self.world_map_tile_entries[world_map_index].address)
    }

    fn get_world_map_tile_props_data(&self, world_map_props_index: usize) -> Cursor<Vec<u8>> {
        self.get_bytes_cursor_lz(self.world_map_props_entries[world_map_props_index].address)
    }

    fn get_world_tileset12_graphics(&self, chips_index: usize) -> Option<Vec<u8>> {
        if self.world_tileset_entries[chips_index].address == 0 {
            return None;
        }

        let data = self.get_bytes_lz(self.world_tileset_entries[chips_index].address);
        let pixels = self.convert_planar_chips_to_linear(data, 4);
        Some(pixels)
    }

    fn get_world_tileset3_graphics(&self, chips_index: usize) -> Option<Vec<u8>> {
        if self.world_tileset_entries[chips_index].address == 0 {
            return None;
        }

        let data = self.get_bytes_lz(self.world_tileset_entries[chips_index].address);
        let pixels = self.convert_planar_chips_to_linear(data, 2);
        Some(pixels)
    }

    fn get_world_tileset12_assembly_data(&self, assembly_index: usize) -> Cursor<Vec<u8>> {
        self.get_bytes_cursor_lz(self.world_tileset12_assembly_entries[assembly_index].address)
    }

    fn get_world_tileset3_assembly_data(&self, assembly_index: usize) -> Cursor<Vec<u8>> {
        self.get_bytes_cursor_lz(self.world_tileset3_assembly_entries[assembly_index].address)
    }

    fn get_world_music_data(&self, music_index: usize) -> Cursor<Vec<u8>> {
        self.get_bytes_cursor_lz(self.world_music_entries[music_index].address)
    }

    fn get_world_exit_data(&self, exits_index: usize) -> Cursor<Vec<u8>> {
        self.get_bytes_cursor_lz(self.world_exit_entries[exits_index].address)
    }

    fn get_world_exit_names(&self, _language: &str) -> Vec<String> {
        self.read_string_list(0x6F400, 112)
    }

    fn get_world_names(&self, _language: &str) -> Vec<String> {
        self.read_string_list(0x6F4D4, 6)
    }

    fn get_world_sprite_data(&self) -> Cursor<Vec<u8>> {
        self.get_bytes_cursor(0x3E000, 8192)
    }

    fn get_world_sprite_graphics(&self, _world_index: usize, tiles_index: usize) -> Option<Vec<u8>> {
        if self.world_sprite_entries[tiles_index].address == 0 {
            return None;
        }

        let data = self.get_bytes_lz(self.world_sprite_entries[tiles_index].address);
        let pixels = self.convert_planar_chips_to_linear(data, 4);
        Some(pixels)
    }

    fn get_world_player_sprite_graphics(&self) -> Option<Vec<u8>> {
        self.get_world_sprite_graphics(0, 0)
    }

    fn get_world_epoch_sprite_graphics(&self) -> Option<Vec<u8>> {
        self.get_world_sprite_graphics(0, 1)
    }

    fn get_world_palette(&self, world_palette_index: usize) -> Palette {
        let data = self.get_bytes_cursor_lz(self.world_palette_entries[world_palette_index].address);
        self.read_palette(data, 0, 16, 8, 0, 0)
    }

    fn get_world_palette_anim_data(&self, world_palette_index: usize) -> Cursor<Vec<u8>> {
        self.get_bytes_cursor_lz(self.world_palette_entries[world_palette_index].address)
    }

    fn get_scene_palette_anim_data(&self) -> (Cursor<Vec<u8>>, Cursor<Vec<u8>>, Cursor<Vec<u8>>) {
        (
            self.get_bytes_cursor(0x3DF9C7, 0xB6),
            self.get_bytes_cursor(0x3DFA77, 0x56E),
            self.get_bytes_cursor(0x367380, 0xC80),
        )
    }

    fn get_scene_palette(&self, scene_palette_index: usize) -> Palette {
        let data = self.get_bytes_cursor(0x3624C0 + scene_palette_index * 210, 210);
        self.read_palette(data, 16, 15, 7, 1, 0)
    }

    fn get_scene_header_data(&self, scene_index: usize) -> Cursor<Vec<u8>> {
        self.get_bytes_cursor(0x360000 + scene_index * 14, 14)
    }

    fn get_scene_map_data(&self, scene_map_index: usize) -> Cursor<Vec<u8>> {
        self.get_bytes_cursor_lz(self.scene_map_entries[scene_map_index].address)
    }

    fn get_scene_layer_priorities(&self, _scene_map_index: usize) -> Cursor<Vec<u8>> {
        Cursor::new([3u8, 1, 2, 2].to_vec())
    }

    fn get_scene_tileset_data(&self, tileset_index: usize) -> Cursor<Vec<u8>> {
        self.get_bytes_cursor(0x361C00 + tileset_index * 8, 8)
    }

    fn get_scene_tileset3_graphics(&self, chips_index: usize) -> Option<Vec<u8>> {
        if self.scene_tileset_entries[chips_index].address == 0 {
            return None;
        }
        let data = self.get_bytes_lz(self.scene_tileset_entries[chips_index].address);
        Some(self.convert_planar_chips_to_linear(data, 2))
    }

    fn get_scene_tileset3_assembly_data(&self, assembly_index: usize) -> Option<Cursor<Vec<u8>>> {
        if self.scene_tileset3_assembly_entries[assembly_index].address == 0 {
            return None;
        }
        Some(self.get_bytes_cursor_lz(self.scene_tileset3_assembly_entries[assembly_index].address))
    }

    fn get_scene_tileset12_graphics(&self, chips_index: usize) -> Vec<u8> {
        let data = self.get_bytes_lz(self.scene_tileset_entries[chips_index].address);
        self.convert_planar_chips_to_linear(data, 4)
    }

    fn get_scene_tileset12_assembly_data(&self, index_assembly: usize) -> Cursor<Vec<u8>> {
        self.get_bytes_cursor_lz(self.scene_tileset12_assembly_entries[index_assembly].address)
    }

    fn get_scene_tileset12_animation_data(&self, chip_anims_index: usize) -> Option<Cursor<Vec<u8>>> {
        if self.scene_tileset12_animation_entries[chip_anims_index].address == 0 {
            return None;
        }

        Some(self.get_bytes_cursor(self.scene_tileset12_animation_entries[chip_anims_index].address, self.scene_tileset12_animation_entries[chip_anims_index].length))
    }

    fn get_scene_exit_data(&self, scene_index: usize) -> Cursor<Vec<u8>> {
        self.get_bytes_cursor(self.scene_exit_entries[scene_index].address, self.scene_exit_entries[scene_index].length)
    }

    fn get_scene_names(&self, _language: &str) -> Vec<String> {
        let mut reader = File::open(&"data/snes_na_scenes.txt").unwrap();
        let mut buffer = vec![0u8; reader.metadata().unwrap().len() as usize];
        reader.read_exact(&mut buffer).unwrap();

        self.read_text_string_list(Cursor::new(buffer), None, None)
    }

    fn get_scene_treasure_data(&self) -> (Vec<u32>, Cursor<Vec<u8>>) {
        let mut pointer_data = self.get_bytes_cursor(0x35F000, 0x402);
        let mut pointers = vec![0u32; 513];
        for pointer in pointers.iter_mut() {
            *pointer = pointer_data.read_u16::<LittleEndian>().unwrap() as u32 - 0xF402;
        }

        (
            pointers,
            self.get_bytes_cursor(0x35F402, 0x3E4),
        )
    }

    fn get_sprite_header_data(&self, sprite_index: usize) -> Cursor<Vec<u8>> {

        // Regular PC/NPC sprites.
        if sprite_index < 0x100 {
            return self.get_bytes_cursor(0x24F000 + sprite_index * 5, 5);
        }

        // Enemy sprites.
        self.get_bytes_cursor(0x24F600 + (sprite_index - 256) * 10, 10)
    }

    fn get_sprite_assembly_data(&self, sprite_assembly_index: usize) -> Cursor<Vec<u8>> {
        self.get_bytes_cursor(self.sprite_assembly_entries[sprite_assembly_index].address, self.sprite_assembly_entries[sprite_assembly_index].length)
    }

    fn get_sprite_animation_data(&self) -> (Vec<usize>, Cursor<Vec<u8>>, Vec<usize>, Cursor<Vec<u8>>) {
        let offset = self.sprite_anim_frame_entries[0].address;
        let mut frames_pointers = Vec::with_capacity(self.sprite_anim_frame_entries.len());
        for entry in self.sprite_anim_frame_entries.iter() {
            frames_pointers.push(entry.address - offset);
        }

        let offset = self.sprite_anim_duration_entries[0].address;
        let mut duration_pointers = Vec::with_capacity(self.sprite_anim_frame_entries.len());
        for entry in self.sprite_anim_duration_entries.iter() {
            duration_pointers.push(entry.address - offset);
        }

        (
            frames_pointers,
            self.get_bytes_cursor(self.sprite_anim_frame_data_entry.address, self.sprite_anim_frame_data_entry.length),
            duration_pointers,
            self.get_bytes_cursor(self.sprite_anim_duration_data_entry.address, self.sprite_anim_duration_data_entry.length),
        )
    }

    fn get_sprite_palette(&self, sprite_index: usize) -> Option<Palette> {
        let entry = &self.sprite_palette_entries[sprite_index];
        if entry.address == 0 {
            return None;
        }
        let data = self.get_bytes_cursor(entry.address, 24);

        Some(self.read_palette(data, 0, 12, 1, 1, 3))
    }

    fn get_sprite_graphics(&self, sprite_index: usize, _chip_count: usize, compressed: bool) -> Vec<u8> {
        let entry = &self.sprite_entries[sprite_index];
        let data = if compressed {
            self.get_bytes_lz(entry.address)
        } else {
            self.data[entry.address..entry.address + entry.length].to_vec()
        };

        self.convert_planar_chips_to_linear(data, 4)
    }

    fn get_item_names(&self, _language: &str) -> Vec<String> {
        let mut data = self.get_bytes_cursor(0xC0B5E, 0xA71);

        let mut strings = Vec::<String>::new();
        for _ in 0..data.get_ref().len() / 11 {
            let mut item = vec![0u8; 11];
            data.read_exact(&mut item).unwrap();
            strings.push(self.text_decoder.decode_mapped_string(item));
        }

        // todo: remove first character, this is either a space or item type symbol we do not
        //  need (yet).

        strings
    }
}

fn get_bytes_cursor(data: &Vec<u8>, offset: usize, len: usize) -> Cursor<Vec<u8>> {
    Cursor::new(data[offset..(offset + len)].to_vec())
}

fn calculate_entry_sizes(entries: &mut Vec<Entry>, end_address: usize) {
    for j in 0..entries.len() {
        let start = entries[j].address;
        let mut found_end = end_address;
        for i in 0..entries.len() {
            if entries[i].address > start && entries[i].address < found_end {
                found_end = entries[i].address;
            }
        }
        entries[j].length = found_end - start;
    }
}

fn calculate_ordered_entry_sizes(entries: &mut Vec<Entry>, end_address: usize) {
    for j in 0..entries.len() - 1 {
        let next = if j == entries.len() - 1 {
            end_address
        } else {
            entries[j + 1].address
        };
        entries[j].length = next - entries[j].address;
    }
}

fn get_entries(data: &Vec<u8>, pointers_address: usize, pointer_count: usize, last_entry_end_address: usize) -> Vec<Entry> {
    let mut entries = Vec::<Entry>::with_capacity(pointer_count);

    let mut pointer_data = get_bytes_cursor(&data, pointers_address, pointer_count * 3);
    for _ in 0..pointer_count {
        let pointer = pointer_data.read_u16::<LittleEndian>().unwrap() as usize;
        let bank = pointer_data.read_u8().unwrap() as usize;
        if bank == 0 && pointer == 0 {
            entries.push(Entry {
                address: 0,
                length: 0,
            });
        } else {
            entries.push(Entry {
                address: 0x10000 * (bank - 0xC0) + pointer,
                length: 0,
            });
        }
    }

    calculate_entry_sizes(&mut entries, last_entry_end_address);

    entries
}

fn get_local_entries(data: &Vec<u8>, pointers_address: usize, pointer_count: usize, last_entry_end_address: usize, ordered: bool) -> Vec<Entry> {
    let mut entries = Vec::<Entry>::with_capacity(pointer_count);

    let page_start = pointers_address & 0xFF0000;
    let mut pointer_data = get_bytes_cursor(&data, pointers_address, pointer_count * 2);
    for _ in 0..pointer_count {
        entries.push(Entry {
            address: page_start + pointer_data.read_u16::<LittleEndian>().unwrap() as usize,
            length: 0,
        });
    }

    if ordered {
        calculate_ordered_entry_sizes(&mut entries, last_entry_end_address);
    } else {
        calculate_entry_sizes(&mut entries, last_entry_end_address);
    }

    entries
}

fn get_relative_entries(data: &Vec<u8>, pointers_address: usize, pointer_count: usize, relative_to: usize, last_entry_end_address: usize) -> Vec<Entry> {
    let mut entries = Vec::<Entry>::with_capacity(pointer_count);

    let mut pointer_data = get_bytes_cursor(&data, pointers_address, pointer_count * 2);
    for _ in 0..pointer_count {
        entries.push(Entry {
            address: pointer_data.read_u16::<LittleEndian>().unwrap() as usize + relative_to,
            length: 0,
        });
    }

    calculate_entry_sizes(&mut entries, last_entry_end_address);

    entries
}
