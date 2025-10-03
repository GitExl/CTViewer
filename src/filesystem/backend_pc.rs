use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::path::Path;
use byteorder::{LittleEndian, ReadBytesExt};
use crate::filesystem::backend::FileSystemBackendTrait;
use crate::filesystem::filesystem::{FileSystem};
use crate::filesystem::resourcesbin::ResourcesBin;
use crate::software_renderer::bitmap::Bitmap;
use crate::software_renderer::palette::{Color, Palette};
use crate::util::bmp_reader::Bmp;
use crate::util::lz_decompress::lz_decompress;

#[derive(Debug)]
pub enum FileSystemBackendPcMode {
    FileSystem,
    ResourcesBin,
}

pub struct FileSystemBackendPc {
    base_path: Box<Path>,
    mode: FileSystemBackendPcMode,
    resources: Option<ResourcesBin>
}

impl FileSystemBackendPc {
    pub fn new(base_path: &Box<Path>, mode: FileSystemBackendPcMode) -> Self {
        println!("Using PC data backend with '{}' in {:?} mode.", base_path.display(), mode);

        let resources = if matches!(mode, FileSystemBackendPcMode::ResourcesBin) {
            Some(ResourcesBin::new(base_path))
        } else {
            None
        };

        FileSystemBackendPc {
            base_path: base_path.clone(),
            mode,
            resources,
        }
    }

    fn file_get(&self, filename: &str) -> Cursor<Vec<u8>> {
        match self.resources {
            None => {
                let path = self.base_path.join(Path::new(&filename));
                let mut file = File::open(&path).expect(&format!("Could not open file {}.", filename));
                let mut data = vec![0; file.metadata().unwrap().len() as usize];
                file.read_exact(&mut data).unwrap();

                Cursor::new(data)
            },
            Some(ref res) => {
                res.file_get(filename)
            }
        }
    }

    fn file_exists(&self, filename: &String) -> bool {
        match self.resources {
            None => {
                let path = self.base_path.join(Path::new(&filename));
                path.exists()
            },
            Some(ref res) => {
                res.file_exists(filename)
            },
        }
    }

    fn get_bytes(&self, reader: &mut Cursor<Vec<u8>>, len: Option<usize>, start: Option<usize>) -> Vec<u8> {
        let mut buffer;

        let mut offset = 0;
        if start.is_some() {
            reader.seek(SeekFrom::Start(start.unwrap() as u64)).unwrap();
            offset = start.unwrap();
        }

        if len.is_none() {
            buffer = vec![0u8; reader.get_mut().len() - offset];
        } else {
            buffer = vec![0u8; len.unwrap()];
        }
        reader.read_exact(&mut buffer).unwrap();

        buffer
    }

    fn get_bytes_cursor(&self, reader: &mut Cursor<Vec<u8>>, len: Option<usize>, start: Option<usize>) -> Cursor<Vec<u8>> {
        let buffer = self.get_bytes(reader, len, start);
        Cursor::new(buffer)
    }

    fn get_file_cursor(&self, filename: &String, len: Option<usize>, start: Option<usize>) -> Cursor<Vec<u8>> {
        let mut reader = self.file_get(filename);
        self.get_bytes_cursor(&mut reader, len, start)
    }

    fn get_file_bytes(&self, filename: &String, len: Option<usize>, start: Option<usize>) -> Vec<u8> {
        let mut reader = self.file_get(filename);
        self.get_bytes(&mut reader, len, start)
    }

    fn unpack_4bpp_graphics(&self, data: &Vec<u8>) -> Vec<u8> {
        let mut pixels = vec![0u8; data.len() * 2];

        let mut dest = 0;
        for src in data.iter() {
            pixels[dest] = (src & 0xF0) >> 4;
            dest += 1;
            pixels[dest] = src & 0x0F;
            dest += 1;
        }

        pixels
    }

    fn remove_string_list_keys(&self, strings: Vec<String>) -> Vec<String> {
        strings.iter().map(|x|x.split_once(",").unwrap().1.to_string()).collect()
    }
}

impl FileSystemBackendTrait for FileSystemBackendPc {
    fn get_world_header_data(&self, world_index: usize) -> Cursor<Vec<u8>> {
        let mut reader = self.file_get(&"Game/common/bankc6.bin".to_string());
        reader.seek(SeekFrom::Start(0xFD10 + world_index as u64 * 23)).unwrap();
        self.get_bytes_cursor(&mut reader, Some(23), None)
    }

