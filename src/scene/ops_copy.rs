use std::io::{Cursor, Read};
use byteorder::{LittleEndian, ReadBytesExt};
use crate::scene::scene_script_decoder::{read_script_blob, ActorRef, DataValue, Op};

pub fn op_decode_copy(op: u8, data: &mut Cursor<Vec<u8>>) -> Op {
    match op {
        // Copy data around in memory.
        // Set actor result from 0x7F0200.
        0x19 => Op::Copy {
            destination: DataValue::ActorResult(ActorRef::This),
            source: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 1,
        },
        // Set actor result from 0x7F0000.
        0x1C => Op::Copy {
            destination: DataValue::ActorResult(ActorRef::This),
            source: DataValue::StoredLower(data.read_u8().unwrap() as usize),
            byte_count: 1,
        },
        // Set what character the first party member is to 0x7F0200.
        0x20 => Op::Copy {
            destination: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            source: DataValue::PartyCharacter(0),
            byte_count: 1,
        },
        // From RAM to temporary memory.
        0x48 => Op::Copy {
            // todo validate that this is read correctly.
            source: DataValue::RAM(
                data.read_u8().unwrap() as usize |
                    data.read_u8().unwrap() as usize >> 8 |
                    data.read_u8().unwrap() as usize >> 16
            ),
            destination: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 1,
        },
        0x49 => Op::Copy {
            // todo validate that this is read correctly.
            source: DataValue::RAM(
                data.read_u8().unwrap() as usize |
                    data.read_u8().unwrap() as usize >> 8 |
                    data.read_u8().unwrap() as usize >> 16
            ),
            destination: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 1,
        },
        // Write directly to RAM.
        0x4A => Op::Copy {
            // todo validate that this is read correctly.
            destination: DataValue::RAM(
                data.read_u8().unwrap() as usize |
                    data.read_u8().unwrap() as usize >> 8 |
                    data.read_u8().unwrap() as usize >> 16
            ),
            source: DataValue::Immediate(data.read_u8().unwrap() as u32),
            byte_count: 1,
        },
        0x4B => Op::Copy {
            // todo validate that this is read correctly.
            destination: DataValue::RAM(
                data.read_u8().unwrap() as usize |
                    data.read_u8().unwrap() as usize >> 8 |
                    data.read_u8().unwrap() as usize >> 16
            ),
            source: DataValue::Immediate(data.read_u16::<LittleEndian>().unwrap() as u32),
            byte_count: 2,
        },
        // Write to RAM.
        0x4C => Op::Copy {
            // todo validate that this is read correctly.
            destination: DataValue::RAM(
                data.read_u8().unwrap() as usize |
                    data.read_u8().unwrap() as usize >> 8 |
                    data.read_u8().unwrap() as usize >> 16
            ),
            source: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 1,
        },
        0x4D => Op::Copy {
            // todo validate that this is read correctly.
            destination: DataValue::RAM(
                data.read_u8().unwrap() as usize |
                    data.read_u8().unwrap() as usize >> 8 |
                    data.read_u8().unwrap() as usize >> 16
            ),
            source: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 1,
        },
        0x4E => {
            // todo validate that this is read correctly.
            let destination =
                data.read_u8().unwrap() as usize |
                    data.read_u8().unwrap() as usize >> 8 |
                    data.read_u8().unwrap() as usize >> 16;
            let blob = read_script_blob(data);

            Op::Copy {
                destination: DataValue::RAM(destination),
                source: DataValue::Bytes(blob),
                byte_count: blob.len(),
            }
        },
        0x4F => Op::Copy {
            source: DataValue::Immediate(data.read_u8().unwrap() as u32),
            destination: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 1,
        },
        0x50 => Op::Copy {
            source: DataValue::Immediate(data.read_u16::<LittleEndian>().unwrap() as u32),
            destination: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 2,
        },
        0x51 => Op::Copy {
            source: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            destination: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 1,
        },
        0x52 => Op::Copy {
            source: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            destination: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 2,
        },
        0x53 => Op::Copy {
            source: DataValue::Upper(data.read_u16::<LittleEndian>().unwrap() as usize),
            destination: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 1,
        },
        0x54 => Op::Copy {
            source: DataValue::Upper(data.read_u16::<LittleEndian>().unwrap() as usize),
            destination: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 2,
        },
        0x56 => Op::Copy {
            source: DataValue::Immediate(data.read_u8().unwrap() as u32),
            destination: DataValue::StoredLower(data.read_u8().unwrap() as usize * 2),
            byte_count: 1,
        },
        0x58 => Op::Copy {
            source: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            destination: DataValue::Upper(data.read_u16::<LittleEndian>().unwrap() as usize),
            byte_count: 1,
        },
        0x59 => Op::Copy {
            source: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            destination: DataValue::Upper(data.read_u16::<LittleEndian>().unwrap() as usize),
            byte_count: 2,
        },
        0x75 => Op::Copy {
            source: DataValue::Immediate(1),
            destination: DataValue::StoredLower(data.read_u8().unwrap() as usize * 2),
            byte_count: 1,
        },
        0x76 => Op::Copy {
            source: DataValue::Immediate(1),
            destination: DataValue::StoredLower(data.read_u8().unwrap() as usize * 2),
            byte_count: 2,
        },
        0x77 => Op::Copy {
            source: DataValue::Immediate(0),
            destination: DataValue::StoredLower(data.read_u8().unwrap() as usize * 2),
            byte_count: 1,
        },

        // Write to storyline counter.
        0x55 => Op::Copy {
            source: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            destination: DataValue::StoredLower(0x00),
            byte_count: 1,
        },
        0x5A => Op::Copy {
            source: DataValue::Immediate(data.read_u8().unwrap() as u32),
            destination: DataValue::StoredLower(0x00),
            byte_count: 1,
        },

        0x7F => Op::Copy {
            source: DataValue::Random,
            destination: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 1,
        },

        _ => panic!("Unknown copy op."),
    }
}
