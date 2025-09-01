use std::io::Cursor;
use byteorder::ReadBytesExt;
use crate::scene_script::ops::Op;
use crate::scene_script::scene_script_decoder::{ActorRef, DataSource};

pub fn ops_decode_movement(op: u8, data: &mut Cursor<Vec<u8>>) -> Op {
    match op {
        0x8F => Op::ActorMoveToActor {
            actor: ActorRef::This,
            to_actor: ActorRef::PartyMember(data.read_u8().unwrap() as usize),
            distance: DataSource::Immediate(0),
            update_direction: false,
            animated: false,
            distant: true,
            forever: false,
        },
        0x92 => Op::ActorMoveAtAngle {
            actor: ActorRef::This,
            angle: DataSource::Immediate(data.read_u8().unwrap() as u32),
            distance: DataSource::Immediate(data.read_u8().unwrap() as u32),
            update_direction: true,
            animated: false,
        },
        0x94 => Op::ActorMoveToActor {
            actor: ActorRef::This,
            to_actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            distance: DataSource::Immediate(0),
            update_direction: true,
            animated: false,
            distant: false,
            forever: false,
        },
        0x95 => Op::ActorMoveToActor {
            actor: ActorRef::This,
            to_actor: ActorRef::PartyMember(data.read_u8().unwrap() as usize),
            distance: DataSource::Immediate(0),
            update_direction: true,
            animated: false,
            distant: false,
            forever: false,
        },
        0x96 => Op::ActorMoveTo {
            actor: ActorRef::This,
            x: DataSource::Immediate(data.read_u8().unwrap() as u32),
            y: DataSource::Immediate(data.read_u8().unwrap() as u32),
            distance: DataSource::Immediate(0),
            update_direction: true,
            animated: false,
        },
        0x97 => Op::ActorMoveTo {
            actor: ActorRef::This,
            x: DataSource::LocalVar(data.read_u8().unwrap() as usize * 2),
            y: DataSource::LocalVar(data.read_u8().unwrap() as usize * 2),
            distance: DataSource::Immediate(0),
            update_direction: true,
            animated: false,
        },
        0x98 => Op::ActorMoveToActor {
            actor: ActorRef::This,
            to_actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            distance: DataSource::Immediate(data.read_u8().unwrap() as u32),
            update_direction: true,
            animated: false,
            distant: false,
            forever: false,
        },
        0x99 => Op::ActorMoveToActor {
            actor: ActorRef::This,
            to_actor: ActorRef::PartyMember(data.read_u8().unwrap() as usize),
            distance: DataSource::Immediate(data.read_u8().unwrap() as u32),
            update_direction: true,
            animated: false,
            distant: false,
            forever: false,
        },
        0x9A => Op::ActorMoveTo {
            actor: ActorRef::This,
            x: DataSource::LocalVar(data.read_u8().unwrap() as usize * 2),
            y: DataSource::LocalVar(data.read_u8().unwrap() as usize * 2),
            distance: DataSource::Immediate(data.read_u8().unwrap() as u32),
            update_direction: true,
            animated: false,
        },
        // Same as 0x92, but different in some unknown way?
        0x9C => Op::ActorMoveAtAngle {
            actor: ActorRef::This,
            angle: DataSource::Immediate(data.read_u8().unwrap() as u32),
            distance: DataSource::Immediate(data.read_u8().unwrap() as u32),
            update_direction: false,
            animated: false,
        },
        // Same as 0x9C, but different in some unknown way?
        0x9D => Op::ActorMoveAtAngle {
            actor: ActorRef::This,
            angle: DataSource::LocalVar(data.read_u8().unwrap() as usize * 2),
            distance: DataSource::LocalVar(data.read_u8().unwrap() as usize * 2),
            update_direction: false,
            animated: false,
        },
        // Same as 0x94, but different in some unknown way?
        0x9E => Op::ActorMoveToActor {
            actor: ActorRef::This,
            to_actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            distance: DataSource::Immediate(0),
            update_direction: false,
            animated: false,
            distant: false,
            forever: false,
        },
        // Same as 0x9E, but different in some unknown way?
        0x9F => Op::ActorMoveToActor {
            actor: ActorRef::This,
            to_actor: ActorRef::PartyMember(data.read_u8().unwrap() as usize),
            distance: DataSource::Immediate(0),
            update_direction: false,
            animated: false,
            distant: false,
            forever: false,
        },
        0xA0 => Op::ActorMoveTo {
            actor: ActorRef::This,
            x: DataSource::Immediate(data.read_u8().unwrap() as u32),
            y: DataSource::Immediate(data.read_u8().unwrap() as u32),
            distance: DataSource::Immediate(0),
            update_direction: false,
            animated: false,
        },
        0xA1 => Op::ActorMoveTo {
            actor: ActorRef::This,
            x: DataSource::LocalVar(data.read_u8().unwrap() as usize),
            y: DataSource::LocalVar(data.read_u8().unwrap() as usize),
            distance: DataSource::Immediate(0),
            update_direction: false,
            animated: false,
        },
        0xB5 => Op::ActorMoveToActor {
            actor: ActorRef::This,
            to_actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            distance: DataSource::Immediate(0),
            update_direction: true,
            animated: false,
            distant: false,
            forever: true,
        },
        0xB6 => Op::ActorMoveToActor {
            actor: ActorRef::This,
            to_actor: ActorRef::PartyMember(data.read_u8().unwrap() as usize),
            distance: DataSource::Immediate(0),
            update_direction: true,
            animated: false,
            distant: false,
            forever: true,
        },
        0xD9 => Op::MovePartyTo {
            pc0_x: data.read_u8().unwrap() as i32,
            pc0_y: data.read_u8().unwrap() as i32,
            pc1_x: data.read_u8().unwrap() as i32,
            pc1_y: data.read_u8().unwrap() as i32,
            pc2_x: data.read_u8().unwrap() as i32,
            pc2_y: data.read_u8().unwrap() as i32,
        },

        // Physical movement related.
        0x7A => Op::ActorMoveJump {
            actor: ActorRef::This,
            x: data.read_i8().unwrap() as i32,
            y: data.read_i8().unwrap() as i32,
            height: data.read_u8().unwrap() as u32,
        },
        0x7B => Op::Unknown {
            code: 0x7B,
            data: [data.read_u8().unwrap(), data.read_u8().unwrap(), data.read_u8().unwrap(), data.read_u8().unwrap()],
        },

        _ => panic!("Unknown movement op."),
    }
}
