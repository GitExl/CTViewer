use std::io::Cursor;
use byteorder::ReadBytesExt;
use crate::scene_script::ops::Op;
use crate::scene_script::scene_script_decoder::ActorRef;
use crate::memory::DataSource;

pub fn op_decode_facing(op: u8, data: &mut Cursor<Vec<u8>>) -> Op {
    match op {
        0x0F => Op::ActorFacingSet {
            actor: ActorRef::This,
            facing: DataSource::Immediate(0),
        },
        0x17 => Op::ActorFacingSet {
            actor: ActorRef::This,
            facing: DataSource::Immediate(1),
        },
        0x1B => Op::ActorFacingSet {
            actor: ActorRef::This,
            facing: DataSource::Immediate(2),
        },
        0x1D => Op::ActorFacingSet {
            actor: ActorRef::This,
            facing: DataSource::Immediate(3),
        },
        0x1E => Op::ActorFacingSet {
            actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            facing: DataSource::Immediate(0),
        },
        0x1F => Op::ActorFacingSet {
            actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            facing: DataSource::Immediate(1),
        },
        0x25 => Op::ActorFacingSet {
            actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            facing: DataSource::Immediate(2),
        },
        0x26 => Op::ActorFacingSet {
            actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            facing: DataSource::Immediate(3),
        },
        0x23 => Op::ActorFacingGet {
            actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            source: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
        },
        0x24 => Op::ActorFacingGet {
            actor: ActorRef::PartyMember(data.read_u8().unwrap() as usize),
            source: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
        },
        0xA6 => Op::ActorFacingSet {
            actor: ActorRef::This,
            facing: DataSource::Immediate(data.read_u8().unwrap() as i32),
        },
        0xA7 => Op::ActorFacingSet {
            actor: ActorRef::This,
            facing: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
        },
        0xA8 => Op::ActorSetFacingTowards {
            actor: ActorRef::This,
            to: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
        },
        0xA9 => Op::ActorSetFacingTowards {
            actor: ActorRef::This,
            to: ActorRef::ScriptActorStoredUpper(data.read_u8().unwrap() as usize * 2),
        },

        _ => panic!("Unknown facing op."),
    }
}
