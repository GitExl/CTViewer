use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use crate::actor::DrawMode;
use crate::scene_script::ops::Op;
use crate::scene_script::scene_script_decoder::{ActorRef, InputBinding};
use crate::memory::DataSource;
use crate::scene_script::scene_script::SceneScriptMode;

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
                panic!("Unknown conditional op {:?}", other);
            },
        }
    }
}

pub fn op_decode_jump(op: u8, data: &mut Cursor<Vec<u8>>, mode: SceneScriptMode) -> Op {
    match op {

        // Relative unconditional jumps.
        0x10 => Op::Jump {
            offset: data.read_u8().unwrap() as i64 + 1,
        },
        0x11 => Op::Jump {
            offset: -(data.read_u8().unwrap() as i64) + 1,
        },

        // Conditional jumps.
        // 1 byte direct compare with 0x7F0200.
        0x12 => Op::JumpConditional8 {
            lhs: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as usize as i32),
            cmp: CompareOp::from_value(data.read_u8().unwrap() as usize),
            offset: data.read_u8().unwrap() as i64 + 4,
        },
        // 2 byte direct compare with 0x7F0200.
        0x13 => Op::JumpConditional16 {
            lhs: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
            rhs: DataSource::Immediate(data.read_u16::<LittleEndian>().unwrap() as i32),
            cmp: CompareOp::from_value(data.read_u8().unwrap() as usize),
            offset: data.read_u8().unwrap() as i64 + 5,
        },
        // 1 byte from 0x7F0200 compare with 0x7F0200.
        0x14 => Op::JumpConditional8 {
            lhs: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
            rhs: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
            cmp: CompareOp::from_value(data.read_u8().unwrap() as usize),
            offset: data.read_u8().unwrap() as i64 + 4,
        },
        // 2 byte from 0x7F0200 compare with 0x7F0200.
        0x15 => Op::JumpConditional16 {
            lhs: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
            rhs: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
            cmp: CompareOp::from_value(data.read_u8().unwrap() as usize),
            offset: data.read_u8().unwrap() as i64 + 4,
        },
        // 1 byte direct compare with 0x7F0000 or 0x7F0100.
        0x16 => {
            let mut lhs = data.read_u8().unwrap() as usize;
            let value = data.read_u8().unwrap();
            let op_value = data.read_u8().unwrap();
            if op_value & 0x80 > 0 {
                lhs += 0x100;
            }
            Op::JumpConditional8 {
                lhs: DataSource::for_global_memory(lhs),
                rhs: DataSource::Immediate((value & 0x7F) as i32),
                cmp: CompareOp::from_value(op_value as usize & 0x7F),
                offset: data.read_u8().unwrap() as i64 + 4,
            }
        },
        // Less than with storyline counter.
        0x18 => Op::JumpConditional8 {
            lhs: DataSource::for_global_memory(0x0000),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            cmp: CompareOp::Lt,
            offset: data.read_u8().unwrap() as i64 + 2,
        },
        // Equal with actor result.
        0x1A => Op::JumpConditional8 {
            lhs: DataSource::ActorResult(ActorRef::This),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            cmp: CompareOp::Eq,
            offset: data.read_u8().unwrap() as i64 + 2,
        },
        // If actor is not drawn.
        0x27 => Op::JumpConditionalDrawMode {
            actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            draw_mode: DrawMode::Hidden,
            offset: data.read_u8().unwrap() as i64 + 2,
        },

        // If actor is in battle range (see actor target movement op exec).
        0x28 => Op::JumpConditionalBattleRange {
            actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            offset: data.read_u8().unwrap() as i64 + 2,
        },

        // Jump on input tests.
        0x2D => Op::JumpConditional8 {
            lhs: DataSource::CurrentInput(false),
            rhs: DataSource::Immediate(0),
            cmp: CompareOp::NotEq,
            offset: data.read_u8().unwrap() as i64 + 1,
        },
        0x30 => Op::JumpConditional8 {
            lhs: DataSource::CurrentInput(false),
            rhs: DataSource::Input(InputBinding::Dash),
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as i64 + 1,
        },
        0x31 => Op::JumpConditional8 {
            lhs: DataSource::CurrentInput(false),
            rhs: DataSource::Input(InputBinding::Confirm),
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as i64 + 1,
        },
        0x34 => Op::JumpConditional8 {
            lhs: DataSource::CurrentInput(false),
            rhs: DataSource::Input(InputBinding::A),
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as i64 + 1,
        },
        0x35 => Op::JumpConditional8 {
            lhs: DataSource::CurrentInput(false),
            rhs: DataSource::Input(InputBinding::B),
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as i64 + 1,
        },
        0x36 => Op::JumpConditional8 {
            lhs: DataSource::CurrentInput(false),
            rhs: DataSource::Input(InputBinding::X),
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as i64 + 1,
        },
        0x37 => Op::JumpConditional8 {
            lhs: DataSource::CurrentInput(false),
            rhs: DataSource::Input(InputBinding::Y),
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as i64 + 1,
        },
        0x38 => Op::JumpConditional8 {
            lhs: DataSource::CurrentInput(false),
            rhs: DataSource::Input(InputBinding::L),
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as i64 + 1,
        },
        0x39 => Op::JumpConditional8 {
            lhs: DataSource::CurrentInput(false),
            rhs: DataSource::Input(InputBinding::R),
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as i64 + 1,
        },

        // Jump on input tests, changed since last test.
        0x3B => Op::JumpConditional8 {
            lhs: DataSource::CurrentInput(true),
            rhs: DataSource::Input(InputBinding::Dash),
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as i64 + 1,
        },
        0x3C => Op::JumpConditional8 {
            lhs: DataSource::CurrentInput(true),
            rhs: DataSource::Input(InputBinding::Confirm),
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as i64 + 1,
        },
        0x3F => Op::JumpConditional8 {
            lhs: DataSource::CurrentInput(true),
            rhs: DataSource::Input(InputBinding::A),
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as i64 + 1,
        },
        0x40 => Op::JumpConditional8 {
            lhs: DataSource::CurrentInput(true),
            rhs: DataSource::Input(InputBinding::B),
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as i64 + 1,
        },
        0x41 => Op::JumpConditional8 {
            lhs: DataSource::CurrentInput(true),
            rhs: DataSource::Input(InputBinding::X),
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as i64 + 1,
        },
        0x42 => Op::JumpConditional8 {
            lhs: DataSource::CurrentInput(true),
            rhs: DataSource::Input(InputBinding::Y),
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as i64 + 1,
        },
        0x43 => Op::JumpConditional8 {
            lhs: DataSource::CurrentInput(true),
            rhs: DataSource::Input(InputBinding::L),
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as i64 + 1,
        },
        0x44 => Op::JumpConditional8 {
            lhs: DataSource::CurrentInput(true),
            rhs: DataSource::Input(InputBinding::R),
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as i64 + 1,
        },

        // Inventory-based jumps
        0xC9 => Op::JumpConditional8 {
            lhs: match mode {
                SceneScriptMode::Snes => DataSource::ItemCount(data.read_u8().unwrap() as usize),
                SceneScriptMode::Pc => DataSource::ItemCount(data.read_u16::<LittleEndian>().unwrap() as usize),
            },
            rhs: DataSource::Immediate(1),
            cmp: CompareOp::GtEq,
            offset: data.read_u8().unwrap() as i64 + 2,
        },
        0xCC => Op::JumpConditional16 {
            lhs: DataSource::GoldCount,
            rhs: DataSource::Immediate(data.read_u16::<LittleEndian>().unwrap() as i32),
            cmp: CompareOp::GtEq,
            offset: data.read_u8().unwrap() as i64 + 3,
        },

        // Party member has been recruited.
        0xCF => Op::JumpConditional8 {
            lhs: DataSource::PCIsRecruited,
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            cmp: CompareOp::GtEq,
            offset: data.read_u8().unwrap() as i64 + 2,
        },
        // Party member is in active party.
        0xD2 => Op::JumpConditional8 {
            lhs: DataSource::PCIsActive,
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            cmp: CompareOp::GtEq,
            offset: data.read_u8().unwrap() as i64 + 2,
        },

        // PC specific ops.
        // 1 byte direct compare with extended memory.
        0x6E => {
            let lhs = data.read_u8().unwrap() as usize;
            let value = data.read_u8().unwrap();
            let op_value = data.read_u8().unwrap();
            Op::JumpConditional8 {
                lhs: DataSource::for_extended_memory(lhs),
                rhs: DataSource::Immediate(value as i32),
                cmp: CompareOp::from_value(op_value as usize & 0x7F),
                offset: data.read_u8().unwrap() as i64 + 4,
            }
        },

        _ => panic!("Unknown jump op."),
    }
}
