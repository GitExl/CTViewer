use std::io::Cursor;
use std::io::SeekFrom;
use std::io::Seek;

use byteorder::LittleEndian;
use byteorder::ReadBytesExt;

use crate::filesystem::filesystem::FileSystem;
use crate::game_palette::GamePalette;
use crate::palette_anim::PaletteAnim;
use crate::palette_anim::PaletteAnimSet;
use crate::palette_anim::PaletteAnimType;
use crate::software_renderer::palette::Color;
use crate::software_renderer::palette::Palette;

impl FileSystem {

    // Read a world tile and sprite palette.
    pub fn read_world_palette(&self, index: usize) -> GamePalette {
        GamePalette {
            index,
            palette: self.backend.get_world_palette(index),
        }
    }

    // Read a scene tile and sprite palette. Sprite colors are unused by the PC version. Instead,
    // the palettes in sprite BMP files are used.
    pub fn read_scene_palette(&self, index: usize) -> GamePalette {
        GamePalette {
            index,
            palette: self.backend.get_scene_palette(index),
        }
    }

    // Each world has palette data for tile and sprite palette animations.
    // The first 4 sets of 32 bytes are the animated tile colors. The destination color index is
    // different per world, perhaps the world scripts change this manually? Or it is stored
    // elsewhere or hardcoded somewhere.
    // The remaining palette data can be used by objects from world scripts. "Command04" in
    // Temporal Flux copies palette data to the current object's palette.
    pub fn read_world_palette_anim_data(&self, palette_anim_index: usize) -> GamePalette {
        let mut data = self.backend.get_world_palette_anim_data(palette_anim_index);

        // Read all data as 2-byte colors.
        let count = data.get_ref().len() / 2;
        let mut colors = Vec::<Color>::new();
        for _ in 0..count {
            colors.push(FileSystem::read_color(&mut data));
        }

        GamePalette {
            index: palette_anim_index,
            palette: Palette::from_colors(&colors),
        }
    }

    // Read a set of palette animations.
    // Palette animations change colors in an existing palette according to predefined methods.
    pub fn read_palette_anim_set(&self, index: usize) -> PaletteAnimSet {
        let mut set = PaletteAnimSet {
            index,
            anims: Vec::new(),
        };

        let (
            mut address_data,
            mut anim_data,
            mut color_data,
        ) = self.backend.get_scene_palette_anim_data();

        // Find the right address of the descriptor data for this set.
        let anim_count = address_data.get_ref().len() / 2;
        if index >= anim_count {
            panic!("Palette animation index is out of bounds.");
        }
        address_data.seek(SeekFrom::Start((index * 2) as u64)).unwrap();
        let offset = address_data.read_u16::<LittleEndian>().unwrap() as u64;

        anim_data.seek(SeekFrom::Start(offset)).unwrap();
        let mut anim_index = 0;
        loop {

            // Hitting a 0 indicates the end of this set.
            let id = anim_data.read_u8().unwrap();
            if id == 0 {
                break;
            }

            // 0x80 = predefined sequence
            // 0x30 = forward cycle
            // 0x20 = backward cycle
            // 0x10 = linear sequence
            // We treat both sequences as if they are predefined, then later on
            // define the frames for the linear sequence.
            let anim_type = match id & 0xF0 {
                0x80 => PaletteAnimType::Sequence,
                0x30 => PaletteAnimType::CycleForward,
                0x20 => PaletteAnimType::CycleBackward,
                0x10 => PaletteAnimType::Sequence,

                // Some definitions are broken (like the Palace/Ocean Palace set 87).
                // So if the type is invalid, try again with the next byte.
                _ => continue,
            };

            // Palette animations start at a color index, and modify n colors.
            let mut anim = PaletteAnim {
                index: anim_index,
                anim_type,
                color_index: anim_data.read_u8().unwrap() as usize,
                color_count: anim_data.read_u8().unwrap() as usize,
                delay: (anim_data.read_u8().unwrap() as f64) * (1.0 / 60.0),
                frames: Vec::new(),
                colors: Vec::new(),
                current_frame: 0,
                timer: 0.0,
            };

            // Some animation types define the exact colors of each sequence.
            if matches!(anim_type, PaletteAnimType::Sequence) {

                // Use the first 2 bytes of the SNES address as the address and map it into the
                // PalAnimaData.dat file.
                let address = anim_data.read_u16::<LittleEndian>().unwrap() as u64 - 0x7380;
                let _ = anim_data.read_u8();

                // Read predefined frames or generate them if it is a linear sequence. Generating
                // linear sequences simplifies the palette animation code itself.
                // Each frame is an index into the set of predefined colors that are loaded later.
                let frame_count = id & 0x0F;
                anim.frames = vec![0; frame_count as usize + 1];
                if id & 0xF0 == 0x80 {
                    for value in anim.frames.iter_mut() {
                        *value = anim_data.read_u8().unwrap() as usize;
                    }
                } else {
                    for (index, value) in anim.frames.iter_mut().enumerate() {
                        *value = index;
                    }
                }

                // Find the maximum color set number so that we can only load color data used by
                // this sequence.
                let mut set_count = 0;
                for set_index in &anim.frames {
                    set_count = std::cmp::max(*set_index, set_count);
                }
                let total_color_count = (set_count + 1) * anim.color_count;

                // Read all color set data for this sequence.
                color_data.seek(SeekFrom::Start(address)).unwrap();
                for _ in 0..total_color_count {
                    anim.colors.push(FileSystem::read_color(&mut color_data));
                }
            }

            set.anims.push(anim);
            anim_index += 1;
        }

        set
    }

    // Read an SNES 5 bits per component color value.
    pub fn read_color(reader: &mut Cursor<Vec<u8>>) -> Color {
        let data = reader.read_u16::<LittleEndian>().unwrap();
        [
            (((data >> 0)  & 0x1F) as f64 * (255.0 / 31.0)).round() as u8,
            (((data >> 5)  & 0x1F) as f64 * (255.0 / 31.0)).round() as u8,
            (((data >> 10) & 0x1F) as f64 * (255.0 / 31.0)).round() as u8,
            0xFF
        ]
    }
}
