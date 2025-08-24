use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use crate::actor::ActorFlags;
use crate::scene_script::ops::Op;
use crate::scene_script::scene_script_decoder::{ActorRef, DataRef, InputBinding};

/// Conditionals for comparisons.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum CompareOp {
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    And,
    Or,
}

impl CompareOp {
    pub fn from_value(value: usize) -> CompareOp {
        match value {
            0 => CompareOp::Eq,
            1 => CompareOp::NotEq,
            2 => CompareOp::Gt,
            3 => CompareOp::Lt,
            4 => CompareOp::GtEq,
            5 => CompareOp::LtEq,
            6 => CompareOp::And,
            7 => CompareOp::Or,
            other => {
                println!("Unknown conditional op {:?}", other);
                CompareOp::Eq
            },
        }
    }
}

pub fn op_decode_jump(op: u8, data: &mut Cursor<Vec<u8>>) -> Op {
    match op {

        // Relative unconditional jumps.
        0x10 => Op::Jump {
            offset: data.read_u8().unwrap() as isize,
        },
        0x11 => Op::Jump {
            offset: -(data.read_u8().unwrap() as isize),
        },

        // Conditional jumps.
        // 1 byte direct compare with 0x7F0200.
        0x12 => Op::JumpConditional {
            lhs: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            rhs: DataRef::Immediate(data.read_u8().unwrap() as usize as u32),
            width: 1,
            cmp: CompareOp::from_value(data.read_u8().unwrap() as usize),
            offset: data.read_u8().unwrap() as isize,
        },
        // 2 byte direct compare with 0x7F0200.
        0x13 => Op::JumpConditional {
            lhs: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            rhs: DataRef::Immediate(data.read_u16::<LittleEndian>().unwrap() as u32),
            width: 2,
            cmp: CompareOp::from_value(data.read_u8().unwrap() as usize),
            offset: data.read_u8().unwrap() as isize,
        },
        // 1 byte from 0x7F0200 compare with 0x7F0200.
        0x14 => Op::JumpConditional {
            lhs: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            rhs: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            width: 1,
            cmp: CompareOp::from_value(data.read_u8().unwrap() as usize),
            offset: data.read_u8().unwrap() as isize,
        },
        // 2 byte from 0x7F0200 compare with 0x7F0200.
        0x15 => Op::JumpConditional {
            lhs: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            rhs: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            width: 2,
            cmp: CompareOp::from_value(data.read_u8().unwrap() as usize),
            offset: data.read_u8().unwrap() as isize,
        },
        // 1 byte direct compare with 0x7F0000 or 0x7F0100.
        0x16 => {
            let mut lhs = data.read_u8().unwrap() as usize;
            let value = data.read_u8().unwrap();
            let op_value = data.read_u8().unwrap();
            if op_value & 0x80 > 0 {
                lhs += 0x100;
            }
            Op::JumpConditional {
                lhs: DataRef::StoredLower(lhs),
                rhs: DataRef::Immediate((value & 0x7F) as u32),
                width: 1,
                cmp: CompareOp::from_value(op_value as usize & 0x7F),
                offset: data.read_u8().unwrap() as isize,
            }
        },
        // Less than with storyline counter.
        0x18 => Op::JumpConditional {
            lhs: DataRef::StoredLower(0x000),
            rhs: DataRef::Immediate(data.read_u8().unwrap() as u32),
            width: 1,
            cmp: CompareOp::Lt,
            offset: data.read_u8().unwrap() as isize,
        },
        // Equal with actor result.
        0x1A => Op::JumpConditional {
            lhs: DataRef::ActorResult(ActorRef::This),
            rhs: DataRef::Immediate(data.read_u8().unwrap() as u32),
            width: 1,
            cmp: CompareOp::Eq,
            offset: data.read_u8().unwrap() as isize,
        },
        // If actor is hidden.
        0x27 => Op::JumpConditional {
            lhs: DataRef::ActorFlag(ActorRef::ScriptActor(data.read_u8().unwrap() as usize * 2), ActorFlags::HIDDEN),
            rhs: DataRef::Immediate(1),
            width: 1,
            cmp: CompareOp::Eq,
            offset: data.read_u8().unwrap() as isize,
        },
        // If actor is in battle.
        0x28 => Op::JumpConditional {
            lhs: DataRef::ActorFlag(ActorRef::ScriptActor(data.read_u8().unwrap() as usize * 2), ActorFlags::IN_BATTLE),
            rhs: DataRef::Immediate(1),
            width: 1,
            cmp: CompareOp::Eq,
            offset: data.read_u8().unwrap() as isize,
        },

        // Jump on input tests.
        0x2D => Op::JumpConditional {
            lhs: DataRef::CurrentInput(false),
            rhs: DataRef::Immediate(0),
            width: 1,
            cmp: CompareOp::NotEq,
            offset: data.read_u8().unwrap() as isize,
        },
        0x30 => Op::JumpConditional {
            lhs: DataRef::CurrentInput(false),
            rhs: DataRef::Input(InputBinding::Dash),
            width: 1,
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as isize,
        },
        0x31 => Op::JumpConditional {
            lhs: DataRef::CurrentInput(false),
            rhs: DataRef::Input(InputBinding::Confirm),
            width: 1,
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as isize,
        },
        0x34 => Op::JumpConditional {
            lhs: DataRef::CurrentInput(false),
            rhs: DataRef::Input(InputBinding::A),
            width: 1,
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as isize,
        },
        0x35 => Op::JumpConditional {
            lhs: DataRef::CurrentInput(false),
            rhs: DataRef::Input(InputBinding::B),
            width: 1,
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as isize,
        },
        0x36 => Op::JumpConditional {
            lhs: DataRef::CurrentInput(false),
            rhs: DataRef::Input(InputBinding::X),
            width: 1,
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as isize,
        },
        0x37 => Op::JumpConditional {
            lhs: DataRef::CurrentInput(false),
            rhs: DataRef::Input(InputBinding::Y),
            width: 1,
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as isize,
        },
        0x38 => Op::JumpConditional {
            lhs: DataRef::CurrentInput(false),
            rhs: DataRef::Input(InputBinding::L),
            width: 1,
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as isize,
        },
        0x39 => Op::JumpConditional {
            lhs: DataRef::CurrentInput(false),
            rhs: DataRef::Input(InputBinding::R),
            width: 1,
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as isize,
        },

        // Jump on input tests, changed since last test.
        0x3B => Op::JumpConditional {
            lhs: DataRef::CurrentInput(true),
            rhs: DataRef::Input(InputBinding::Dash),
            width: 1,
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as isize,
        },
        0x3C => Op::JumpConditional {
            lhs: DataRef::CurrentInput(true),
            rhs: DataRef::Input(InputBinding::Confirm),
            width: 1,
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as isize,
        },
        0x3F => Op::JumpConditional {
            lhs: DataRef::CurrentInput(true),
            rhs: DataRef::Input(InputBinding::A),
            width: 1,
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as isize,
        },
        0x40 => Op::JumpConditional {
            lhs: DataRef::CurrentInput(true),
            rhs: DataRef::Input(InputBinding::B),
            width: 1,
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as isize,
        },
        0x41 => Op::JumpConditional {
            lhs: DataRef::CurrentInput(true),
            rhs: DataRef::Input(InputBinding::X),
            width: 1,
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as isize,
        },
        0x42 => Op::JumpConditional {
            lhs: DataRef::CurrentInput(true),
            rhs: DataRef::Input(InputBinding::Y),
            width: 1,
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as isize,
        },
        0x43 => Op::JumpConditional {
            lhs: DataRef::CurrentInput(true),
            rhs: DataRef::Input(InputBinding::L),
            width: 1,
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as isize,
        },
        0x44 => Op::JumpConditional {
            lhs: DataRef::CurrentInput(true),
            rhs: DataRef::Input(InputBinding::R),
            width: 1,
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as isize,
        },

        // Inventory-based jumps
        0xC9 => Op::JumpConditional {
            lhs: DataRef::ItemCount(data.read_u8().unwrap() as usize),
            rhs: DataRef::Immediate(1),
            width: 1,
            cmp: CompareOp::GtEq,
            offset: data.read_u8().unwrap() as isize,
        },
        0xCC => Op::JumpConditional {
            lhs: DataRef::GoldCount,
            rhs: DataRef::Immediate(data.read_u16::<LittleEndian>().unwrap() as u32),
            width: 2,
            cmp: CompareOp::GtEq,
            offset: data.read_u8().unwrap() as isize,
        },

        // Party member has been recruited.
        0xCF => Op::JumpConditional {
            lhs: DataRef::PCIsRecruited,
            rhs: DataRef::Immediate(data.read_u8().unwrap() as u32),
            width: 2,
            cmp: CompareOp::GtEq,
            offset: data.read_u8().unwrap() as isize,
        },
        // Party member is in active party.
        0xD2 => Op::JumpConditional {
            lhs: DataRef::PCIsActive,
            rhs: DataRef::Immediate(data.read_u8().unwrap() as u32),
            width: 2,
            cmp: CompareOp::GtEq,
            offset: data.read_u8().unwrap() as isize,
        },

        _ => panic!("Unknown jump op."),
    }
}
