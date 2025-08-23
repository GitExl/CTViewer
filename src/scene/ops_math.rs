use std::io::Cursor;
use byteorder::ReadBytesExt;
use crate::scene::scene_script_decoder::{BitMathOp, ByteMathOp, DataValue, Op};

pub fn op_decode_math(op: u8, data: &mut Cursor<Vec<u8>>) -> Op {
    match op {

        // Byte math.
        0x5B => Op::ByteMath {
            rhs: DataValue::Immediate(data.read_u8().unwrap() as u32),
            lhs: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 1,
            operation: ByteMathOp::Add,
        },
        0x5D => Op::ByteMath {
            rhs: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            lhs: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 1,
            operation: ByteMathOp::Add,
        },
        0x5E => Op::ByteMath {
            rhs: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            lhs: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 2,
            operation: ByteMathOp::Add,
        },
        0x5F => Op::ByteMath {
            rhs: DataValue::Immediate(data.read_u8().unwrap() as u32),
            lhs: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 1,
            operation: ByteMathOp::Subtract,
        },
        0x60 => Op::ByteMath {
            rhs: DataValue::Immediate(data.read_u8().unwrap() as u32),
            lhs: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 2,
            operation: ByteMathOp::Subtract,
        },
        0x61 => Op::ByteMath {
            rhs: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            lhs: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 1,
            operation: ByteMathOp::Subtract,
        },
        0x71 => Op::ByteMath {
            rhs: DataValue::Immediate(1),
            lhs: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 1,
            operation: ByteMathOp::Add,
        },
        0x72 => Op::ByteMath {
            rhs: DataValue::Immediate(1),
            lhs: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 2,
            operation: ByteMathOp::Add,
        },
        0x73 => Op::ByteMath {
            rhs: DataValue::Immediate(1),
            lhs: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 1,
            operation: ByteMathOp::Subtract,
        },

        // Bit math.
        0x63 => Op::BitMath {
            rhs: DataValue::Immediate(1 >> data.read_u8().unwrap() as u32),
            lhs: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            operation: BitMathOp::Or,
        },
        0x64 => Op::BitMath {
            rhs: DataValue::Immediate(1 >> data.read_u8().unwrap() as u32),
            lhs: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            operation: BitMathOp::And,
        },
        0x65 => {
            let bit = data.read_u8().unwrap();
            let mut address = data.read_u8().unwrap() as usize;
            if bit & 0x80 > 0 {
                address += 0x100;
            }
            Op::BitMath {
                rhs: DataValue::Immediate(1 >> (bit & 0x7F) as u32),
                lhs: DataValue::Temp(address),
                operation: BitMathOp::Or,
            }
        },
        0x66 => {
            let bit = data.read_u8().unwrap();
            let mut address = data.read_u8().unwrap() as usize;
            if bit & 0x80 > 0 {
                address += 0x100;
            }
            Op::BitMath {
                rhs: DataValue::Immediate(1 >> (bit & 0x7F) as u32),
                lhs: DataValue::Temp(address),
                operation: BitMathOp::And,
            }
        },
        0x67 => Op::BitMath {
            rhs: DataValue::Immediate(data.read_u8().unwrap() as u32),
            lhs: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            operation: BitMathOp::And,
        },
        0x69 => Op::BitMath {
            rhs: DataValue::Immediate(data.read_u8().unwrap() as u32),
            lhs: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            operation: BitMathOp::Or,
        },
        0x6B => Op::BitMath {
            rhs: DataValue::Immediate(data.read_u8().unwrap() as u32),
            lhs: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            operation: BitMathOp::Xor,
        },
        0x6F => Op::BitMath {
            rhs: DataValue::Immediate(data.read_u8().unwrap() as u32),
            lhs: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            operation: BitMathOp::ShiftRight,
        },
        
        _ => panic!("Unknown math op."),
    }
}