    fn get_world_map_tile_data(&self, world_map_index: usize) -> Cursor<Vec<u8>> {
        self.get_file_cursor(&format!("Game/world/Map/Map_{:0>4}.dat", world_map_index), None, None)
    }

    fn get_world_map_tile_props_data(&self, world_map_props_index: usize) -> Cursor<Vec<u8>> {
        self.get_file_cursor(&format!("Game/world/Id/Id_{:0>4}.dat", world_map_props_index), None, None)
    }

    fn get_world_tileset12_graphics(&self, chips_index: usize) -> Option<Vec<u8>> {
        let filename = &format!("Game/world/map_bin/cg{}.bin", chips_index);
        if !self.file_exists(filename) {
            return None;
        }

        let mut reader = self.file_get(&filename);
        let data = self.get_bytes(&mut reader, None, Some(4));

        Some(self.unpack_4bpp_graphics(&data))
    }

    fn get_world_tileset3_graphics(&self, chips_index: usize) -> Option<Vec<u8>> {
        let filename = &format!("Game/world/map_bin/cg{}.bin", chips_index);
        if !self.file_exists(filename) {
            return None;
        }

        let mut reader = self.file_get(&filename);
        let data = self.get_bytes(&mut reader, None, Some(4));

        Some(self.unpack_4bpp_graphics(&data))
    }

    fn get_world_tileset12_assembly_data(&self, assembly_index: usize) -> Cursor<Vec<u8>> {
        self.get_file_cursor(&format!("Game/world/Chip/Chip_{:04}.dat", assembly_index), None, None)
    }

    fn get_world_tileset3_assembly_data(&self, assembly_index: usize) -> Cursor<Vec<u8>> {

        // Load tile assembly from bankc6.bin. Why isn't this extracted out into files?
        let mut reader = self.file_get(&"Game/common/bankc6.bin".to_string());
        reader.seek(SeekFrom::Start(0xFF40 + assembly_index as u64 * 3)).unwrap();
        let offset = reader.read_u16::<LittleEndian>().unwrap() as u64;
        reader.seek(SeekFrom::Start(offset)).unwrap();

        // Decompress it.
        let mut assembly_data = vec![0u8; 0x1000];
        let mut assembly = vec![0u8; 0x1000];
        reader.seek(SeekFrom::Start(offset)).unwrap();
        reader.read_exact(&mut assembly_data).unwrap();
        lz_decompress(&assembly_data, &mut assembly, 0);

        Cursor::new(assembly)
    }

    fn get_world_music_data(&self, music_index: usize) -> Cursor<Vec<u8>> {
        self.get_file_cursor(&format!("Game/world/SeId/SeId_{:0>4}.dat", music_index), None, None)
    }

    fn get_world_exit_data(&self, exits_index: usize) -> Cursor<Vec<u8>> {
        self.get_file_cursor(&format!("Game/world/EventTable/EventTable_{:04}.dat", exits_index), None, None)
    }

    fn get_world_exit_names(&self, language: &str) -> Vec<String> {
        let data = self.get_file_cursor(&format!("Localize/{}/msg/w_map.txt", language), None, None);
        self.read_text_string_list(data, None, None)
    }

    fn get_world_names(&self, language: &str) -> Vec<String> {
        let data = self.get_file_cursor(&format!("Localize/{}/msg/w_map.txt", language), None, None);
        self.read_text_string_list(data, Some(106), Some(112))
    }

    fn get_world_sprite_data(&self) -> Cursor<Vec<u8>> {
        self.get_file_cursor(&String::from("Game/common/shapeSeqTbl.bin"), None, None)
    }

