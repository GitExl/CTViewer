use std::io::Cursor;
use byteorder::ReadBytesExt;
use crate::scene_script::scene_script_ops::Op;
use crate::scene_script::scene_script_decoder::ActorRef;
use crate::memory::DataSource;

pub fn op_decode_animation(op: u8, data: &mut Cursor<Vec<u8>>) -> Op {
    match op {
        // "animeset"
        0xAA => Op::Animation {
            actor: ActorRef::This,
            animation: DataSource::Immediate(data.read_u8().unwrap() as i32),
        },
        // "anime"
        0xAB => Op::AnimationLoopCount {
            actor: ActorRef::This,
            animation: DataSource::Immediate(data.read_u8().unwrap() as i32),
            loops: DataSource::Immediate(1),
        },
        // "shape"
        0xAC => Op::AnimationStaticFrame {
            actor: ActorRef::This,
            frame: DataSource::Immediate(data.read_u8().unwrap() as i32),
        },
        // "autoanime"
        0xAE => Op::AnimationReset {
            actor: ActorRef::This,
        },
        // "animeset0"
        0xB3 => Op::Animation {
            actor: ActorRef::This,
            animation: DataSource::Immediate(0),
        },
        // "animeset1"
        0xB4 => Op::Animation {
            actor: ActorRef::This,
            animation: DataSource::Immediate(1),
        },
        // "animerepeat"
        0xB7 => Op::AnimationLoopCount {
            actor: ActorRef::This,
            animation: DataSource::Immediate(data.read_u8().unwrap() as i32),
            loops: DataSource::Immediate(data.read_u8().unwrap() as i32),
        },

        // Limit active animations, but we don't have to deal with this because we are not an SNES?
        0x47 => Op::NOP,

        _ => panic!("Unknown animate op."),
    }
}
