use std::io::Cursor;
use byteorder::ReadBytesExt;
use crate::scene::ops::Op;
use crate::scene::scene_script_decoder::read_script_blob;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum SubPalette {
    This,
    Index(usize),
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ColorMathMode {
    Additive,
    Subtractive,
}

pub fn op_decode_palette(op: u8, data: &mut Cursor<Vec<u8>>) -> Op {
    match op {

        // Dual mode palette command.
        0x2E => {
            let mode = data.read_u8().unwrap();
            if mode & 0x40 > 0 {
                let b = ((mode & 0x4) >> 2) > 0;
                let g = ((mode & 0x2) >> 1) > 0;
                let r = ((mode & 0x1) >> 0) > 0;

                let color_start = data.read_u8().unwrap();
                let color_count = data.read_u8().unwrap();

                let intensity_bits = data.read_u8().unwrap();
                let intensity_end: f64 = (intensity_bits & 0xF) as f64 * (1.0 / 15.0);
                let intensity_start: f64 = ((intensity_bits & 0xF0) >> 4) as f64 * (1.0 / 15.0);

                // todo what unit is this in? Assuming 60 Hz frames for now.
                let duration = data.read_u8().unwrap() as f64 * (1.0 / 60.0);

                Op::ColorMath {
                    mode: if mode & 0x50 > 0 { ColorMathMode::Additive } else { ColorMathMode::Subtractive },
                    r, g, b,
                    color_start, color_count,
                    intensity_start, intensity_end,
                    duration,
                }

            } else if mode & 0x80 > 0 {
                let bits = data.read_u8().unwrap() as usize;
                let color_index = bits & 0xF;
                let sub_palette = (bits & 0xF0) >> 4;

                Op::PaletteSetImmediate {
                    sub_palette: SubPalette::Index(sub_palette),
                    color_index,
                    data: read_script_blob(data),
                }
            } else {
                println!("Mode for op 0x2E is unknown.");
                Op::NOP
            }
        },
        0x33 => Op::PaletteSet {
            palette: data.read_u8().unwrap() as usize,
        },

        // 0x88 sub ops.
        0x88 => {
            let cmd = data.read_u8().unwrap();
            if cmd == 0 {
                Op::PaletteRestore
            } else if cmd == 0x20 {
                Op::Unknown {
                    code: 0x88,
                    data: [cmd, data.read_u8().unwrap(), data.read_u8().unwrap(), 0],
                }
            } else if cmd == 0x30 {
                Op::Unknown {
                    code: 0x88,
                    data: [cmd, data.read_u8().unwrap(), data.read_u8().unwrap(), 0],
                }
            } else if cmd > 0x40 && cmd < 0x60 {
                Op::Unknown {
                    code: 0x88,
                    data: [cmd, data.read_u8().unwrap(), data.read_u8().unwrap(), data.read_u8().unwrap()],
                }
            } else if cmd > 0x80 && cmd < 0x90 {
                Op::PaletteSetImmediate {
                    color_index: cmd as usize & 0xF,
                    sub_palette: SubPalette::This,
                    data: read_script_blob(data),
                }
            } else {
                panic!("Unknown 0x88 command {}.", cmd);
            }
        },

        _ => panic!("Unknown palette op."),
    }
}
