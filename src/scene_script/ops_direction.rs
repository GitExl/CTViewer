use std::io::Cursor;
use byteorder::ReadBytesExt;
use crate::scene_script::ops::Op;
use crate::scene_script::scene_script_decoder::{ActorRef, DataSource};

pub fn op_decode_direction(op: u8, data: &mut Cursor<Vec<u8>>) -> Op {
    match op {
        0x0F => Op::ActorSetDirection {
            actor: ActorRef::This,
            direction: DataSource::Immediate(0),
        },
        0x17 => Op::ActorSetDirection {
            actor: ActorRef::This,
            direction: DataSource::Immediate(1),
        },
        0x1B => Op::ActorSetDirection {
            actor: ActorRef::This,
            direction: DataSource::Immediate(2),
        },
        0x1D => Op::ActorSetDirection {
            actor: ActorRef::This,
            direction: DataSource::Immediate(3),
        },
        0x1E => Op::ActorSetDirection {
            actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            direction: DataSource::Immediate(0),
        },
        0x1F => Op::ActorSetDirection {
            actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            direction: DataSource::Immediate(1),
        },
        0x25 => Op::ActorSetDirection {
            actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            direction: DataSource::Immediate(2),
        },
        0x26 => Op::ActorSetDirection {
            actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            direction: DataSource::Immediate(3),
        },
        0x23 => Op::ActorDirectionGet {
            actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            source: DataSource::LocalVar(data.read_u8().unwrap() as usize * 2),
        },
        0x24 => Op::ActorDirectionGet {
            actor: ActorRef::PartyMember(data.read_u8().unwrap() as usize),
            source: DataSource::LocalVar(data.read_u8().unwrap() as usize * 2),
        },
        0xA6 => Op::ActorSetDirection {
            actor: ActorRef::This,
            direction: DataSource::Immediate(data.read_u8().unwrap() as u32),
        },
        0xA7 => Op::ActorSetDirection {
            actor: ActorRef::This,
            direction: DataSource::LocalVar(data.read_u8().unwrap() as usize * 2),
        },
        0xA8 => Op::ActorSetDirectionTowards {
            actor: ActorRef::This,
            to: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
        },
        0xA9 => Op::ActorSetDirectionTowards {
            actor: ActorRef::This,
            to: ActorRef::ScriptActorStoredUpper(data.read_u8().unwrap() as usize * 2),
        },

        _ => panic!("Unknown direction op."),
    }
}