    fn get_world_sprite_graphics(&self, world_index: usize, tiles_index: usize) -> Option<Vec<u8>> {

        // Map world bitmap indices to Steam BMP files.
        //
        // These bitmaps start at 0x59A56 in the SNES ROM, with offsets at 0x6FDF0. They are
        // mostly 128x64x4bpp pixels per set, loaded into the top 4 banks of VRAM (0x0000,
        // 0x1000, 0x2000 or 0x3000). Exceptions are the Lavos sprites, which are 128x128 pixels,
        // and the "kodai_break" sprites at 128x96 pixels.
        //
        // Idx | BMP file                | Description                 | VRAM    | Tile
        // ----|-------------------------|-----------------------------|---------|---------
        // 0   | common/worldChara       | Player characters           | 0x2000  | 0x1000
        // 1   | common/silbird          | Epoch                       | 0x2000  | 0x1000
        // 2   | common/lavos            | Lavos                       | 0x0000  | 0x0000
        // 3   | common/blackdream       | Black omen                  | 0x0000  | 0x0000
        // 4   | world/gif/0_wobj0       | 1000 AD ferry, vortex       | 0x0000  | 0x0000
        // 5   | world/gif/0_wobj1       | 1000 AD ferry smoke         | 0x1000  | 0x0800
        // 6   | -                       | Placeholder for PCs         | 0x2000  | 0x1000
        // 7   | world/gif/{world}_wboa  | Epoch, year, birds, etc.    | 0x3000  | 0x1800
        // 8   | world/gif/1_wobj0       | 600 AD magus caste a.o.     | 0x0000  | 0x0000
        // 9   | world/gif/1_wobj1       | 600 AD sunken cave          | 0x1000  | 0x0800
        // A   | world/gif/3_wobj0       | 65M BC dactyl               | 0x0000  | 0x0000
        // B   | world/gif/4_wobj0       | 12000 BC mt. woe            | 0x0000  | 0x0000
        // C   | world/gif/4_wobj1       | 12000 BC mt. woe, blackbird | 0x1000  | 0x0800
        // D   | world/gif/4_kodai_break | Deteriorating zeal          | 0x0000? | 0x0000?
        let world_bitmap = format!("Game/world/gif/{}_wboa.bmp", world_index);
        let name = match tiles_index {
            0 => "Game/common/worldChara.bmp",
            1 => "Game/common/silbird.bmp",
            2 => "Game/common/lavos.bmp",
            3 => "Game/common/blackdream.bmp",
            4 => "Game/world/gif/0_wobj0.bmp",
            5 => "Game/world/gif/0_wobj1.bmp",
            7 => world_bitmap.as_str(),
            8 => "Game/world/gif/1_wobj0.bmp",
            9 => "Game/world/gif/1_wobj1.bmp",
            10 => "Game/world/gif/3_wobj0.bmp",
            11 => "Game/world/gif/4_wobj0.bmp",
            12 => "Game/world/gif/4_wobj1.bmp",
            13 => "Game/world/gif/4_kodai_break.bmp",
            _ => return None,
        };

        let bitmap = Bmp::from_cursor(&mut self.file_get(&name.to_string()));
        Some(bitmap.pixels)
    }

    fn get_world_player_sprite_graphics(&self) -> Option<Vec<u8>> {
        let bmp = Bmp::from_path(Path::new("data/pc_sprites_empty.bmp"));

        Some(bmp.pixels)
    }

    fn get_world_epoch_sprite_graphics(&self) -> Option<Vec<u8>> {
        let bmp = Bmp::from_path(Path::new("data/epoch_sprites_empty.bmp"));

        Some(bmp.pixels)
    }

    fn get_world_palette(&self, world_palette_index: usize) -> Palette {
        let mut data = self.get_file_cursor(&format!("Game/world/plt_bin/plt{}.bin", world_palette_index), None, None);
        data.seek(SeekFrom::Start(2)).unwrap();

        let mut colors = Vec::<Color>::new();
        for _ in 0..256 {
            colors.push(FileSystem::read_color(&mut data));
        }

        Palette::from_colors(&colors)
    }

    fn get_world_palette_anim_data(&self, world_palette_index: usize) -> Cursor<Vec<u8>> {
        self.get_file_cursor(&format!("Game/world/colanim_bin/{}_colanim.bin", world_palette_index), None, None)
    }

    fn get_scene_palette_anim_data(&self) -> (Cursor<Vec<u8>>, Cursor<Vec<u8>>, Cursor<Vec<u8>>) {
        (
            self.get_file_cursor(&"Game/common/PalAnimaAdrs.dat".to_string(), None, Some(4)),
            self.get_file_cursor(&"Game/common/PalAnimaData.dat".to_string(), None, Some(4)),
            self.get_file_cursor(&"Game/common/PaletteAnimeColor.bin".to_string(), None, None),
        )
    }

    fn get_scene_palette(&self, scene_palette_index: usize) -> Palette {
        let mut data = self.get_file_cursor(&format!("Game/field/palette_bin/plt{}.bin", scene_palette_index), None, None);
        data.seek(SeekFrom::Start(2)).unwrap();

        let mut colors = Vec::<Color>::new();
        for _ in 0..256 {
            colors.push(FileSystem::read_color(&mut data));
        }

        Palette::from_colors(&colors)
    }

    fn get_scene_header_data(&self, scene_index: usize) -> Cursor<Vec<u8>> {
        self.get_file_cursor(&format!("Game/field/Mapinfo/mapinfo_{}.dat", scene_index), None, None)
    }

