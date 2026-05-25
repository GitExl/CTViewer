use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use crate::actor::DrawMode;
use crate::character::CharacterId;
use crate::GameMode;
use crate::shared_op::CompareOp;
use crate::scene_script::scene_script_ops::Op;
use crate::scene_script::scene_script_decoder::{ActorRef, InputBinding};
use crate::memory::DataSource;

pub fn op_decode_jump(op: u8, data: &mut Cursor<Vec<u8>>, mode: GameMode) -> Op {
    match op {

        // Relative unconditional jumps.
        // "skip"
        0x10 => Op::Jump {
            offset: data.read_u8().unwrap() as i64 + 1,
        },
        // "back"
        0x11 => Op::Jump {
            offset: -(data.read_u8().unwrap() as i64) + 1,
        },

        // Conditional jumps.
        // 1 byte direct compare with 0x7F0200.
        // "if"
        0x12 => Op::JumpConditional8 {
            lhs: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as usize as i32),
            cmp: CompareOp::from_value(data.read_u8().unwrap() as usize),
            offset: data.read_u8().unwrap() as i64 + 4,
        },
        // 2 byte direct compare with 0x7F0200.
        // "if2"
        0x13 => Op::JumpConditional16 {
            lhs: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
            rhs: DataSource::Immediate(data.read_u16::<LittleEndian>().unwrap() as i32),
            cmp: CompareOp::from_value(data.read_u8().unwrap() as usize),
            offset: data.read_u8().unwrap() as i64 + 5,
        },
        // 1 byte from 0x7F0200 compare with 0x7F0200.
        // "vif"
        0x14 => Op::JumpConditional8 {
            lhs: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
            rhs: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
            cmp: CompareOp::from_value(data.read_u8().unwrap() as usize),
            offset: data.read_u8().unwrap() as i64 + 4,
        },
        // 2 byte from 0x7F0200 compare with 0x7F0200.
        // "vif2"
        0x15 => Op::JumpConditional16 {
            lhs: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
            rhs: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
            cmp: CompareOp::from_value(data.read_u8().unwrap() as usize),
            offset: data.read_u8().unwrap() as i64 + 4,
        },
        // 1 byte direct compare with 0x7F0000 or 0x7F0100.
        // "gif"
        0x16 => {
            let mut lhs = data.read_u8().unwrap() as usize;
            let rhs = data.read_u8().unwrap() as i32;
            let op_value = data.read_u8().unwrap();
            if op_value & 0x80 > 0 {
                lhs += 0x100;
            }
            Op::JumpConditional8 {
                lhs: DataSource::for_global_memory(lhs),
                rhs: DataSource::Immediate(rhs),
                cmp: CompareOp::from_value(op_value as usize & 0x7F),
                offset: data.read_u8().unwrap() as i64 + 4,
            }
        },
        // Less than with storyline counter.
        // "sif"
        0x18 => Op::JumpConditional8 {
            lhs: DataSource::for_global_memory(0x0000),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            cmp: CompareOp::Lt,
            offset: data.read_u8().unwrap() as i64 + 2,
        },
        // Equal with actor result.
        // "case"
        0x1A => Op::JumpConditional8 {
            lhs: DataSource::ActorResult(ActorRef::This),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            cmp: CompareOp::Eq,
            offset: data.read_u8().unwrap() as i64 + 2,
        },

        // If actor is not drawn.
        // TODO: is this really "not on screen", meaning it wasn't drawn beause it was off-screen?
        // "inscreen"
        0x27 => Op::JumpConditionalDrawMode {
            actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            draw_mode: DrawMode::Hidden,
            offset: data.read_u8().unwrap() as i64 + 2,
        },

        // If actor is in battle range (see actor target movement op exec).
        // "binscreen"
        0x28 => Op::JumpConditionalBattleRange {
            actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            offset: data.read_u8().unwrap() as i64 + 2,
        },

        // Jump on input tests.
        // "anykeys"
        0x2D => Op::JumpConditional8 {
            lhs: DataSource::CurrentInput(false),
            rhs: DataSource::Immediate(0),
            cmp: CompareOp::NotEq,
            offset: data.read_u8().unwrap() as i64 + 1,
        },
        // "dashkeys?"
        0x30 => Op::JumpConditional8 {
            lhs: DataSource::CurrentInput(false),
            rhs: DataSource::Input(InputBinding::Dash),
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as i64 + 1,
        },
        // "okkeys"
        0x31 => Op::JumpConditional8 {
            lhs: DataSource::CurrentInput(false),
            rhs: DataSource::Input(InputBinding::Confirm),
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as i64 + 1,
        },
        // "Akeys"
        0x34 => Op::JumpConditional8 {
            lhs: DataSource::CurrentInput(false),
            rhs: DataSource::Input(InputBinding::A),
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as i64 + 1,
        },
        // "Bkeys"
        0x35 => Op::JumpConditional8 {
            lhs: DataSource::CurrentInput(false),
            rhs: DataSource::Input(InputBinding::B),
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as i64 + 1,
        },
        // "Xkeys"
        0x36 => Op::JumpConditional8 {
            lhs: DataSource::CurrentInput(false),
            rhs: DataSource::Input(InputBinding::X),
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as i64 + 1,
        },
        // "Ykeys"
        0x37 => Op::JumpConditional8 {
            lhs: DataSource::CurrentInput(false),
            rhs: DataSource::Input(InputBinding::Y),
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as i64 + 1,
        },
        // "Lkeys"
        0x38 => Op::JumpConditional8 {
            lhs: DataSource::CurrentInput(false),
            rhs: DataSource::Input(InputBinding::L),
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as i64 + 1,
        },
        // "Rkeys"
        0x39 => Op::JumpConditional8 {
            lhs: DataSource::CurrentInput(false),
            rhs: DataSource::Input(InputBinding::R),
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as i64 + 1,
        },

        // Jump on input tests, changed since last test.
        // "dashkeyw"
        0x3B => Op::JumpConditional8 {
            lhs: DataSource::CurrentInput(true),
            rhs: DataSource::Input(InputBinding::Dash),
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as i64 + 1,
        },
        // "okkeyw"
        0x3C => Op::JumpConditional8 {
            lhs: DataSource::CurrentInput(true),
            rhs: DataSource::Input(InputBinding::Confirm),
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as i64 + 1,
        },
        // "Akeys" - dup?
        0x3F => Op::JumpConditional8 {
            lhs: DataSource::CurrentInput(true),
            rhs: DataSource::Input(InputBinding::A),
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as i64 + 1,
        },
        // "Bkeys" - dup?
        0x40 => Op::JumpConditional8 {
            lhs: DataSource::CurrentInput(true),
            rhs: DataSource::Input(InputBinding::B),
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as i64 + 1,
        },
        // "Xkeys" - dup?
        0x41 => Op::JumpConditional8 {
            lhs: DataSource::CurrentInput(true),
            rhs: DataSource::Input(InputBinding::X),
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as i64 + 1,
        },
        // "Ykeys" - dup?
        0x42 => Op::JumpConditional8 {
            lhs: DataSource::CurrentInput(true),
            rhs: DataSource::Input(InputBinding::Y),
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as i64 + 1,
        },
        // "Lkeys" - dup?
        0x43 => Op::JumpConditional8 {
            lhs: DataSource::CurrentInput(true),
            rhs: DataSource::Input(InputBinding::L),
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as i64 + 1,
        },
        // "Rkeys" - dup?
        0x44 => Op::JumpConditional8 {
            lhs: DataSource::CurrentInput(true),
            rhs: DataSource::Input(InputBinding::R),
            cmp: CompareOp::Or,
            offset: data.read_u8().unwrap() as i64 + 1,
        },

        // Inventory-based jumps
        // "itemQ"
        0xC9 => Op::JumpConditional8 {
            lhs: match mode {
                GameMode::Snes => DataSource::ItemCount(data.read_u8().unwrap() as usize),
                GameMode::Pc => DataSource::ItemCount(data.read_u16::<LittleEndian>().unwrap() as usize),
            },
            rhs: DataSource::Immediate(1),
            cmp: CompareOp::GtEq,
            offset: data.read_u8().unwrap() as i64 + 2,
        },
        // "goldQ"
        0xCC => Op::JumpConditional16 {
            lhs: DataSource::GoldCount,
            rhs: DataSource::Immediate(data.read_u16::<LittleEndian>().unwrap() as i32),
            cmp: CompareOp::GtEq,
            offset: data.read_u8().unwrap() as i64 + 3,
        },

        // Party member is active or in reserve.
        // "memberQ"
        0xCF => Op::JumpConditional8 {
            lhs: DataSource::PCIsActiveOrReserve(data.read_u8().unwrap() as CharacterId),
            rhs: DataSource::Immediate(1),
            cmp: CompareOp::GtEq,
            offset: data.read_u8().unwrap() as i64 + 2,
        },
        // Party member is in active party.
        // "partyQ"
        0xD2 => Op::JumpConditional8 {
            lhs: DataSource::PCIsActive(data.read_u8().unwrap() as CharacterId),
            rhs: DataSource::Immediate(1),
            cmp: CompareOp::GtEq,
            offset: data.read_u8().unwrap() as i64 + 2,
        },

        // PC specific ops.
        // 1 byte direct compare with extended memory.
        // "exif"
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
