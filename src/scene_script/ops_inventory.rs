use std::io::Cursor;
use byteorder::ReadBytesExt;
use crate::scene_script::ops::Op;
use crate::scene_script::scene_script_decoder::{ActorRef, DataRef};

pub fn op_decode_inventory(op: u8, data: &mut Cursor<Vec<u8>>) -> Op {
    match op {
        // Inventory.
        0xC7 => Op::ItemGive {
            actor: ActorRef::This,
            item: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
        },
        0xCA => Op::ItemGive {
            actor: ActorRef::This,
            item: DataRef::Immediate(data.read_u8().unwrap() as u32),
        },
        0xCB => Op::ItemTake {
            actor: ActorRef::This,
            item: DataRef::Immediate(data.read_u8().unwrap() as u32),
        },
        0xCD => Op::GoldGive {
            actor: ActorRef::This,
            amount: DataRef::Immediate(data.read_u8().unwrap() as u32),
        },
        0xCE => Op::GoldTake {
            actor: ActorRef::This,
            amount: DataRef::Immediate(data.read_u8().unwrap() as u32),
        },
        0xD7 => Op::ItemGetAmount {
            item: data.read_u8().unwrap() as usize,
            dest: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
        },

        _ => panic!("Unknown inventory op."),
    }
}
