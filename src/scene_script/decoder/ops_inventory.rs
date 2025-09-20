use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use crate::scene_script::ops::Op;
use crate::scene_script::scene_script::SceneScriptMode;
use crate::scene_script::scene_script_decoder::ActorRef;
use crate::scene_script::scene_script_memory::DataSource;

pub fn op_decode_inventory(op: u8, data: &mut Cursor<Vec<u8>>, mode: SceneScriptMode) -> Op {
    match op {
        // Inventory.
        0xC7 => Op::ItemGive {
            actor: ActorRef::This,
            item: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
            category: match mode {
                SceneScriptMode::Pc => data.read_u8().unwrap() as usize,
                SceneScriptMode::Snes => 0,
            }
        },
        0xCA => Op::ItemGive {
            actor: ActorRef::This,
            item: DataSource::Immediate(data.read_u8().unwrap() as u32),
            category: match mode {
                SceneScriptMode::Pc => data.read_u8().unwrap() as usize,
                SceneScriptMode::Snes => 0,
            }
        },
        0xCB => Op::ItemTake {
            actor: ActorRef::This,
            item: DataSource::Immediate(data.read_u8().unwrap() as u32),
            category: match mode {
                SceneScriptMode::Pc => data.read_u8().unwrap() as usize,
                SceneScriptMode::Snes => 0,
            }
        },
        0xCD => Op::GoldGive {
            actor: ActorRef::This,
            amount: DataSource::Immediate(data.read_u16::<LittleEndian>().unwrap() as u32),
        },
        0xCE => Op::GoldTake {
            actor: ActorRef::This,
            amount: DataSource::Immediate(data.read_u16::<LittleEndian>().unwrap() as u32),
        },
        0xD7 => Op::ItemGetAmount {
            item: data.read_u8().unwrap() as usize,
            category: match mode {
                SceneScriptMode::Pc => data.read_u8().unwrap() as usize,
                SceneScriptMode::Snes => 0,
            },
            dest: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
        },

        _ => panic!("Unknown inventory op."),
    }
}
