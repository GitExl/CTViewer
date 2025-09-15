use std::fs::File;
use std::io::Cursor;
use std::io::Read;
use std::io::Seek;
use std::path::Path;
use byteorder::LittleEndian;
use byteorder::ReadBytesExt;
use crate::software_renderer::palette::Color;

pub type BmpColor = [u8; 4];

pub enum BPP {
    Bpp1,
    Bpp2,
    Bpp4,
    Bpp8,
}

pub struct Bmp {
    pub width: u32,
    pub height: u32,
    pub bpp: BPP,
    pub palette: Vec<BmpColor>,
    pub pixels: Vec<u8>,
}

impl Bmp {
    pub fn get_raw_palette(&self) -> Vec<[u8; 4]> {
        let mut colors = vec![Color::default(); self.palette.len()];
        for (index, color) in self.palette.iter().enumerate() {
            colors[index][0] = color[0];
            colors[index][1] = color[1];
            colors[index][2] = color[2];
            colors[index][3] = 0xFF;
        }

        colors
    }

    pub fn from_path(path: &Path) -> Bmp {
        let mut file = File::open(path).unwrap();
        let mut data = vec![0u8; file.metadata().unwrap().len() as usize];
        file.read_exact(&mut data).unwrap();
        let mut cursor = Cursor::new(data);
        Bmp::from_cursor(&mut cursor)
    }

    pub fn from_cursor(cursor: &mut Cursor<Vec<u8>>) -> Bmp {

        // BitmapHeader
        let id = cursor.read_u16::<LittleEndian>().unwrap();
        if id != 0x4D42 {
            panic!("Bitmap has a bad id.");
        }

        let file_size = cursor.read_u32::<LittleEndian>().unwrap() as u64;
        if file_size != cursor.get_ref().len() as u64 {
            panic!("Bitmap size does not match file size.");
        }

        cursor.read_u16::<LittleEndian>().unwrap();
        cursor.read_u16::<LittleEndian>().unwrap();
        let data_offset = cursor.read_u32::<LittleEndian>().unwrap() as u64;
        if data_offset >= cursor.get_ref().len() as u64 {
            panic!("Bitmap has data past the file size.");
        }

        // BitmapInfoHeader
        let header_size = cursor.read_u32::<LittleEndian>().unwrap();
        if header_size != 108 && header_size != 40 {
            panic!("Bitmap has a BitmapInfoHeader that is not 108 or 40 bytes.");
        }
        let width = cursor.read_i32::<LittleEndian>().unwrap() as u32;
        let height = cursor.read_i32::<LittleEndian>().unwrap() as u32;
        if width > 32767 || height > 32767 {
            panic!("Bitmap has bad dimensions.");
        }
        let planes = cursor.read_u16::<LittleEndian>().unwrap();
        if planes != 1 {
            panic!("Bitmap must have 1 plane.");
        }
        let bpp = match cursor.read_u16::<LittleEndian>().unwrap() {
            1 => BPP::Bpp1,
            2 => BPP::Bpp2,
            4 => BPP::Bpp4,
            8 => BPP::Bpp8,
            _ => {
                panic!("Bitmap must be paletted.");
            }
        };
        let compression_method = cursor.read_u32::<LittleEndian>().unwrap();
        if compression_method != 0 {
            panic!("Bitmap must be uncompressed.")
        }
        cursor.read_u32::<LittleEndian>().unwrap();
        cursor.read_u32::<LittleEndian>().unwrap();
        cursor.read_u32::<LittleEndian>().unwrap();
        let color_count = cursor.read_u32::<LittleEndian>().unwrap() as usize;
        cursor.read_u32::<LittleEndian>().unwrap();

        // Skip BITMAPV4HEADER color correction stuff.
        cursor.seek(std::io::SeekFrom::Current(68)).expect("Bitmap cannot seek past header.");

        // Palette.
        let mut palette: Vec<BmpColor> = vec![[0, 0, 0, 0]; color_count];
        for i in 0..color_count {
            // ARGB > RGBA
            palette[i][2] = cursor.read_u8().unwrap();
            palette[i][1] = cursor.read_u8().unwrap();
            palette[i][0] = cursor.read_u8().unwrap();
            palette[i][3] = cursor.read_u8().unwrap();
        }

        cursor.seek(std::io::SeekFrom::Start(data_offset))
            .expect("Bitmap could not seek to start of image data.");
        let pixels = read_pixels(cursor, width, height, &bpp);

        Bmp {
            width,
            height,
            bpp,
            palette,
            pixels,
        }
    }
}

fn read_pixels(reader: &mut Cursor<Vec<u8>>, width: u32, height: u32, bpp: &BPP) -> Vec<u8> {
    let bytes_per_row = match bpp {
        BPP::Bpp1 => width / 8,
        BPP::Bpp2 => width / 4,
        BPP::Bpp4 => width / 2,
        BPP::Bpp8 => width,
    };

    let mut raw_data: Vec<u8> = vec![0; (bytes_per_row * height) as usize];
    reader.read_exact(&mut raw_data).unwrap();

    let mut pixels: Vec<u8> = vec![0; (width * height) as usize];
    let mut dest;
    let mut src = 0;

    for y in 0..height {
        dest = ((height - y - 1) * width) as usize;

        for _ in 0..bytes_per_row {
            match bpp {
                BPP::Bpp1 => {
                    pixels[dest + 0] = (raw_data[src] & 0x80) >> 7;
                    pixels[dest + 1] = (raw_data[src] & 0x40) >> 6;
                    pixels[dest + 2] = (raw_data[src] & 0x20) >> 5;
                    pixels[dest + 3] = (raw_data[src] & 0x10) >> 4;
                    pixels[dest + 4] = (raw_data[src] & 0x08) >> 3;
                    pixels[dest + 5] = (raw_data[src] & 0x04) >> 2;
                    pixels[dest + 6] = (raw_data[src] & 0x02) >> 1;
                    pixels[dest + 7] =  raw_data[src] & 0x01;
                    dest += 8;
                    src += 1;
                },
                BPP::Bpp2 => {
                    pixels[dest + 0] = (raw_data[src] & 0xC0) >> 6;
                    pixels[dest + 1] = (raw_data[src] & 0x30) >> 4;
                    pixels[dest + 2] = (raw_data[src] & 0x0C) >> 2;
                    pixels[dest + 3] =  raw_data[src] & 0x03;
                    dest += 4;
                    src += 1;
                },
                BPP::Bpp4 => {
                    pixels[dest + 0] = (raw_data[src] & 0xF0) >> 4;
                    pixels[dest + 1] =  raw_data[src] & 0x0F;
                    dest += 2;
                    src += 1;
                },
                BPP::Bpp8 => {
                    pixels[dest] = raw_data[src];
                    dest += 1;
                    src += 1;
                },
            };
        }
    }

    pixels
}
