use std::io::Cursor;
use byteorder::ReadBytesExt;
use crate::scene_script::scene_script_ops::Op;

pub fn op_decode_party(op: u8, data: &mut Cursor<Vec<u8>>) -> Op {
    match op {
        // "memberP", party action 0x04
        0xD0 => Op::PartyMemberAddToReserve {
            pc: data.read_u8().unwrap() as usize,
        },
        // "memberM", party action 0x05
        0xD1 => Op::PartyMemberRemoveFromActive {
            pc: data.read_u8().unwrap() as usize,
        },
        // "partyP", party action 0x01
        0xD3 => Op::PartyMemberAddToActive {
            pc: data.read_u8().unwrap() as usize,
        },
        // "partyM", party action 0x02
        0xD4 => Op::PartyMemberMoveToReserve {
            pc: data.read_u8().unwrap() as usize,
        },
        // "partyMM", no party action
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
