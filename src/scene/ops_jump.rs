use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use crate::actor::ActorFlags;
use crate::scene::scene_script_decoder::{ActorRef, ConditionalOp, DataValue, InputBinding, Op};

pub fn op_decode_jump(op: u8, data: &mut Cursor<Vec<u8>>) -> Op {
    match op {

        // Relative unconditional jumps.
        0x10 => Op::Jump {
            jump_by: data.read_u8().unwrap() as isize,
        },
        0x11 => Op::Jump {
            jump_by: -(data.read_u8().unwrap() as isize),
        },

        // Conditional jumps.
        // 1 byte direct compare with 0x7F0200.
        0x12 => Op::JumpConditional {
            lhs: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            rhs: DataValue::Immediate(data.read_u8().unwrap() as usize as u32),
            byte_width: 1,
            conditional_op: ConditionalOp::from_value(data.read_u8().unwrap() as usize),
            jump_by: data.read_u8().unwrap() as isize,
        },
        // 2 byte direct compare with 0x7F0200.
        0x13 => Op::JumpConditional {
            lhs: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            rhs: DataValue::Immediate(data.read_u16::<LittleEndian>().unwrap() as u32),
            byte_width: 2,
            conditional_op: ConditionalOp::from_value(data.read_u8().unwrap() as usize),
            jump_by: data.read_u8().unwrap() as isize,
        },
        // 1 byte from 0x7F0200 compare with 0x7F0200.
        0x14 => Op::JumpConditional {
            lhs: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            rhs: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_width: 1,
            conditional_op: ConditionalOp::from_value(data.read_u8().unwrap() as usize),
            jump_by: data.read_u8().unwrap() as isize,
        },
        // 2 byte from 0x7F0200 compare with 0x7F0200.
        0x15 => Op::JumpConditional {
            lhs: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            rhs: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_width: 2,
            conditional_op: ConditionalOp::from_value(data.read_u8().unwrap() as usize),
            jump_by: data.read_u8().unwrap() as isize,
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
                lhs: DataValue::StoredLower(lhs),
                rhs: DataValue::Immediate((value & 0x7F) as u32),
                byte_width: 1,
                conditional_op: ConditionalOp::from_value(op_value as usize & 0x7F),
                jump_by: data.read_u8().unwrap() as isize,
            }
        },
        // Less than with storyline counter.
        0x18 => Op::JumpConditional {
            lhs: DataValue::StoredLower(0x000),
            rhs: DataValue::Immediate(data.read_u8().unwrap() as u32),
            byte_width: 1,
            conditional_op: ConditionalOp::Lt,
            jump_by: data.read_u8().unwrap() as isize,
        },
        // Equal with actor result.
        0x1A => Op::JumpConditional {
            lhs: DataValue::ActorResult(ActorRef::This),
            rhs: DataValue::Immediate(data.read_u8().unwrap() as u32),
            byte_width: 1,
            conditional_op: ConditionalOp::Eq,
            jump_by: data.read_u8().unwrap() as isize,
        },
        // If actor is hidden.
        0x27 => Op::JumpConditional {
            lhs: DataValue::ActorFlag(ActorRef::ScriptActor(data.read_u8().unwrap() as usize * 2), ActorFlags::HIDDEN),
            rhs: DataValue::Immediate(1),
            byte_width: 1,
            conditional_op: ConditionalOp::Eq,
            jump_by: data.read_u8().unwrap() as isize,
        },
        // If actor is in battle.
        0x28 => Op::JumpConditional {
            lhs: DataValue::ActorFlag(ActorRef::ScriptActor(data.read_u8().unwrap() as usize * 2), ActorFlags::IN_BATTLE),
            rhs: DataValue::Immediate(1),
            byte_width: 1,
            conditional_op: ConditionalOp::Eq,
            jump_by: data.read_u8().unwrap() as isize,
        },

        // Jump on input tests.
        0x2D => Op::JumpConditional {
            lhs: DataValue::CurrentInput(false),
            rhs: DataValue::Immediate(0),
            byte_width: 1,
            conditional_op: ConditionalOp::NotEq,
            jump_by: data.read_u8().unwrap() as isize,
        },
        0x30 => Op::JumpConditional {
            lhs: DataValue::CurrentInput(false),
            rhs: DataValue::Input(InputBinding::Dash),
            byte_width: 1,
            conditional_op: ConditionalOp::Or,
            jump_by: data.read_u8().unwrap() as isize,
        },
        0x31 => Op::JumpConditional {
            lhs: DataValue::CurrentInput(false),
            rhs: DataValue::Input(InputBinding::Confirm),
            byte_width: 1,
            conditional_op: ConditionalOp::Or,
            jump_by: data.read_u8().unwrap() as isize,
        },
        0x34 => Op::JumpConditional {
            lhs: DataValue::CurrentInput(false),
            rhs: DataValue::Input(InputBinding::A),
            byte_width: 1,
            conditional_op: ConditionalOp::Or,
            jump_by: data.read_u8().unwrap() as isize,
        },
        0x35 => Op::JumpConditional {
            lhs: DataValue::CurrentInput(false),
            rhs: DataValue::Input(InputBinding::B),
            byte_width: 1,
            conditional_op: ConditionalOp::Or,
            jump_by: data.read_u8().unwrap() as isize,
        },
        0x36 => Op::JumpConditional {
            lhs: DataValue::CurrentInput(false),
            rhs: DataValue::Input(InputBinding::X),
            byte_width: 1,
            conditional_op: ConditionalOp::Or,
            jump_by: data.read_u8().unwrap() as isize,
        },
        0x37 => Op::JumpConditional {
            lhs: DataValue::CurrentInput(false),
            rhs: DataValue::Input(InputBinding::Y),
            byte_width: 1,
            conditional_op: ConditionalOp::Or,
            jump_by: data.read_u8().unwrap() as isize,
        },
        0x38 => Op::JumpConditional {
            lhs: DataValue::CurrentInput(false),
            rhs: DataValue::Input(InputBinding::L),
            byte_width: 1,
            conditional_op: ConditionalOp::Or,
            jump_by: data.read_u8().unwrap() as isize,
        },
        0x39 => Op::JumpConditional {
            lhs: DataValue::CurrentInput(false),
            rhs: DataValue::Input(InputBinding::R),
            byte_width: 1,
            conditional_op: ConditionalOp::Or,
            jump_by: data.read_u8().unwrap() as isize,
        },

        // Jump on input tests, changed since last test.
        0x3B => Op::JumpConditional {
            lhs: DataValue::CurrentInput(true),
            rhs: DataValue::Input(InputBinding::Dash),
            byte_width: 1,
            conditional_op: ConditionalOp::Or,
            jump_by: data.read_u8().unwrap() as isize,
        },
        0x3C => Op::JumpConditional {
            lhs: DataValue::CurrentInput(true),
            rhs: DataValue::Input(InputBinding::Confirm),
            byte_width: 1,
            conditional_op: ConditionalOp::Or,
            jump_by: data.read_u8().unwrap() as isize,
        },
        0x3F => Op::JumpConditional {
            lhs: DataValue::CurrentInput(true),
            rhs: DataValue::Input(InputBinding::A),
            byte_width: 1,
            conditional_op: ConditionalOp::Or,
            jump_by: data.read_u8().unwrap() as isize,
        },
        0x40 => Op::JumpConditional {
            lhs: DataValue::CurrentInput(true),
            rhs: DataValue::Input(InputBinding::B),
            byte_width: 1,
            conditional_op: ConditionalOp::Or,
            jump_by: data.read_u8().unwrap() as isize,
        },
        0x41 => Op::JumpConditional {
            lhs: DataValue::CurrentInput(true),
            rhs: DataValue::Input(InputBinding::X),
            byte_width: 1,
            conditional_op: ConditionalOp::Or,
            jump_by: data.read_u8().unwrap() as isize,
        },
        0x42 => Op::JumpConditional {
            lhs: DataValue::CurrentInput(true),
            rhs: DataValue::Input(InputBinding::Y),
            byte_width: 1,
            conditional_op: ConditionalOp::Or,
            jump_by: data.read_u8().unwrap() as isize,
        },
        0x43 => Op::JumpConditional {
            lhs: DataValue::CurrentInput(true),
            rhs: DataValue::Input(InputBinding::L),
            byte_width: 1,
            conditional_op: ConditionalOp::Or,
            jump_by: data.read_u8().unwrap() as isize,
        },
        0x44 => Op::JumpConditional {
            lhs: DataValue::CurrentInput(true),
            rhs: DataValue::Input(InputBinding::R),
            byte_width: 1,
            conditional_op: ConditionalOp::Or,
            jump_by: data.read_u8().unwrap() as isize,
        },

        _ => panic!("Unknown jump op."),
    }
}
