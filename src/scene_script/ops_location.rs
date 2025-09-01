use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use crate::scene_script::ops::Op;
use crate::scene_script::scene_script_decoder::DataSource;

pub fn op_decode_location(op: u8, data: &mut Cursor<Vec<u8>>) -> Op {
    match op {

        // These are all the same except for the last one that reads parameters from memory.
        // The differences are not at all documented.
        0xDC => Op::ChangeLocation {
            index_direction: DataSource::Immediate(data.read_u16::<LittleEndian>().unwrap() as u32),
            x: DataSource::Immediate(data.read_u8().unwrap() as u32),
            y: DataSource::Immediate(data.read_u8().unwrap() as u32),
            variant: 0xDC,
        },
        0xDD => Op::ChangeLocation {
            index_direction: DataSource::Immediate(data.read_u16::<LittleEndian>().unwrap() as u32),
            x: DataSource::Immediate(data.read_u8().unwrap() as u32),
            y: DataSource::Immediate(data.read_u8().unwrap() as u32),
            variant: 0xDD,
        },
        0xDE => Op::ChangeLocation {
            index_direction: DataSource::Immediate(data.read_u16::<LittleEndian>().unwrap() as u32),
            x: DataSource::Immediate(data.read_u8().unwrap() as u32),
            y: DataSource::Immediate(data.read_u8().unwrap() as u32),
            variant: 0xDE,
        },
        0xDF => Op::ChangeLocation {
            index_direction: DataSource::Immediate(data.read_u16::<LittleEndian>().unwrap() as u32),
            x: DataSource::Immediate(data.read_u8().unwrap() as u32),
            y: DataSource::Immediate(data.read_u8().unwrap() as u32),
            variant: 0xDF,
        },
        0xE0 => Op::ChangeLocation {
            index_direction: DataSource::Immediate(data.read_u16::<LittleEndian>().unwrap() as u32),
            x: DataSource::Immediate(data.read_u8().unwrap() as u32),
            y: DataSource::Immediate(data.read_u8().unwrap() as u32),
            variant: 0xDE,
        },
        0xE1 => Op::ChangeLocation {
            index_direction: DataSource::Immediate(data.read_u16::<LittleEndian>().unwrap() as u32),
            x: DataSource::Immediate(data.read_u8().unwrap() as u32),
            y: DataSource::Immediate(data.read_u8().unwrap() as u32),
            variant: 0xE1,
        },
        0xE2 => Op::ChangeLocation {
            index_direction: DataSource::LocalVar(data.read_u8().unwrap() as usize * 2),
            x: DataSource::LocalVar(data.read_u8().unwrap() as usize * 2),
            y: DataSource::LocalVar(data.read_u8().unwrap() as usize * 2),
            variant: 0xE2,
        },

        _ => panic!("Unknown location op."),
    }
}
