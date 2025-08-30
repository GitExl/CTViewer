use std::io::Cursor;
use byteorder::ReadBytesExt;
use crate::scene_script::ops::Op;
use crate::scene_script::scene_script_decoder::{ActorRef, DataRef};

pub fn op_decode_animation(op: u8, data: &mut Cursor<Vec<u8>>) -> Op {
    match op {
        0xAA => Op::Animate {
            actor: ActorRef::This,
            animation: DataRef::Immediate(data.read_u8().unwrap() as u32),
            wait: false,
            run: true,
            loops: DataRef::Immediate(0xFFFFFFFF),
        },
        0xAB => Op::Animate {
            actor: ActorRef::This,
            animation: DataRef::Immediate(data.read_u8().unwrap() as u32),
            wait: true,
            run: true,
            loops: DataRef::Immediate(0),
        },
        // "Also writes 0 to object address 0x1780 instead of 1".
        0xAE => Op::Animate {
            actor: ActorRef::This,
            animation: DataRef::Immediate(0),
            wait: false,
            run: true,
            loops: DataRef::Immediate(0xFFFFFFFF),
        },
        0xB3 => Op::Animate {
            actor: ActorRef::This,
            animation: DataRef::Immediate(0),
            wait: false,
            run: true,
            loops: DataRef::Immediate(0xFFFFFFFF),
        },
        0xB4 => Op::Animate {
            actor: ActorRef::This,
            animation: DataRef::Immediate(1),
            wait: false,
            run: true,
            loops: DataRef::Immediate(0xFFFFFFFF),
        },
        0xB7 => Op::Animate {
            actor: ActorRef::This,
            animation: DataRef::Immediate(data.read_u8().unwrap() as u32),
            wait: true,
            run: true,
            loops: DataRef::Immediate(data.read_u8().unwrap() as u32),
        },

        // Limit active animations?
        0x47 => Op::AnimationLimit {
            limit: data.read_u8().unwrap(),
        },

        _ => panic!("Unknown animate op."),
    }
}
