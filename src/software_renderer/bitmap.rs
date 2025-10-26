use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::ptr::write_bytes;
use png::{ColorType, Decoder};
use crate::software_renderer::palette::Palette;
use crate::util::rect::Rect;

pub struct Bitmap {
    pub width: u32,
    pub height: u32,
    pub clip: Rect,
    pub data: Vec<u8>,
}

impl Bitmap {
    pub fn new(width: u32, height: u32) -> Bitmap {
        Bitmap {
            width,
            height,
            clip: Rect::new(0, 0, width as i32, height as i32),
            data: vec![0; (width * height) as usize],
        }
    }

    pub fn from_raw_data(width: u32, height: u32, data: Vec<u8>) -> Bitmap {
        Bitmap {
            width,
            height,
            clip: Rect::new(0, 0, width as i32, height as i32),
            data,
        }
    }

    pub fn from_png(path: &Path) -> (Bitmap, Palette) {
        let buf = BufReader::new(File::open(path).unwrap());
        let decoder = Decoder::new(buf);
        let mut reader = decoder.read_info().unwrap();
        let mut buf = vec![0; reader.output_buffer_size().unwrap()];

        let info = reader.next_frame(&mut buf).unwrap();
        if info.color_type != ColorType::Indexed {
            panic!("Bitmap only supports reading indexed PNG images.");
        }

        let mut bitmap = Bitmap::new(info.width, info.height);
        bitmap.data.copy_from_slice(&buf[..info.buffer_size()]);

        let mut palette = Palette::new(256);
        let data = reader.info().palette.as_ref().unwrap();
        for i in 0..data.len() / 3 {
            palette.colors[i][0] = data[i * 3 + 0];
            palette.colors[i][1] = data[i * 3 + 1];
            palette.colors[i][2] = data[i * 3 + 2];
        }

        (bitmap, palette)
    }

    pub fn clear(&mut self) {
        unsafe {
            let vec_ptr = self.data.as_mut_ptr();
            write_bytes(vec_ptr, 0, self.data.len());
        }
    }

    pub fn scale_linear(&self, factor: f64) -> Bitmap {
        let width = (self.width as f64 * factor).ceil() as u32;
        let height = (self.height as f64 * factor).ceil() as u32;
        let mut bitmap = Bitmap::new(width, height);

        for y in 0..height {
            for x in 0..width {
                let scale_x = x as f64 / factor;
                let scale_y = y as f64 / factor;
                let src = ((scale_y * self.width as f64) + scale_x) as usize;
                let dest = (y * width + x) as usize;

                bitmap.data[dest] = self.data[src];
            }
        }

        bitmap
    }
}
