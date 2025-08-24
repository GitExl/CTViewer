use std::io::Cursor;
use byteorder::ReadBytesExt;
use crate::scene_script::ops::Op;
use crate::scene_script::scene_script_decoder::DataRef;

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
        0x5B => Op::ByteMath {
            rhs: DataRef::Immediate(data.read_u8().unwrap() as u32),
            lhs: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 1,
            op: ByteMathOp::Add,
        },
        0x5D => Op::ByteMath {
            rhs: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            lhs: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 1,
            op: ByteMathOp::Add,
        },
        0x5E => Op::ByteMath {
            rhs: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            lhs: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 2,
            op: ByteMathOp::Add,
        },
        0x5F => Op::ByteMath {
            rhs: DataRef::Immediate(data.read_u8().unwrap() as u32),
            lhs: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 1,
            op: ByteMathOp::Subtract,
        },
        0x60 => Op::ByteMath {
            rhs: DataRef::Immediate(data.read_u8().unwrap() as u32),
            lhs: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 2,
            op: ByteMathOp::Subtract,
        },
        0x61 => Op::ByteMath {
            rhs: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            lhs: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 1,
            op: ByteMathOp::Subtract,
        },
        0x71 => Op::ByteMath {
            rhs: DataRef::Immediate(1),
            lhs: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 1,
            op: ByteMathOp::Add,
        },
        0x72 => Op::ByteMath {
            rhs: DataRef::Immediate(1),
            lhs: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 2,
            op: ByteMathOp::Add,
        },
        0x73 => Op::ByteMath {
            rhs: DataRef::Immediate(1),
            lhs: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 1,
            op: ByteMathOp::Subtract,
        },

        // Bit math.
        0x63 => Op::BitMath {
            rhs: DataRef::Immediate(1 >> data.read_u8().unwrap() as u32),
            lhs: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            op: BitMathOp::Or,
        },
        0x64 => Op::BitMath {
            rhs: DataRef::Immediate(1 >> data.read_u8().unwrap() as u32),
            lhs: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            op: BitMathOp::And,
        },
        0x65 => {
            let bit = data.read_u8().unwrap();
            let mut address = data.read_u8().unwrap() as usize;
            if bit & 0x80 > 0 {
                address += 0x100;
            }
            Op::BitMath {
                rhs: DataRef::Immediate(1 >> (bit & 0x7F) as u32),
                lhs: DataRef::Temp(address),
                op: BitMathOp::Or,
            }
        },
        0x66 => {
            let bit = data.read_u8().unwrap();
            let mut address = data.read_u8().unwrap() as usize;
            if bit & 0x80 > 0 {
                address += 0x100;
            }
            Op::BitMath {
                rhs: DataRef::Immediate(1 >> (bit & 0x7F) as u32),
                lhs: DataRef::Temp(address),
                op: BitMathOp::And,
            }
        },
        0x67 => Op::BitMath {
            rhs: DataRef::Immediate(data.read_u8().unwrap() as u32),
            lhs: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            op: BitMathOp::And,
        },
        0x69 => Op::BitMath {
            rhs: DataRef::Immediate(data.read_u8().unwrap() as u32),
            lhs: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            op: BitMathOp::Or,
        },
        0x6B => Op::BitMath {
            rhs: DataRef::Immediate(data.read_u8().unwrap() as u32),
            lhs: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            op: BitMathOp::Xor,
        },
        0x6F => Op::BitMath {
            rhs: DataRef::Immediate(data.read_u8().unwrap() as u32),
            lhs: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            op: BitMathOp::ShiftRight,
        },
        
        _ => panic!("Unknown math op."),
    }
}