    fn get_scene_map_data(&self, scene_map_index: usize) -> Cursor<Vec<u8>> {
        self.get_file_cursor(&format!("Game/field/MapTable/MapTable_{:0>4}.dat", scene_map_index), None, None)
    }

    fn get_scene_layer_priorities(&self, scene_map_index: usize) -> Cursor<Vec<u8>> {
        self.get_file_cursor(&format!("Game/field/PrioMap/PrioMap{:}.dat", scene_map_index), None, None)
    }

    fn get_scene_tileset_data(&self, tileset_index: usize) -> Cursor<Vec<u8>> {
        self.get_file_cursor(&format!("Game/field/BGSetTable/bgsettable_{}.dat", tileset_index), None, None)
    }

    fn get_scene_tileset3_graphics(&self, chips_index: usize) -> Option<Vec<u8>> {
        let filename = &format!("Game/field/weather_bin/cg{}.bin", chips_index);
        if !self.file_exists(filename) {
            return None;
        }

        let mut reader = self.file_get(&filename);
        let data = self.get_bytes(&mut reader, None, Some(4));

        Some(self.unpack_4bpp_graphics(&data))
    }

    fn get_scene_tileset3_assembly_data(&self, assembly_index: usize) -> Option<Cursor<Vec<u8>>> {
        let filename = &format!("Game/field/ChipTable/ChipTableBg3_{:0>4}.dat", assembly_index);
        if !self.file_exists(filename) {
            return None;
        }

        let mut reader = self.file_get(&filename);
        Some(self.get_bytes_cursor(&mut reader, None, None))
    }

    fn get_scene_tileset12_graphics(&self, chips_index: usize) -> Vec<u8> {
        let data = self.get_file_bytes(&format!("Game/field/map_bin/cg{}.bin", chips_index), None, Some(4));
        self.unpack_4bpp_graphics(&data)
    }

    fn get_scene_tileset12_assembly_data(&self, index_assembly: usize) -> Cursor<Vec<u8>> {
        self.get_file_cursor(&format!("Game/field/ChipTable/ChipTable_{:0>4}.dat", index_assembly), None, None)
    }

    fn get_scene_tileset12_animation_data(&self, chip_anims_index: usize) -> Option<Cursor<Vec<u8>>> {
        let filename = &format!("Game/field/BGAnime/bganimeinfo_{}.dat", chip_anims_index);
        if !self.file_exists(filename) {
            return None;
        }

        let mut reader = self.file_get(&filename);
        Some(self.get_bytes_cursor(&mut reader, None, Some(1)))
    }

    fn get_scene_exit_data(&self, scene_index: usize) -> Cursor<Vec<u8>> {

        // Read offsets to the exits of each scene.
        let mut offsets_data = self.get_file_cursor(&String::from("Game/common/MapJumpOffsetTbl.dat"), None, None);
        let ptr_count = offsets_data.read_u32::<LittleEndian>().unwrap() as usize;
        let mut offsets = vec![0; ptr_count];
        for i in 0..ptr_count {
            offsets[i] = offsets_data.read_u16::<LittleEndian>().unwrap() as usize * 8 + 4;
        }

        let data = self.get_file_cursor(&String::from("Game/common/MapJumpDataTbl.dat"), None, None);
        let len = data.get_ref().len();
        let offset = offsets[scene_index];

        // Determine how many exits there are based on the current and next offset.
        let size;
        if scene_index == offsets.len() - 1 {
            size = len - offset;
        } else {
            size = offsets[scene_index + 1] - offset;
        }

        Cursor::new(data.get_ref()[offset..offset + size].to_vec())
    }

    fn get_scene_names(&self, language: &str) -> Vec<String> {
        let data = self.get_file_cursor(&format!("Localize/{}/msg/debug_map.txt", language), None, None);
        let mut strings = Vec::<String>::new();
        strings.push("".to_string());
        strings.append(&mut self.remove_string_list_keys(self.read_text_string_list(data, None, None)));

        strings
    }

    fn get_scene_treasure_data(&self) -> (Vec<u32>, Cursor<Vec<u8>>) {
        let mut data = self.file_get(&"Game/common/TakaraDataTbl.dat".to_string());
        let mut offsets_data = self.file_get(&"Game/common/TakaraOffsetTbl.dat".to_string());

        let ptr_count = offsets_data.read_u32::<LittleEndian>().unwrap() as usize;
        let mut offsets = vec![0u32; ptr_count];
        for offset in offsets.iter_mut() {
            *offset = offsets_data.read_u16::<LittleEndian>().unwrap() as u32 * 6 + 4;
        }

        // Add a last entry so that the number of chests can be calculated like the SNES version.
        offsets.push(offsets[ptr_count - 1]);

        (
            offsets,
            self.get_bytes_cursor(&mut data, None, None),
        )
    }

