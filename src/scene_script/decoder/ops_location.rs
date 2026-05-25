use std::io::Cursor;
use byteorder::ReadBytesExt;
use crate::destination::Destination;
use crate::GameMode;
use crate::scene_script::ops::Op;
use crate::memory::DataSource;

pub fn op_decode_location(op: u8, data: &mut Cursor<Vec<u8>>, mode: GameMode) -> Op {
    match op {
        // "nextescape"
        0xDC => Op::ChangeLocation {
            destination: Destination::from_cursor(data, mode),
            instant: false,
            queue_different_unknown: true,
        },
        // "nextjump"
        0xDD => Op::ChangeLocation {
            destination: Destination::from_cursor(data, mode),
            instant: true,
            queue_different_unknown: false,
        },
        // "nextmjump"
        0xDE => Op::ChangeLocation {
            destination: Destination::from_cursor(data, mode),
            instant: true,
            queue_different_unknown: false,
        },
        // "mjump"
        0xDF => Op::ChangeLocation {
            destination: Destination::from_cursor(data, mode),
            instant: true,
            queue_different_unknown: false,
        },
        // "jump"
        0xE0 => Op::ChangeLocation {
            destination: Destination::from_cursor(data, mode),
            instant: false,
            queue_different_unknown: false,
        },
        // "djump"
        0xE1 => Op::ChangeLocation {
            destination: Destination::from_cursor(data, mode),
            instant: true,
            queue_different_unknown: false,
        },
        // "vjump"
        0xE2 => Op::ChangeLocationFromMemory {
            byte1: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
            byte2: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
            byte3: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
            byte4: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
        },

        _ => panic!("Unknown location op."),
    }
}
