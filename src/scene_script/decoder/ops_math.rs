use std::io::Cursor;
use byteorder::ReadBytesExt;
use crate::scene_script::ops::Op;
use crate::scene_script::scene_script_memory::{DataDest, DataSource};

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum BitMathOp {
    Or,
    And,
    Xor,
    ShiftLeft,
    ShiftRight,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ByteMathOp {
    Add,
    Subtract,
}

pub fn op_decode_math(op: u8, data: &mut Cursor<Vec<u8>>) -> Op {
    match op {

        // Byte math.
        0x5B => {
            let rhs = data.read_u8().unwrap() as i32;
            let lhs = data.read_u8().unwrap() as usize * 2;
            Op::ByteMath8 {
                dest: DataDest::for_local_memory(lhs),
                lhs: DataSource::for_local_memory(lhs),
                op: ByteMathOp::Add,
                rhs: DataSource::Immediate(rhs),
            }
        },
        0x5D => {
            let rhs = data.read_u8().unwrap() as usize * 2;
            let lhs = data.read_u8().unwrap() as usize * 2;
            Op::ByteMath8 {
                dest: DataDest::for_local_memory(lhs),
                lhs: DataSource::for_local_memory(lhs),
                op: ByteMathOp::Add,
                rhs: DataSource::for_local_memory(rhs),
            }
        },
        0x5E => {
            let rhs = data.read_u8().unwrap() as usize * 2;
            let lhs = data.read_u8().unwrap() as usize * 2;
            Op::ByteMath16 {
                dest: DataDest::for_local_memory(lhs),
                lhs: DataSource::for_local_memory(lhs),
                op: ByteMathOp::Add,
                rhs: DataSource::for_local_memory(rhs),
            }
        },
        0x5F => {
            let rhs = data.read_u8().unwrap() as i32;
            let lhs = data.read_u8().unwrap() as usize * 2;
            Op::ByteMath8 {
                dest: DataDest::for_local_memory(lhs),
                lhs: DataSource::for_local_memory(lhs),
                op: ByteMathOp::Subtract,
                rhs: DataSource::Immediate(rhs),
            }
        },
        0x60 => {
            let rhs = data.read_u8().unwrap() as usize * 2;
            let lhs = data.read_u8().unwrap() as usize * 2;
            Op::ByteMath16 {
                dest: DataDest::for_local_memory(lhs),
                lhs: DataSource::for_local_memory(lhs),
                op: ByteMathOp::Subtract,
                rhs: DataSource::for_local_memory(rhs),
            }
        },
        0x61 => {
            let rhs = data.read_u8().unwrap() as usize * 2;
            let lhs = data.read_u8().unwrap() as usize * 2;
            Op::ByteMath8 {
                dest: DataDest::for_local_memory(lhs),
                lhs: DataSource::for_local_memory(lhs),
                op: ByteMathOp::Subtract,
                rhs: DataSource::for_local_memory(rhs),
            }
        },
        0x71 => {
            let lhs = data.read_u8().unwrap() as usize * 2;
            Op::ByteMath8 {
                dest: DataDest::for_local_memory(lhs),
                lhs: DataSource::for_local_memory(lhs),
                op: ByteMathOp::Add,
                rhs: DataSource::Immediate(1),
            }
        },
        0x72 => {
            let lhs = data.read_u8().unwrap() as usize * 2;
            Op::ByteMath16 {
                dest: DataDest::for_local_memory(lhs),
                lhs: DataSource::for_local_memory(lhs),
                op: ByteMathOp::Add,
                rhs: DataSource::Immediate(1),
            }
        },
        0x73 => {
            let lhs = data.read_u8().unwrap() as usize * 2;
            Op::ByteMath8 {
                dest: DataDest::for_local_memory(lhs),
                lhs: DataSource::for_local_memory(lhs),
                op: ByteMathOp::Subtract,
                rhs: DataSource::Immediate(1),
            }
        },

        // Bit math.
        0x2A => Op::BitMath {
            dest: DataDest::for_temp_memory(0x7E0154),
            lhs: DataSource::for_temp_memory(0x7E0154),
            op: BitMathOp::Or,
            rhs: DataSource::Immediate(0x04),
        },
        0x2B => Op::BitMath {
            dest: DataDest::for_temp_memory(0x7E0154),
            lhs: DataSource::for_temp_memory(0x7E0154),
            op: BitMathOp::Or,
            rhs: DataSource::Immediate(0x08),
        },
        0x32 => Op::BitMath {
            dest: DataDest::for_temp_memory(0x7E0154),
            lhs: DataSource::for_temp_memory(0x7E0154),
            op: BitMathOp::Or,
            rhs: DataSource::Immediate(0x10),
        },
        0x63 => {
            let rhs = 1 << data.read_u8().unwrap() as u32;
            let lhs = data.read_u8().unwrap() as usize * 2;
            Op::BitMath {
                dest: DataDest::for_local_memory(lhs),
                lhs: DataSource::for_local_memory(lhs),
                op: BitMathOp::Or,
                rhs: DataSource::Immediate(rhs),
            }
        },
        0x64 => {
            let rhs = !(1 << data.read_u8().unwrap() as u32);
            let lhs = data.read_u8().unwrap() as usize * 2;
            Op::BitMath {
                dest: DataDest::for_local_memory(lhs),
                lhs: DataSource::for_local_memory(lhs),
                op: BitMathOp::And,
                rhs: DataSource::Immediate(rhs),
            }
        },
        0x65 => {
            let bit = data.read_u8().unwrap();
            let mut lhs = data.read_u8().unwrap() as usize;
            if bit & 0x80 > 0 {
                lhs += 0x100;
            }
            Op::BitMath {
                dest: DataDest::for_global_memory(lhs),
                rhs: DataSource::Immediate(1 << (bit & 0x7F) as u32),
                lhs: DataSource::for_global_memory(lhs),
                op: BitMathOp::Or,
            }
        },
        0x66 => {
            let bit = data.read_u8().unwrap();
            let mut lhs = data.read_u8().unwrap() as usize;
            if bit & 0x80 > 0 {
                lhs += 0x100;
            }
            Op::BitMath {
                dest: DataDest::for_global_memory(lhs),
                rhs: DataSource::Immediate(1 << (bit & 0x7F)),
                lhs: DataSource::for_global_memory(lhs),
                op: BitMathOp::And,
            }
        },
        0x67 => {
            let rhs = data.read_u8().unwrap() as i32;
            let lhs = data.read_u8().unwrap() as usize * 2;
            Op::BitMath {
                dest: DataDest::for_local_memory(lhs),
                lhs: DataSource::for_local_memory(lhs),
                op: BitMathOp::And,
                rhs: DataSource::Immediate(rhs),
            }
        },
        0x69 => {
            let rhs = data.read_u8().unwrap() as i32;
            let lhs = data.read_u8().unwrap() as usize * 2;
            Op::BitMath {
                dest: DataDest::for_local_memory(lhs),
                lhs: DataSource::for_local_memory(lhs),
                op: BitMathOp::Or,
                rhs: DataSource::Immediate(rhs),
            }
        },
        0x6B => {
            let rhs = data.read_u8().unwrap() as i32;
            let lhs = data.read_u8().unwrap() as usize * 2;
            Op::BitMath {
                dest: DataDest::for_local_memory(lhs),
                lhs: DataSource::for_local_memory(lhs),
                op: BitMathOp::Xor,
                rhs: DataSource::Immediate(rhs),
            }
        },
        0x6F => {
            let rhs = data.read_u8().unwrap() as i32;
            let lhs = data.read_u8().unwrap() as usize * 2;
            Op::BitMath {
                dest: DataDest::for_local_memory(lhs),
                lhs: DataSource::for_local_memory(lhs),
                op: BitMathOp::ShiftRight,
                rhs: DataSource::Immediate(rhs),
            }
        },

        // PC specific ops.
        0x45 => {
            let bit = data.read_u8().unwrap();
            let lhs = data.read_u8().unwrap() as usize;
            Op::BitMath {
                dest: DataDest::for_extended_memory(lhs),
                rhs: DataSource::Immediate(1 >> (bit & 0x7F) as u32),
                lhs: DataSource::for_extended_memory(lhs),
                op: BitMathOp::Or,
            }
        },
        0x46 => {
            let rhs = !(1 << data.read_u8().unwrap() as u32);
            let lhs = data.read_u8().unwrap() as usize;
            Op::BitMath {
                dest: DataDest::for_extended_memory(lhs),
                lhs: DataSource::for_extended_memory(lhs),
                op: BitMathOp::And,
                rhs: DataSource::Immediate(rhs),
            }
        },
        
        _ => panic!("Unknown math op."),
    }
}
