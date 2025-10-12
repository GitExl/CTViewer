use std::io::Cursor;
use byteorder::ReadBytesExt;
use crate::scene_script::ops::Op;
use crate::scene_script::scene_script::SceneScriptMode;

pub fn op_decode_party(op: u8, data: &mut Cursor<Vec<u8>>, mode: SceneScriptMode) -> Op {
    match op {
        0xD0 => Op::PartyMemberMakeActive {
            pc: data.read_u8().unwrap() as usize,
        },
        0xD1 => Op::PartyMemberAddToReserve {
            pc: data.read_u8().unwrap() as usize,
        },
        0xD3 => Op::PartyMemberRemove {
            pc: data.read_u8().unwrap() as usize,
        },
        0xD4 => Op::PartyMemberRemoveFromActive {
            pc: data.read_u8().unwrap() as usize,
        },
        0xD6 => Op::PartyMemberToReserve {
            pc: data.read_u8().unwrap() as usize,
        },
        0xD5 => Op::PartyMemberEquip {
            pc: data.read_u8().unwrap() as usize,
            item: data.read_u8().unwrap() as usize,
            category: match mode {
                SceneScriptMode::Pc => data.read_u8().unwrap() as usize,
                SceneScriptMode::Snes => 0,
            }
        },
        0xDA => Op::PartyFollow,
        0xE3 => Op::PartyExploreMode {
            value: data.read_u8().unwrap(),
        },

        _ => panic!("Unknown party op."),
    }
}
