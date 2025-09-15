use std::io::Cursor;
use byteorder::ReadBytesExt;
use crate::scene_script::ops::Op;
use crate::scene_script::scene_script_decoder::ActorRef;
use crate::scene_script::scene_script_memory::DataSource;

pub fn ops_decode_movement(op: u8, data: &mut Cursor<Vec<u8>>) -> Op {
    match op {
        0x8F => Op::ActorMoveToActor {
            actor: ActorRef::This,
            to_actor: ActorRef::PartyMember(data.read_u8().unwrap() as usize),
            distance: DataSource::Immediate(0),
            update_direction: false,
            animated: true,
            distant: true,
            forever: false,
        },
        0x92 => Op::ActorMoveAtAngle {
            actor: ActorRef::This,
            angle: DataSource::Immediate(data.read_u8().unwrap() as u32),
            steps: DataSource::Immediate(data.read_u8().unwrap() as u32),
            update_direction: true,
            animated: true,
        },
        0x94 => Op::ActorMoveToActor {
            actor: ActorRef::This,
            to_actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            distance: DataSource::Immediate(0),
            update_direction: true,
            animated: true,
            distant: false,
            forever: false,
        },
        0x95 => Op::ActorMoveToActor {
            actor: ActorRef::This,
            to_actor: ActorRef::PartyMember(data.read_u8().unwrap() as usize),
            distance: DataSource::Immediate(0),
            update_direction: true,
            animated: true,
            distant: false,
            forever: false,
        },
        0x96 => Op::ActorMoveTo {
            actor: ActorRef::This,
            x: DataSource::Immediate(data.read_u8().unwrap() as u32),
            y: DataSource::Immediate(data.read_u8().unwrap() as u32),
            steps: None,
            update_direction: true,
            animated: true,
        },
        0x97 => Op::ActorMoveTo {
            actor: ActorRef::This,
            x: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
            y: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
            steps: None,
            update_direction: true,
            animated: true,
        },
        0x98 => Op::ActorMoveToActor {
            actor: ActorRef::This,
            to_actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            distance: DataSource::Immediate(data.read_u8().unwrap() as u32),
            update_direction: true,
            animated: true,
            distant: false,
            forever: false,
        },
        0x99 => Op::ActorMoveToActor {
            actor: ActorRef::This,
            to_actor: ActorRef::PartyMember(data.read_u8().unwrap() as usize),
            distance: DataSource::Immediate(data.read_u8().unwrap() as u32),
            update_direction: true,
            animated: true,
            distant: false,
            forever: false,
        },
        0x9A => Op::ActorMoveTo {
            actor: ActorRef::This,
            x: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
            y: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
            steps: Some(DataSource::Immediate(data.read_u8().unwrap() as u32)),
            update_direction: true,
            animated: true,
        },
        // Same as 0x92, but different in some unknown way?
        0x9C => Op::ActorMoveAtAngle {
            actor: ActorRef::This,
            angle: DataSource::Immediate(data.read_u8().unwrap() as u32),
            steps: DataSource::Immediate(data.read_u8().unwrap() as u32),
            update_direction: false,
            animated: false,
        },
        // Same as 0x9C, but different in some unknown way?
        0x9D => Op::ActorMoveAtAngle {
            actor: ActorRef::This,
            angle: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
            steps: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
            update_direction: false,
            animated: true,
        },
        // Same as 0x94, but different in some unknown way?
        0x9E => Op::ActorMoveToActor {
            actor: ActorRef::This,
            to_actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            distance: DataSource::Immediate(0),
            update_direction: false,
            animated: true,
            distant: false,
            forever: false,
        },
        // Same as 0x9E, but different in some unknown way?
        0x9F => Op::ActorMoveToActor {
            actor: ActorRef::This,
            to_actor: ActorRef::PartyMember(data.read_u8().unwrap() as usize),
            distance: DataSource::Immediate(0),
            update_direction: false,
            animated: true,
            distant: false,
            forever: false,
        },
        0xA0 => Op::ActorMoveTo {
            actor: ActorRef::This,
            x: DataSource::Immediate(data.read_u8().unwrap() as u32),
            y: DataSource::Immediate(data.read_u8().unwrap() as u32),
            steps: None,
            update_direction: false,
            animated: true,
        },
        0xA1 => Op::ActorMoveTo {
            actor: ActorRef::This,
            x: DataSource::for_local_memory(data.read_u8().unwrap() as usize),
            y: DataSource::for_local_memory(data.read_u8().unwrap() as usize),
            steps: None,
            update_direction: false,
            animated: true,
        },
        0xB5 => Op::ActorMoveToActor {
            actor: ActorRef::This,
            to_actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            distance: DataSource::Immediate(0),
            update_direction: true,
            animated: true,
            distant: false,
            forever: true,
        },
        0xB6 => Op::ActorMoveToActor {
            actor: ActorRef::This,
            to_actor: ActorRef::PartyMember(data.read_u8().unwrap() as usize),
            distance: DataSource::Immediate(0),
            update_direction: true,
            animated: true,
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

        // Jumping.
        0x7A => Op::ActorJump {
            actor: ActorRef::This,
            x: data.read_i8().unwrap() as i32,
            y: data.read_i8().unwrap() as i32,
            height: data.read_u8().unwrap() as u32,
        },
        0x7B => Op::ActorJumpUnknown {
            actor: ActorRef::This,
            move_x: data.read_u8().unwrap() as i32,
            move_y: data.read_u8().unwrap() as i32,
            unknown: data.read_u8().unwrap() as u32,
            steps: data.read_u8().unwrap() as u32,
        },

        _ => panic!("Unknown movement op."),
    }
}
