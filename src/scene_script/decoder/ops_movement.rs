use std::io::Cursor;
use byteorder::ReadBytesExt;
use crate::scene_script::ops::Op;
use crate::scene_script::scene_script_decoder::ActorRef;
use crate::memory::DataSource;

pub fn ops_decode_movement(op: u8, data: &mut Cursor<Vec<u8>>) -> Op {
    match op {
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

        0x8F => Op::ActorMoveToActor {
            to_actor: ActorRef::ActivePartyIndex(data.read_u8().unwrap() as usize),
            script_cycle_count: None,
            update_facing: true,
            animated: true,
            into_battle_range: true,
            forever: false,
        },
        0x92 => Op::ActorMoveAtAngle {
            angle: DataSource::Immediate(data.read_u8().unwrap() as i32),
            steps: DataSource::Immediate(data.read_u8().unwrap() as i32),
            update_facing: true,
            animated: true,
        },
        0x94 => Op::ActorMoveToActor {
            to_actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            script_cycle_count: None,
            update_facing: true,
            animated: true,
            into_battle_range: false,
            forever: false,
        },
        0x95 => Op::ActorMoveToActor {
            to_actor: ActorRef::ActivePartyIndex(data.read_u8().unwrap() as usize),
            script_cycle_count: None,
            update_facing: true,
            animated: true,
            into_battle_range: false,
            forever: false,
        },
        0x96 => Op::ActorMoveToTile {
            x: DataSource::Immediate(data.read_u8().unwrap() as i32),
            y: DataSource::Immediate(data.read_u8().unwrap() as i32),
            steps: None,
            update_facing: true,
            animated: true,
        },
        0x97 => Op::ActorMoveToTile {
            x: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
            y: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
            steps: None,
            update_facing: true,
            animated: true,
        },
        0x98 => Op::ActorMoveToActor {
            to_actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            script_cycle_count: Some(data.read_u8().unwrap() as u32),
            update_facing: true,
            animated: true,
            into_battle_range: false,
            forever: false,
        },
        0x99 => Op::ActorMoveToActor {
            to_actor: ActorRef::ActivePartyIndex(data.read_u8().unwrap() as usize),
            script_cycle_count: Some(data.read_u8().unwrap() as u32),
            update_facing: true,
            animated: true,
            into_battle_range: false,
            forever: false,
        },
        0x9A => Op::ActorMoveToTile {
            x: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
            y: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
            steps: Some(DataSource::Immediate(data.read_u8().unwrap() as i32)),
            update_facing: true,
            animated: true,
        },
        // Same as 0x92, but different in some unknown way?
        0x9C => Op::ActorMoveAtAngle {
            angle: DataSource::Immediate(data.read_u8().unwrap() as i32),
            steps: DataSource::Immediate(data.read_u8().unwrap() as i32),
            update_facing: false,
            animated: false,
        },
        // Same as 0x9C, but different in some unknown way?
        0x9D => Op::ActorMoveAtAngle {
            angle: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
            steps: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
            update_facing: false,
            animated: true,
        },
        0x9E => Op::ActorMoveToActor {
            to_actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            script_cycle_count: None,
            update_facing: false,
            animated: false,
            into_battle_range: false,
            forever: false,
        },
        // Same as 0x9E, but different in some unknown way?
        0x9F => Op::ActorMoveToActor {
            to_actor: ActorRef::ActivePartyIndex(data.read_u8().unwrap() as usize),
            script_cycle_count: None,
            update_facing: false,
            animated: false,
            into_battle_range: false,
            forever: false,
        },
        0xA0 => Op::ActorMoveToTile {
            x: DataSource::Immediate(data.read_u8().unwrap() as i32),
            y: DataSource::Immediate(data.read_u8().unwrap() as i32),
            steps: None,
            update_facing: false,
            animated: true,
        },
        0xA1 => Op::ActorMoveToTile {
            x: DataSource::for_local_memory(data.read_u8().unwrap() as usize),
            y: DataSource::for_local_memory(data.read_u8().unwrap() as usize),
            steps: None,
            update_facing: false,
            animated: true,
        },
        0xB5 => Op::ActorMoveToActor {
            to_actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            script_cycle_count: None,
            update_facing: true,
            animated: true,
            into_battle_range: false,
            forever: true,
        },
        0xB6 => Op::ActorMoveToActor {
            to_actor: ActorRef::ActivePartyIndex(data.read_u8().unwrap() as usize),
            script_cycle_count: None,
            update_facing: true,
            animated: true,
            into_battle_range: false,
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

        _ => panic!("Unknown movement op."),
    }
}
