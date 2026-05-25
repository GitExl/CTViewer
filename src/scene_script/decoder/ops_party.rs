use std::io::Cursor;
use byteorder::ReadBytesExt;
use crate::scene_script::ops::Op;

pub fn op_decode_party(op: u8, data: &mut Cursor<Vec<u8>>) -> Op {
    match op {
        // "memberP"
        0xD0 => Op::PartyMemberAddToReserve {
            pc: data.read_u8().unwrap() as usize,
        },
        // "memberM"
        0xD1 => Op::PartyMemberRemoveFromActive {
            pc: data.read_u8().unwrap() as usize,
        },
        // "partyP"
        0xD3 => Op::PartyMemberAddToActive {
            pc: data.read_u8().unwrap() as usize,
        },
        // "partyM"
        0xD4 => Op::PartyMemberMoveToReserve {
            pc: data.read_u8().unwrap() as usize,
        },
        // "partyMM"
        0xD6 => Op::PartyMemberMoveOutOfParty {
            pc: data.read_u8().unwrap() as usize,
        },
        // "userscroll"
        0xE3 => Op::PartyExploreMode {
            value: data.read_u8().unwrap(),
        },

        _ => panic!("Unknown party op."),
    }
}
