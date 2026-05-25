use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use crate::GameMode;
use crate::scene_script::scene_script_ops::Op;
use crate::memory::{DataDest, DataSource};
use crate::util::data_read::{read_24_bit_address, read_script_blob, read_segmented_address};

pub fn op_decode_copy(op: u8, data: &mut Cursor<Vec<u8>>, mode: GameMode) -> Op {
    match op {

        // Set what character the first party member is to 0x7F0200.
        // "getp0"
        0x20 => Op::Copy8 {
            dest: DataDest::for_local_memory(data.read_u8().unwrap() as usize * 2),
            source: DataSource::PartyCharacter(0),
        },

        // From RAM to temporary memory.
        // "read"
        0x48 => Op::Copy8 {
            source: match mode {
                GameMode::Snes => DataSource::Memory(read_24_bit_address(data)),
                GameMode::Pc => DataSource::Memory(read_segmented_address(data)),
            },
            dest: DataDest::for_local_memory(data.read_u8().unwrap() as usize * 2),
        },
        // "read2"
        0x49 => Op::Copy16 {
            source: match mode {
                GameMode::Snes => DataSource::Memory(read_24_bit_address(data)),
                GameMode::Pc => DataSource::Memory(read_segmented_address(data)),
            },
            dest: DataDest::for_local_memory(data.read_u8().unwrap() as usize * 2),
        },

        // Write to RAM.
        // "write"
        0x4A => Op::Copy8 {
            dest: match mode {
                GameMode::Snes => DataDest::Memory(read_24_bit_address(data)),
                GameMode::Pc => DataDest::Memory(read_segmented_address(data)),
            },
            source: DataSource::Immediate(data.read_u8().unwrap() as i32),
        },
        // "write2"
        0x4B => Op::Copy16 {
            dest: match mode {
                GameMode::Snes => DataDest::Memory(read_24_bit_address(data)),
                GameMode::Pc => DataDest::Memory(read_segmented_address(data)),
            },
            source: DataSource::Immediate(data.read_u16::<LittleEndian>().unwrap() as i32),
        },
        // "vwrite"
        0x4C => Op::Copy8 {
            dest: match mode {
                GameMode::Snes => DataDest::Memory(read_24_bit_address(data)),
                GameMode::Pc => DataDest::Memory(read_segmented_address(data)),
            },
            source: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
        },
        // "vwrite2"
        0x4D => Op::Copy16 {
            dest: match mode {
                GameMode::Snes => DataDest::Memory(read_24_bit_address(data)),
                GameMode::Pc => DataDest::Memory(read_segmented_address(data)),
            },
            source: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
        },
        // "datawrite"
        0x4E => {
            let destination = match mode {
                GameMode::Snes => read_24_bit_address(data),
                GameMode::Pc => read_segmented_address(data),
            };
            let (blob, length) = read_script_blob(data);
            Op::CopyBytes {
                dest: DataDest::Memory(destination),
                bytes: blob,
                length,
            }
        },

        // To local variables.
        // "set"
        0x4F => Op::Copy8 {
            source: DataSource::Immediate(data.read_u8().unwrap() as i32),
            dest: DataDest::for_local_memory(data.read_u8().unwrap() as usize * 2),
        },
        // "set2"
        0x50 => Op::Copy16 {
            source: DataSource::Immediate(data.read_u16::<LittleEndian>().unwrap() as i32),
            dest: DataDest::for_local_memory(data.read_u8().unwrap() as usize * 2),
        },
        // "vset"
        0x51 => Op::Copy8 {
            source: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
            dest: DataDest::for_local_memory(data.read_u8().unwrap() as usize * 2),
        },
        // "vset2"
        0x52 => Op::Copy16 {
            source: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
            dest: DataDest::for_local_memory(data.read_u8().unwrap() as usize * 2),
        },
        // "gset"
        0x53 => Op::Copy8 {
            source: DataSource::for_upper_memory(data.read_u16::<LittleEndian>().unwrap() as usize),
            dest: DataDest::for_local_memory(data.read_u8().unwrap() as usize * 2),
        },
        // "gset2"
        0x54 => Op::Copy16 {
            source: DataSource::for_upper_memory(data.read_u16::<LittleEndian>().unwrap() as usize),
            dest: DataDest::for_local_memory(data.read_u8().unwrap() as usize * 2),
        },

        // "setg"
        0x56 => Op::Copy8 {
            source: DataSource::Immediate(data.read_u8().unwrap() as i32),
            dest: DataDest::for_upper_memory(data.read_u16::<LittleEndian>().unwrap() as usize),
        },
        // "vsetg"
        0x58 => Op::Copy8 {
            source: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
            dest: DataDest::for_upper_memory(data.read_u16::<LittleEndian>().unwrap() as usize),
        },
        // "vsetg2"
        0x59 => Op::Copy16 {
            source: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
            dest: DataDest::for_upper_memory(data.read_u16::<LittleEndian>().unwrap() as usize),
        },

        // "on"
        0x75 => Op::Copy8 {
            source: DataSource::Immediate(1),
            dest: DataDest::for_local_memory(data.read_u8().unwrap() as usize * 2),
        },
        // "on2"
        0x76 => Op::Copy16 {
            source: DataSource::Immediate(1),
            dest: DataDest::for_local_memory(data.read_u8().unwrap() as usize * 2),
        },
        // "off"
        0x77 => Op::Copy8 {
            source: DataSource::Immediate(0),
            dest: DataDest::for_local_memory(data.read_u8().unwrap() as usize * 2),
        },

        // Write to storyline counter.
        // "sset"
        0x55 => Op::Copy8 {
            source: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
            dest: DataDest::for_global_memory(0x00),
        },
        // "sets"
        0x5A => Op::Copy8 {
            source: DataSource::Immediate(data.read_u8().unwrap() as i32),
            dest: DataDest::for_global_memory(0x00),
        },

        // PC-specific ops.
        // "setex"
        0x3A => Op::Copy8 {
            source: DataSource::Immediate(data.read_u8().unwrap() as i32),
            dest: DataDest::for_extended_memory(data.read_u8().unwrap() as usize),
        },
        // "vsetex"
        0x3D => Op::Copy8 {
            source: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
            dest: DataDest::for_extended_memory(data.read_u8().unwrap() as usize),
        },
        // "exset"
        0x3E => Op::Copy8 {
            source: DataSource::for_extended_memory(data.read_u8().unwrap() as usize),
            dest: DataDest::for_local_memory(data.read_u8().unwrap() as usize * 2),
        },
        // "getp"
        0x70 => Op::Copy8 {
            source: DataSource::PartyCharacter(data.read_u8().unwrap() as usize),
            dest: DataDest::for_local_memory(data.read_u8().unwrap() as usize * 2),
        },
        // "exset2"
        0x74 => Op::Copy16 {
            source: DataSource::for_extended_memory(data.read_u8().unwrap() as usize),
            dest: DataDest::for_local_memory(data.read_u8().unwrap() as usize * 2),
        },
        // "vsetex2"
        0x78 => Op::Copy16 {
            source: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
            dest: DataDest::for_extended_memory(data.read_u8().unwrap() as usize),
        },

        _ => panic!("Unknown copy op."),
    }
}
