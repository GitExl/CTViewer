use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use crate::destination::Destination;
use crate::facing::Facing;
use crate::scene_script::ops::Op;
use crate::scene_script::scene_script::SceneScriptMode;
use crate::scene_script::scene_script_memory::DataSource;
use crate::util::vec2di32::Vec2Di32;

pub fn op_decode_location(op: u8, data: &mut Cursor<Vec<u8>>, mode: SceneScriptMode) -> Op {
    match op {
        0xDC => Op::ChangeLocation {
            destination: read_destination(data, mode),
            instant: false,
            queue_different_unknown: true,
        },
        0xDD => Op::ChangeLocation {
            destination: read_destination(data, mode),
            instant: true,
            queue_different_unknown: false,
        },
        0xDE => Op::ChangeLocation {
            destination: read_destination(data, mode),
            instant: true,
            queue_different_unknown: false,
        },
        0xDF => Op::ChangeLocation {
            destination: read_destination(data, mode),
            instant: true,
            queue_different_unknown: false,
        },
        0xE0 => Op::ChangeLocation {
            destination: read_destination(data, mode),
            instant: false,
            queue_different_unknown: false,
        },
        0xE1 => Op::ChangeLocation {
            destination: read_destination(data, mode),
            instant: true,
            queue_different_unknown: false,
        },
        0xE2 => Op::ChangeLocationFromMemory {
            byte1: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
            byte2: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
            byte3: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
            byte4: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
        },

        _ => panic!("Unknown location op."),
    }
}

fn read_destination(data: &mut Cursor<Vec<u8>>, mode: SceneScriptMode) -> Destination {
    match mode {
        SceneScriptMode::Snes => {
            let index_facing = data.read_u16::<LittleEndian>().unwrap() as usize;
            let index = index_facing & 0x01FF;
            let facing = index_facing & 0x0600;

            let x = data.read_u8().unwrap() as i32 * 16;
            let y = data.read_u8().unwrap() as i32 * 16;
            let pos = Vec2Di32 { x, y };

            if index >= 0x1F0 && index <= 0x1FF {
                Destination::World {
                    index: index - 0x1F0,
                    pos,
                }
            } else {
                Destination::Scene {
                    index,
                    facing: Facing::from_index(facing),
                    pos,
                }
            }
        },
        SceneScriptMode::Pc => {
            let index = data.read_u16::<LittleEndian>().unwrap() as usize;
            let facing = data.read_u8().unwrap() as usize;

            let x = data.read_u8().unwrap() as i32 * 16;
            let y = data.read_u8().unwrap() as i32 * 16;
            let pos = Vec2Di32 { x, y };

            if index >= 0x1F0 && index <= 0x1FF {
                Destination::World {
                    index: index - 0x1F0,
                    pos,
                }
            } else {
                Destination::Scene {
                    index,
                    facing: Facing::from_index(facing),
                    pos,
                }
            }
        }
    }
}
