use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use crate::scene_script::ops::Op;
use crate::scene_script::scene_script_decoder::{read_24_bit_address, read_script_blob, ActorRef, DataRef};

pub fn op_decode_copy(op: u8, data: &mut Cursor<Vec<u8>>) -> Op {
    match op {

        // Set actor result from 0x7F0200.
        0x19 => Op::Copy {
            dest: DataRef::ActorResult(ActorRef::This),
            source: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            width: 1,
        },

        // Set actor result from 0x7F0000.
        0x1C => Op::Copy {
            dest: DataRef::ActorResult(ActorRef::This),
            source: DataRef::StoredLower(data.read_u8().unwrap() as usize),
            width: 1,
        },

        // Set what character the first party member is to 0x7F0200.
        0x20 => Op::Copy {
            dest: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            source: DataRef::PartyCharacter(0),
            width: 1,
        },

        // From RAM to temporary memory.
        0x48 => Op::Copy {
            source: DataRef::RAM(read_24_bit_address(data)),
            dest: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            width: 1,
        },
        0x49 => Op::Copy {
            source: DataRef::RAM(read_24_bit_address(data)),
            dest: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            width: 1,
        },

        // Write directly to RAM.
        0x4A => Op::Copy {
            dest: DataRef::RAM(
                data.read_u8().unwrap() as usize |
                data.read_u8().unwrap() as usize >> 8 |
                data.read_u8().unwrap() as usize >> 16
            ),
            source: DataRef::Immediate(data.read_u8().unwrap() as u32),
            width: 1,
        },
        0x4B => Op::Copy {
            dest: DataRef::RAM(read_24_bit_address(data)),
            source: DataRef::Immediate(data.read_u16::<LittleEndian>().unwrap() as u32),
            width: 2,
        },

        // Write to RAM.
        0x4C => Op::Copy {
            dest: DataRef::RAM(read_24_bit_address(data)),
            source: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            width: 1,
        },
        0x4D => Op::Copy {
            dest: DataRef::RAM(read_24_bit_address(data)),
            source: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            width: 1,
        },
        0x4E => {
            let destination = read_24_bit_address(data);
            let blob = read_script_blob(data);

            Op::Copy {
                dest: DataRef::RAM(destination),
                source: DataRef::Bytes(blob),
                width: blob.len(),
            }
        },
        0x4F => Op::Copy {
            source: DataRef::Immediate(data.read_u8().unwrap() as u32),
            dest: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            width: 1,
        },
        0x50 => Op::Copy {
            source: DataRef::Immediate(data.read_u16::<LittleEndian>().unwrap() as u32),
            dest: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            width: 2,
        },
        0x51 => Op::Copy {
            source: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            dest: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            width: 1,
        },
        0x52 => Op::Copy {
            source: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            dest: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            width: 2,
        },
        0x53 => Op::Copy {
            source: DataRef::Upper(data.read_u16::<LittleEndian>().unwrap() as usize),
            dest: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            width: 1,
        },
        0x54 => Op::Copy {
            source: DataRef::Upper(data.read_u16::<LittleEndian>().unwrap() as usize),
            dest: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            width: 2,
        },
        0x56 => Op::Copy {
            source: DataRef::Immediate(data.read_u8().unwrap() as u32),
            dest: DataRef::Upper(data.read_u16::<LittleEndian>().unwrap() as usize),
            width: 1,
        },
        0x58 => Op::Copy {
            source: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            dest: DataRef::Upper(data.read_u16::<LittleEndian>().unwrap() as usize),
            width: 1,
        },
        0x59 => Op::Copy {
            source: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            dest: DataRef::Upper(data.read_u16::<LittleEndian>().unwrap() as usize),
            width: 2,
        },
        0x75 => Op::Copy {
            source: DataRef::Immediate(1),
            dest: DataRef::StoredLower(data.read_u8().unwrap() as usize * 2),
            width: 1,
        },
        0x76 => Op::Copy {
            source: DataRef::Immediate(1),
            dest: DataRef::StoredLower(data.read_u8().unwrap() as usize * 2),
            width: 2,
        },
        0x77 => Op::Copy {
            source: DataRef::Immediate(0),
            dest: DataRef::StoredLower(data.read_u8().unwrap() as usize * 2),
            width: 1,
        },

        // Write to storyline counter.
        0x55 => Op::Copy {
            source: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            dest: DataRef::StoredLower(0x00),
            width: 1,
        },
        0x5A => Op::Copy {
            source: DataRef::Immediate(data.read_u8().unwrap() as u32),
            dest: DataRef::StoredLower(0x00),
            width: 1,
        },

        0x7F => Op::Copy {
            source: DataRef::Random,
            dest: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            width: 1,
        },

        _ => panic!("Unknown copy op."),
    }
}