    fn get_scene_script_data(&self, scene_script_index: usize) -> Cursor<Vec<u8>> {
        self.get_file_cursor(&format!("Game/field/atel/Atel_{:0>4}.dat", scene_script_index), None, None)
    }

    fn get_sprite_header_data(&self, sprite_index: usize) -> Cursor<Vec<u8>> {
        self.get_file_cursor(&format!("Game/chara/dat/c{:0>3}.dat", sprite_index), None, None)
    }

    fn get_sprite_assembly_data(&self, sprite_assembly_index: usize) -> Cursor<Vec<u8>> {
        self.get_file_cursor(&format!("Game/chara/cell/c{:0>3}.cel", sprite_assembly_index), None, None)
    }

    fn get_sprite_animation_data(&self) -> (Vec<usize>, Cursor<Vec<u8>>, Vec<usize>, Cursor<Vec<u8>>) {
        let mut pointers_slots = Vec::new();
        let mut data_slot_pointers = self.get_file_cursor(&String::from("Game/common/SlotAddressTable.dat"), None, Some(4));
        for _ in 0..201 {
            pointers_slots.push(data_slot_pointers.read_u16::<LittleEndian>().unwrap() as usize);
        }

        let mut pointers_intervals = Vec::new();
        let mut data_interval_pointers = self.get_file_cursor(&String::from("Game/common/IntervalAddressTable.dat"), None, Some(4));
        for _ in 0..201 {
            pointers_intervals.push(data_interval_pointers.read_u16::<LittleEndian>().unwrap() as usize);
        }

        (
            pointers_slots,
            self.get_file_cursor(&String::from("Game/common/SlotAddress.bin"), None, None),
            pointers_intervals,
            self.get_file_cursor(&String::from("Game/common/IntervalAddress.bin"), None, None),
        )
    }

    fn get_sprite_palette(&self, sprite_index: usize) -> Option<Palette> {

        // The PC version stores these in the sprite BMP files, so we get it from the
        // first BMP file instead.
        let filename = format!("Game/chara/bmp/c{:0>3}_0.bmp", sprite_index);
        if !self.file_exists(&filename) {
            return None;
        }

        let mut bmp_reader = self.file_get(&filename);
        let bmp = Bmp::from_cursor(&mut bmp_reader);

        // Normalize the BMP palette into raw RGBA color bytes.
        let mut colors = vec![Color::default(); bmp.palette.len()];
        for (index, color) in bmp.palette.iter().enumerate() {
            colors[index][0] = color[0];
            colors[index][1] = color[1];
            colors[index][2] = color[2];
            colors[index][3] = 0xFF;
        }

        Some(Palette::from_colors(&colors))
    }

    fn get_sprite_graphics(&self, sprite_index: usize, chip_count: usize, _compressed: bool) -> Vec<u8> {
        let bitmap_count = (chip_count as f64 / 512.0).ceil() as usize;
        let mut tile_data = vec![0u8; 256 * bitmap_count * 256];

        for bitmap_index in 0..bitmap_count {
            let filename = format!("Game/chara/bmp/c{:0>3}_{}.bmp", sprite_index, bitmap_index);
            if !self.file_exists(&filename) {
                continue;
            }

            let mut reader = self.file_get(&filename);
            let bmp = Bmp::from_cursor(&mut reader);
            let offset = bitmap_index * 0x10000;
            tile_data[offset..offset + bmp.pixels.len()].copy_from_slice(&bmp.pixels);
        }

        tile_data
    }

    fn get_item_names(&self, language: &str) -> Vec<String> {
        let data = self.get_file_cursor(&format!("Localize/{}/msg/item.txt", language), None, None);
        self.read_text_string_list(data, None, None)
    }

    fn get_textbox_string_table(&self, _address: usize) -> Vec<String> {
        // todo: map to one of the localization files, but how?

        Vec::new()
    }

    fn get_ui_theme_cursor_graphics(&self, _ui_theme_index: usize) -> (Bitmap, Palette) {
        (Bitmap::new(32, 16), Palette::new(16))
    }

    fn get_ui_theme_window_graphics(&self, _ui_theme_index: usize) -> (Bitmap, Palette) {
        (Bitmap::new(32, 48), Palette::new(16))
    }
}
