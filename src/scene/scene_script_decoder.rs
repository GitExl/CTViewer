use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use crate::actor::ActorFlags;

/// How to wait for script execution.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum CallWaitMode {
    NoWait,
    WaitForCompletion,
    WaitForReturn,
}

/// Type of reference to an actor.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ActorRef {
    This,
    Actor(usize),
    PartyMember(usize),
}

/// Conditionals for comparisons.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ConditionalOp {
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    And,
    Or,
}

impl ConditionalOp {
    pub fn from_value(value: usize) -> ConditionalOp {
        match value {
            0 => ConditionalOp::Eq,
            1 => ConditionalOp::NotEq,
            2 => ConditionalOp::Gt,
            3 => ConditionalOp::Lt,
            4 => ConditionalOp::GtEq,
            5 => ConditionalOp::LtEq,
            6 => ConditionalOp::And,
            7 => ConditionalOp::Or,
            other => {
                println!("Unknown conditional op {:?}", other);
                ConditionalOp::Eq
            },
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum InputBinding {
    Dash,
    Confirm,
    A,
    B,
    X,
    Y,
    L,
    R,
}

/// Source or destination values for data operations.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum DataValue {
    // Immediate value.
    Immediate(u32),

    // Persistent data space, 0x7F0000 to 0x7F0200.
    // Address, byte width.
    Persistent(usize, usize),

    // Temporary data space, 0x7F0200 to 0x7F0400.
    // Address, byte width.
    Temp(usize, usize),

    // The result value of an actor.
    ActorResult(ActorRef),

    // The current character at the party index.
    PartyCharacter(usize),

    // An actor flag.
    ActorFlag(ActorRef, ActorFlags),

    // Button state.
    // Since last check?
    CurrentInput(bool),

    // A specific input.
    Input(InputBinding),
}

/// Opcodes.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Op {
    NOP,
    Yield,
    Call {
        actor: ActorRef,
        priority: usize,
        function_index: usize,
        wait_mode: CallWaitMode,
    },
    UpdateActorFlags {
        actor: ActorRef,
        flags_set: ActorFlags,
        flags_remove: ActorFlags,
    },
    StoreDirection {
        actor: ActorRef,
        direction: usize,
    },
    JumpRelative {
        jump_by: isize,
    },
    JumpConditional {
        lhs: DataValue,
        rhs: DataValue,
        conditional_op: ConditionalOp,
        jump_by: isize,
    },
    Store {
        destination: DataValue,
        source: DataValue,
    },
    LoadCoordinates {
        actor: ActorRef,
        source_x: DataValue,
        source_y: DataValue,
    },
    LoadDirection {
        actor: ActorRef,
        source: DataValue,
    },
    Unimplemented {
        code: u8,
        data: [u8; 4],
    },
    ColorMath {
        // todo
    },
}

pub fn op_decode(data: &mut Cursor<Vec<u8>>) -> Op {
    let op_byte = data.read_u8().unwrap();

    match op_byte {

        // Yield to the function with the next higher priority number.
        // If there is none, simply yield.
        0x00 => Op::Yield,

        // Call function on actor.
        // If current function priority < priority
        //   Saves the current execution position
        //   Calls function in priority slot
        // If current function priority == priority
        //   Do nothing
        // If current function priority > priority
        //   Set the function exit address (?) to point to the function
        //   Only if the exit address is not defined yet.
        // Note that higher priority number == less urgent.
        0x02 => {
            let actor_index = data.read_u8().unwrap() as usize / 2;
            let bits = data.read_u8().unwrap();
            Op::Call {
                actor: ActorRef::Actor(actor_index),
                function_index: (bits & 0x0F) as usize,
                priority: (bits & 0xF0) as usize >> 4,
                wait_mode: CallWaitMode::NoWait,
            }
        },
        // Wait until the other actor completes a more urgent task, then call as in 0x02.
        0x03 => {
            let actor_index = data.read_u8().unwrap() as usize / 2;
            let bits = data.read_u8().unwrap();
            Op::Call {
                actor: ActorRef::Actor(actor_index),
                function_index: (bits & 0x0F) as usize,
                priority: (bits & 0xF0) as usize >> 4,
                wait_mode: CallWaitMode::WaitForCompletion,
            }
        },
        // Wait until the other actor completes a more urgent task, then call as in 0x02,
        // then wait until that function completes.
        0x04 => {
            let actor_index = data.read_u8().unwrap() as usize / 2;
            let bits = data.read_u8().unwrap();
            Op::Call {
                actor: ActorRef::Actor(actor_index),
                function_index: (bits & 0x0F) as usize,
                priority: (bits & 0xF0) as usize >> 4,
                wait_mode: CallWaitMode::WaitForReturn,
            }
        },

        // Same as 0x02, 0x03 and 0x04, but calls the actor of a specific party member.
        0x05 => {
            let party_member_index = data.read_u8().unwrap() as usize / 2;
            let bits = data.read_u8().unwrap();
            Op::Call {
                actor: ActorRef::PartyMember(party_member_index),
                function_index: (bits & 0x0F) as usize,
                priority: (bits & 0xF0) as usize >> 4,
                wait_mode: CallWaitMode::NoWait,
            }
        },
        0x06 => {
            let party_member_index = data.read_u8().unwrap() as usize / 2;
            let bits = data.read_u8().unwrap();
            Op::Call {
                actor: ActorRef::PartyMember(party_member_index),
                function_index: (bits & 0x0F) as usize,
                priority: (bits & 0xF0) as usize >> 4,
                wait_mode: CallWaitMode::WaitForCompletion,
            }
        },
        0x07 => {
            let party_member_index = data.read_u8().unwrap() as usize / 2;
            let bits = data.read_u8().unwrap();
            Op::Call {
                actor: ActorRef::PartyMember(party_member_index),
                function_index: (bits & 0x0F) as usize,
                priority: (bits & 0xF0) as usize >> 4,
                wait_mode: CallWaitMode::WaitForReturn,
            }
        },

        // Enable/disable this actor being able to be touched.
        0x08 => Op::UpdateActorFlags {
            actor: ActorRef::This,
            flags_set: ActorFlags::TOUCHABLE,
            flags_remove: ActorFlags::empty(),
        },
        0x09 => Op::UpdateActorFlags {
            actor: ActorRef::This,
            flags_set: ActorFlags::TOUCHABLE,
            flags_remove: ActorFlags::empty(),
        },

        // Disable and hide another actor.
        0x0A => Op::UpdateActorFlags {
            actor: ActorRef::Actor(data.read_u8().unwrap() as usize / 2),
            flags_set: ActorFlags::DISABLED,
            flags_remove: ActorFlags::VISIBLE,
        },

        // Disable/enable script execution.
        0x0B => Op::UpdateActorFlags {
            actor: ActorRef::Actor(data.read_u8().unwrap() as usize / 2),
            flags_set: ActorFlags::DISABLED,
            flags_remove: ActorFlags::empty(),
        },
        0x0C => {
            Op::UpdateActorFlags {
                actor: ActorRef::Actor(data.read_u8().unwrap() as usize / 2),
                flags_set: ActorFlags::empty(),
                flags_remove: ActorFlags::DISABLED,
            }
        },

        // Set actor collision properties.
        0x0D => {
            let flags = data.read_u8().unwrap();
            let mut flags_set = ActorFlags::empty();
            let mut flags_remove = ActorFlags::empty();

            if flags & 0x01 > 0 {
                flags_set.set(ActorFlags::COLLISION_TILE, true);
            } else {
                flags_remove.set(ActorFlags::COLLISION_TILE, true);
            }
            if flags & 0x02 > 0 {
                flags_set.set(ActorFlags::COLLISION_PC, true);
            } else {
                flags_remove.set(ActorFlags::COLLISION_PC, true);
            }

            Op::UpdateActorFlags {
                actor: ActorRef::This,
                flags_set,
                flags_remove,
            }
        },

        // Set actor movement properties.
        0x0E => {
            let flags = data.read_u8().unwrap();
            let mut flags_set = ActorFlags::empty();
            let mut flags_remove = ActorFlags::empty();

            if flags & 0x01 > 0 {
                flags_set.set(ActorFlags::MOVE_ONTO_TILE, true);
            } else {
                flags_remove.set(ActorFlags::MOVE_ONTO_TILE, true);
            }
            if flags & 0x02 > 0 {
                flags_set.set(ActorFlags::MOVE_ONTO_OBJECT, true);
            } else {
                flags_remove.set(ActorFlags::MOVE_ONTO_OBJECT, true);
            }

            Op::UpdateActorFlags {
                actor: ActorRef::This,
                flags_set,
                flags_remove,
            }
        },

        // Store values.
        // Set actor result from 0x7F0200.
        0x19 => Op::Store {
            destination: DataValue::ActorResult(ActorRef::This),
            source: DataValue::Temp(data.read_u8().unwrap() as usize * 2, 1),
        },
        // Set actor result from 0x7F0000.
        0x1C => Op::Store {
            destination: DataValue::ActorResult(ActorRef::This),
            source: DataValue::Persistent(data.read_u8().unwrap() as usize, 1),
        },
        // Set what character the first party member is to 0x7F0200.
        0x20 => Op::Store {
            destination: DataValue::Temp(data.read_u8().unwrap() as usize * 2, 1),
            source: DataValue::PartyCharacter(0),
        },

        // Actor coordinates.
        // From actor.
        0x21 => Op::LoadCoordinates {
            actor: ActorRef::Actor(data.read_u8().unwrap() as usize / 2),
            source_x: DataValue::Temp(data.read_u8().unwrap() as usize * 2, 1),
            source_y: DataValue::Temp(data.read_u8().unwrap() as usize * 2, 1),
        },
        // From party member actor.
        0x22 => Op::LoadCoordinates {
            actor: ActorRef::PartyMember(data.read_u8().unwrap() as usize),
            source_x: DataValue::Temp(data.read_u8().unwrap() as usize * 2, 1),
            source_y: DataValue::Temp(data.read_u8().unwrap() as usize * 2, 1),
        },

        // Direction.
        0x0F => Op::StoreDirection {
            actor: ActorRef::This,
            direction: 0,
        },
        0x17 => Op::StoreDirection {
            actor: ActorRef::This,
            direction: 1,
        },
        0x1B => Op::StoreDirection {
            actor: ActorRef::This,
            direction: 2,
        },
        0x1D => Op::StoreDirection {
            actor: ActorRef::This,
            direction: 3,
        },
        0x1E => Op::StoreDirection {
            actor: ActorRef::Actor(data.read_u8().unwrap() as usize / 2),
            direction: 0,
        },
        0x1F => Op::StoreDirection {
            actor: ActorRef::Actor(data.read_u8().unwrap() as usize / 2),
            direction: 1,
        },
        0x25 => Op::StoreDirection {
            actor: ActorRef::Actor(data.read_u8().unwrap() as usize / 2),
            direction: 2,
        },
        0x26 => Op::StoreDirection {
            actor: ActorRef::Actor(data.read_u8().unwrap() as usize / 2),
            direction: 3,
        },
        0x23 => Op::LoadDirection {
            actor: ActorRef::Actor(data.read_u8().unwrap() as usize / 2),
            source: DataValue::Temp(data.read_u8().unwrap() as usize * 2, 1),
        },
        0x24 => Op::LoadDirection {
            actor: ActorRef::PartyMember(data.read_u8().unwrap() as usize),
            source: DataValue::Temp(data.read_u8().unwrap() as usize * 2, 1),
        },

        // Relative code jump.
        0x10 => Op::JumpRelative {
            jump_by: data.read_u8().unwrap() as isize,
        },
        0x11 => Op::JumpRelative {
            jump_by: -(data.read_u8().unwrap() as isize),
        },

        // Conditional code jumps.
        // 1 byte direct compare with 0x7F0200.
        0x12 => Op::JumpConditional {
            lhs: DataValue::Temp(data.read_u8().unwrap() as usize * 2, 1),
            rhs: DataValue::Immediate(data.read_u8().unwrap() as usize as u32),
            conditional_op: ConditionalOp::from_value(data.read_u8().unwrap() as usize),
            jump_by: data.read_u8().unwrap() as isize,
        },
        // 2 byte direct compare with 0x7F0200.
        0x13 => Op::JumpConditional {
            lhs: DataValue::Temp(data.read_u8().unwrap() as usize * 2, 2),
            rhs: DataValue::Immediate(data.read_u16::<LittleEndian>().unwrap() as u32),
            conditional_op: ConditionalOp::from_value(data.read_u8().unwrap() as usize),
            jump_by: data.read_u8().unwrap() as isize,
        },
        // 1 byte from 0x7F0200 compare with 0x7F0200.
        0x14 => Op::JumpConditional {
            lhs: DataValue::Temp(data.read_u8().unwrap() as usize * 2, 1),
            rhs: DataValue::Temp(data.read_u8().unwrap() as usize * 2, 1),
            conditional_op: ConditionalOp::from_value(data.read_u8().unwrap() as usize),
            jump_by: data.read_u8().unwrap() as isize,
        },
        // 2 byte from 0x7F0200 compare with 0x7F0200.
        0x15 => Op::JumpConditional {
            lhs: DataValue::Temp(data.read_u8().unwrap() as usize * 2, 2),
            rhs: DataValue::Temp(data.read_u8().unwrap() as usize * 2, 2),
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
                lhs: DataValue::Persistent(lhs, 1),
                rhs: DataValue::Immediate((value & 0x7F) as u32),
                conditional_op: ConditionalOp::from_value(op_value as usize & 0x7F),
                jump_by: data.read_u8().unwrap() as isize,
            }
        },
        // Less than with storyline counter.
        0x18 => Op::JumpConditional {
            lhs: DataValue::Persistent(0x000, 1),
            rhs: DataValue::Immediate(data.read_u8().unwrap() as u32),
            conditional_op: ConditionalOp::Lt,
            jump_by: data.read_u8().unwrap() as isize,
        },
        // Equal with actor result.
        0x1A => Op::JumpConditional {
            lhs: DataValue::ActorResult(ActorRef::This),
            rhs: DataValue::Immediate(data.read_u8().unwrap() as u32),
            conditional_op: ConditionalOp::Eq,
            jump_by: data.read_u8().unwrap() as isize,
        },
        // If actor is hidden.
        0x27 => Op::JumpConditional {
            lhs: DataValue::ActorFlag(ActorRef::Actor(data.read_u8().unwrap() as usize * 2), ActorFlags::VISIBLE),
            rhs: DataValue::Immediate(0),
            conditional_op: ConditionalOp::Eq,
            jump_by: data.read_u8().unwrap() as isize,
        },
        // If actor is in battle.
        0x28 => Op::JumpConditional {
            lhs: DataValue::ActorFlag(ActorRef::Actor(data.read_u8().unwrap() as usize * 2), ActorFlags::IN_BATTLE),
            rhs: DataValue::Immediate(1),
            conditional_op: ConditionalOp::Eq,
            jump_by: data.read_u8().unwrap() as isize,
        },

        // Palette.
        0x2E => Op::ColorMath {
            // todo
        },

        // Input tests.
        0x2D => Op::JumpConditional {
            lhs: DataValue::CurrentInput(false),
            rhs: DataValue::Immediate(0),
            conditional_op: ConditionalOp::NotEq,
            jump_by: data.read_u8().unwrap() as isize,
        },
        0x30 => Op::JumpConditional {
            lhs: DataValue::CurrentInput(false),
            rhs: DataValue::Input(InputBinding::Dash),
            conditional_op: ConditionalOp::Or,
            jump_by: data.read_u8().unwrap() as isize,
        },
        0x31 => Op::JumpConditional {
            lhs: DataValue::CurrentInput(false),
            rhs: DataValue::Input(InputBinding::Confirm),
            conditional_op: ConditionalOp::Or,
            jump_by: data.read_u8().unwrap() as isize,
        },
        0x34 => Op::JumpConditional {
            lhs: DataValue::CurrentInput(false),
            rhs: DataValue::Input(InputBinding::A),
            conditional_op: ConditionalOp::Or,
            jump_by: data.read_u8().unwrap() as isize,
        },
        0x35 => Op::JumpConditional {
            lhs: DataValue::CurrentInput(false),
            rhs: DataValue::Input(InputBinding::B),
            conditional_op: ConditionalOp::Or,
            jump_by: data.read_u8().unwrap() as isize,
        },
        0x36 => Op::JumpConditional {
            lhs: DataValue::CurrentInput(false),
            rhs: DataValue::Input(InputBinding::X),
            conditional_op: ConditionalOp::Or,
            jump_by: data.read_u8().unwrap() as isize,
        },
        0x37 => Op::JumpConditional {
            lhs: DataValue::CurrentInput(false),
            rhs: DataValue::Input(InputBinding::Y),
            conditional_op: ConditionalOp::Or,
            jump_by: data.read_u8().unwrap() as isize,
        },
        0x38 => Op::JumpConditional {
            lhs: DataValue::CurrentInput(false),
            rhs: DataValue::Input(InputBinding::L),
            conditional_op: ConditionalOp::Or,
            jump_by: data.read_u8().unwrap() as isize,
        },
        0x39 => Op::JumpConditional {
            lhs: DataValue::CurrentInput(false),
            rhs: DataValue::Input(InputBinding::R),
            conditional_op: ConditionalOp::Or,
            jump_by: data.read_u8().unwrap() as isize,
        },
        // Input tests, changed since last test.
        0x3B => Op::JumpConditional {
            lhs: DataValue::CurrentInput(true),
            rhs: DataValue::Input(InputBinding::Dash),
            conditional_op: ConditionalOp::Or,
            jump_by: data.read_u8().unwrap() as isize,
        },
        0x3C => Op::JumpConditional {
            lhs: DataValue::CurrentInput(true),
            rhs: DataValue::Input(InputBinding::Confirm),
            conditional_op: ConditionalOp::Or,
            jump_by: data.read_u8().unwrap() as isize,
        },
        0x3F => Op::JumpConditional {
            lhs: DataValue::CurrentInput(true),
            rhs: DataValue::Input(InputBinding::A),
            conditional_op: ConditionalOp::Or,
            jump_by: data.read_u8().unwrap() as isize,
        },
        0x40 => Op::JumpConditional {
            lhs: DataValue::CurrentInput(true),
            rhs: DataValue::Input(InputBinding::B),
            conditional_op: ConditionalOp::Or,
            jump_by: data.read_u8().unwrap() as isize,
        },
        0x41 => Op::JumpConditional {
            lhs: DataValue::CurrentInput(true),
            rhs: DataValue::Input(InputBinding::X),
            conditional_op: ConditionalOp::Or,
            jump_by: data.read_u8().unwrap() as isize,
        },
        0x42 => Op::JumpConditional {
            lhs: DataValue::CurrentInput(true),
            rhs: DataValue::Input(InputBinding::Y),
            conditional_op: ConditionalOp::Or,
            jump_by: data.read_u8().unwrap() as isize,
        },
        0x43 => Op::JumpConditional {
            lhs: DataValue::CurrentInput(true),
            rhs: DataValue::Input(InputBinding::L),
            conditional_op: ConditionalOp::Or,
            jump_by: data.read_u8().unwrap() as isize,
        },
        0x44 => Op::JumpConditional {
            lhs: DataValue::CurrentInput(true),
            rhs: DataValue::Input(InputBinding::R),
            conditional_op: ConditionalOp::Or,
            jump_by: data.read_u8().unwrap() as isize,
        },

        // Ascii text related (???)
        0x29 => Op::Unimplemented {
            code: 0x29,
            data: [data.read_u8().unwrap(), 0, 0, 0],
        },
        0x2A => Op::Unimplemented {
            code: 0x2A,
            data: [0, 0, 0, 0],
        },
        0x2B => Op::Unimplemented {
            code: 0x2B,
            data: [0, 0, 0, 0],
        },
        0x2C => Op::Unimplemented {
            code: 0x2C,
            data: [data.read_u8().unwrap(), data.read_u8().unwrap(), 0, 0],
        },

        _ => {
            println!("Decoding unimplemented opcode 0x{:02X} as NOP", op_byte);
            Op::NOP
        },
    }
}
