use std::ptr::write_bytes;
use crate::software_renderer::clip::ClipRect;

pub struct Bitmap {
    pub width: u32,
    pub height: u32,
    pub clip: ClipRect,
    pub data: Vec<u8>,
}

impl Bitmap {
    pub fn new(width: u32, height: u32) -> Bitmap {
        Bitmap {
            width,
            height,
            clip: ClipRect::new(0, 0, width as i32, height as i32),
            data: vec![0; (width * height) as usize],
        }
    }

    pub fn from_raw_data(width: u32, height: u32, data: Vec<u8>) -> Bitmap {
        Bitmap {
            width,
            height,
            clip: ClipRect::new(0, 0, width as i32, height as i32),
            data,
        }
    }

    pub fn clear(&mut self) {
        unsafe {
            let vec_ptr = self.data.as_mut_ptr();
            write_bytes(vec_ptr, 0, self.data.len());
        }
    }
}
