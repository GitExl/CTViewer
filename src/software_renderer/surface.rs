use std::io::BufWriter;
use std::io::Write;
use std::path::Path;
use std::fs::File;
use std::ptr::write_bytes;
use byteorder::LittleEndian;
use byteorder::WriteBytesExt;
use png::ColorType;
use png::Decoder;
use crate::software_renderer::clip::ClipRect;
use super::palette::Color;

pub struct Surface {
    pub width: u32,
    pub height: u32,
    pub clip: ClipRect,
    pub data: Vec<u8>,
}

impl Surface {
    pub fn new(width: u32, height: u32) -> Surface {
        let len = (width * height * 4) as usize;
        Surface {
            width,
            height,
            clip: ClipRect::new(0, 0, width as i32, height as i32),
            data: vec![0; len],
        }
    }

    pub fn from_png(path: &Path) -> Surface {
        let decoder = Decoder::new(File::open(path).unwrap());
        let mut reader = decoder.read_info().unwrap();
        let mut buf = vec![0; reader.output_buffer_size()];

        let info = reader.next_frame(&mut buf).unwrap();
        if info.color_type != ColorType::Rgba {
            panic!("Surface only supports RGBA PNG images.");
        }

        let mut surface = Surface::new(info.width, info.height);
        surface.data.copy_from_slice(&buf[..info.buffer_size()]);

        surface
    }

    pub fn fill(&mut self, color: Color) {
        let row_len = (self.width * 4) as usize;
        unsafe {
            let (first, rest) = self.data.split_at_mut_unchecked(row_len);
            for i in (0..first.len()).step_by(4) {
                first[i..i + 4].copy_from_slice(&color);
            }
            for i in (0..rest.len()).step_by(row_len) {
                rest[i..i + row_len].copy_from_slice(&first);
            }
        }
    }

    pub fn clear(&mut self) {
        unsafe {
            let vec_ptr = self.data.as_mut_ptr();
            write_bytes(vec_ptr, 0, self.data.len());
        }
    }

    pub fn write_to_bmp(self: &Surface, path: &Path) {
        let file_size = 14 + 40 + self.data.len();
        let data_offset = 14 + 40;

        let file = File::create(&path).expect("Could not open bitmap file for writing");
        let mut writer = BufWriter::new(&file);

        // BitmapHeader
        writer.write_u8(0x42).unwrap();
        writer.write_u8(0x4D).unwrap();
        writer.write_u32::<LittleEndian>(file_size as u32).unwrap();
        writer.write_u16::<LittleEndian>(1).unwrap();
        writer.write_u16::<LittleEndian>(1).unwrap();
        writer.write_u32::<LittleEndian>(data_offset as u32).unwrap();

        // BitmapInfoHeader
        writer.write_u32::<LittleEndian>(40).unwrap();
        writer.write_i32::<LittleEndian>(self.width as i32).unwrap();
        writer.write_i32::<LittleEndian>(self.height as i32).unwrap();
        writer.write_u16::<LittleEndian>(1).unwrap();
        writer.write_u16::<LittleEndian>(32).unwrap();
        writer.write_u32::<LittleEndian>(0).unwrap();
        writer.write_u32::<LittleEndian>(self.data.len() as u32).unwrap();
        writer.write_i32::<LittleEndian>(96).unwrap();
        writer.write_i32::<LittleEndian>(96).unwrap();
        writer.write_u32::<LittleEndian>(0).unwrap();
        writer.write_u32::<LittleEndian>(0).unwrap();

        let mut reversed_pixels = Vec::<u8>::new();
        for src_row in (0..self.height).rev() {
            for pixel in 0..self.width {
                let src = (src_row * self.width * 4 + pixel * 4) as usize;
                reversed_pixels.push(self.data[src + 2]);
                reversed_pixels.push(self.data[src + 1]);
                reversed_pixels.push(self.data[src + 0]);
                reversed_pixels.push(self.data[src + 3]);
            }

        }
        writer.write_all(&reversed_pixels).unwrap();
    }
}
