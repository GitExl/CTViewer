use std::io::{Cursor, Read};
use byteorder::{LittleEndian, ReadBytesExt};
use crate::actor::ActorFlags;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum BitMathOp {
    Or,
    And,
    Xor,
    ShiftLeft,
    ShiftRight,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ByteMathOp {
    Add,
    Subtract,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ChararacterType {
    PC,
    NPC,
    Monster,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ColorMathMode {
    Additive,
    Subtractive,
}

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

    // Temporary memory from 0x7E0000 to 0x7E0100.
    Temp(usize),

    // Persistently stored space from 0x7F0000 to 0x7F0200.
    StoredLower(usize),

    // Persistently stored space from 0x7F0200 to 0x7F400.
    StoredUpper(usize),

    // Entire upper space from 0x7F0000 to 0x7FFFFF.
    Upper(usize),

    // The result value of an actor.
    ActorResult(ActorRef),

    // The current character at the party index.
    PartyCharacter(usize),

    // A flag of an actor.
    ActorFlag(ActorRef, ActorFlags),

    // Button state.
    // Since last check?
    CurrentInput(bool),

    // A specific input.
    Input(InputBinding),

    // All of SNES RAM.
    RAM(usize),

    // Up to 32 bytes.
    Bytes([u8; 32])
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
        byte_width: usize,
        conditional_op: ConditionalOp,
        jump_by: isize,
    },
    Copy {
        destination: DataValue,
        source: DataValue,
        byte_count: usize,
    },
    LoadCoordinates {
        actor: ActorRef,
        x_to: DataValue,
        y_to: DataValue,
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
        mode: ColorMathMode,
        r: bool,
        g: bool,
        b: bool,
        color_start: u8,
        color_count: u8,
        intensity_start: f64,
        intensity_end: f64,
        duration: f64,
    },
    SetPaletteData {
        sub_palette: usize,
        color_index: usize,
        data: [u8; 32],
    },
    LoadPalette {
        palette_index: usize,
    },
    LoadCharacter {
        character_type: ChararacterType,
        character_index: usize,
        is_in_party: bool,
    },
    ByteMath {
        rhs: DataValue,
        lhs: DataValue,
        byte_count: usize,
        operation: ByteMathOp,
    },
    BitMath {
        rhs: DataValue,
        lhs: DataValue,
        operation: BitMathOp,
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

        // Copy data around in memory.
        // Set actor result from 0x7F0200.
        0x19 => Op::Copy {
            destination: DataValue::ActorResult(ActorRef::This),
            source: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 1,
        },
        // Set actor result from 0x7F0000.
        0x1C => Op::Copy {
            destination: DataValue::ActorResult(ActorRef::This),
            source: DataValue::StoredLower(data.read_u8().unwrap() as usize),
            byte_count: 1,
        },
        // Set what character the first party member is to 0x7F0200.
        0x20 => Op::Copy {
            destination: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            source: DataValue::PartyCharacter(0),
            byte_count: 1,
        },
        // From RAM to temporary memory.
        0x48 => Op::Copy {
            // todo validate that this is read correctly.
            source: DataValue::RAM(
                data.read_u8().unwrap() as usize |
                data.read_u8().unwrap() as usize >> 8 |
                data.read_u8().unwrap() as usize >> 16
            ),
            destination: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 1,
        },
        0x49 => Op::Copy {
            // todo validate that this is read correctly.
            source: DataValue::RAM(
                data.read_u8().unwrap() as usize |
                data.read_u8().unwrap() as usize >> 8 |
                data.read_u8().unwrap() as usize >> 16
            ),
            destination: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 1,
        },
        // Write directly to RAM.
        0x4A => Op::Copy {
            // todo validate that this is read correctly.
            destination: DataValue::RAM(
                data.read_u8().unwrap() as usize |
                data.read_u8().unwrap() as usize >> 8 |
                data.read_u8().unwrap() as usize >> 16
            ),
            source: DataValue::Immediate(data.read_u8().unwrap() as u32),
            byte_count: 1,
        },
        0x4B => Op::Copy {
            // todo validate that this is read correctly.
            destination: DataValue::RAM(
                data.read_u8().unwrap() as usize |
                data.read_u8().unwrap() as usize >> 8 |
                data.read_u8().unwrap() as usize >> 16
            ),
            source: DataValue::Immediate(data.read_u16::<LittleEndian>().unwrap() as u32),
            byte_count: 2,
        },
        // Write to RAM.
        0x4C => Op::Copy {
            // todo validate that this is read correctly.
            destination: DataValue::RAM(
                data.read_u8().unwrap() as usize |
                data.read_u8().unwrap() as usize >> 8 |
                data.read_u8().unwrap() as usize >> 16
            ),
            source: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 1,
        },
        0x4D => Op::Copy {
            // todo validate that this is read correctly.
            destination: DataValue::RAM(
                data.read_u8().unwrap() as usize |
                data.read_u8().unwrap() as usize >> 8 |
                data.read_u8().unwrap() as usize >> 16
            ),
            source: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 1,
        },
        0x4E => {
            // todo validate that this is read correctly.
            let destination =
                data.read_u8().unwrap() as usize |
                data.read_u8().unwrap() as usize >> 8 |
                data.read_u8().unwrap() as usize >> 16;

            // todo this stores up to 32 bytes because we cannot copy a Vec.
            let data_len = data.read_u16::<LittleEndian>().unwrap() as usize - 2;
            if data_len > 32 {
                panic!("0x4E copy data is larger than 32 bytes.");
            }
            let mut bytes_data = vec![0u8; data_len];
            data.read_exact(&mut bytes_data).unwrap();

            Op::Copy {
                destination: DataValue::RAM(destination),
                source: DataValue::Bytes(bytes_data.first_chunk::<32>().unwrap().clone()),
                byte_count: data_len,
            }
        },
        0x4F => Op::Copy {
            source: DataValue::Immediate(data.read_u8().unwrap() as u32),
            destination: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 1,
        },
        0x50 => Op::Copy {
            source: DataValue::Immediate(data.read_u16::<LittleEndian>().unwrap() as u32),
            destination: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 2,
        },
        0x51 => Op::Copy {
            source: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            destination: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 1,
        },
        0x52 => Op::Copy {
            source: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            destination: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 2,
        },
        0x53 => Op::Copy {
            source: DataValue::Upper(data.read_u16::<LittleEndian>().unwrap() as usize),
            destination: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 1,
        },
        0x54 => Op::Copy {
            source: DataValue::Upper(data.read_u16::<LittleEndian>().unwrap() as usize),
            destination: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 2,
        },
        0x56 => Op::Copy {
            source: DataValue::Immediate(data.read_u8().unwrap() as u32),
            destination: DataValue::StoredLower(data.read_u8().unwrap() as usize * 2),
            byte_count: 1,
        },
        0x58 => Op::Copy {
            source: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            destination: DataValue::Upper(data.read_u16::<LittleEndian>().unwrap() as usize),
            byte_count: 1,
        },
        0x59 => Op::Copy {
            source: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            destination: DataValue::Upper(data.read_u16::<LittleEndian>().unwrap() as usize),
            byte_count: 2,
        },
        0x75 => Op::Copy {
            source: DataValue::Immediate(1),
            destination: DataValue::StoredLower(data.read_u8().unwrap() as usize * 2),
            byte_count: 1,
        },
        0x76 => Op::Copy {
            source: DataValue::Immediate(1),
            destination: DataValue::StoredLower(data.read_u8().unwrap() as usize * 2),
            byte_count: 2,
        },
        0x77 => Op::Copy {
            source: DataValue::Immediate(0),
            destination: DataValue::StoredLower(data.read_u8().unwrap() as usize * 2),
            byte_count: 1,
        },

        // Byte math.
        0x5B => Op::ByteMath {
            rhs: DataValue::Immediate(data.read_u8().unwrap() as u32),
            lhs: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 1,
            operation: ByteMathOp::Add,
        },
        0x5D => Op::ByteMath {
            rhs: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            lhs: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 1,
            operation: ByteMathOp::Add,
        },
        0x5E => Op::ByteMath {
            rhs: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            lhs: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 2,
            operation: ByteMathOp::Add,
        },
        0x5F => Op::ByteMath {
            rhs: DataValue::Immediate(data.read_u8().unwrap() as u32),
            lhs: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 1,
            operation: ByteMathOp::Subtract,
        },
        0x60 => Op::ByteMath {
            rhs: DataValue::Immediate(data.read_u8().unwrap() as u32),
            lhs: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 2,
            operation: ByteMathOp::Subtract,
        },
        0x61 => Op::ByteMath {
            rhs: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            lhs: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 1,
            operation: ByteMathOp::Subtract,
        },
        0x71 => Op::ByteMath {
            rhs: DataValue::Immediate(1),
            lhs: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 1,
            operation: ByteMathOp::Add,
        },
        0x72 => Op::ByteMath {
            rhs: DataValue::Immediate(1),
            lhs: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 2,
            operation: ByteMathOp::Add,
        },
        0x73 => Op::ByteMath {
            rhs: DataValue::Immediate(1),
            lhs: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            byte_count: 1,
            operation: ByteMathOp::Subtract,
        },

        // Bit math.
        0x63 => Op::BitMath {
            rhs: DataValue::Immediate(1 >> data.read_u8().unwrap() as u32),
            lhs: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            operation: BitMathOp::Or,
        },
        0x64 => Op::BitMath {
            rhs: DataValue::Immediate(1 >> data.read_u8().unwrap() as u32),
            lhs: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            operation: BitMathOp::And,
        },
        0x65 => {
            let bit = data.read_u8().unwrap();
            let mut address = data.read_u8().unwrap() as usize;
            if bit & 0x80 > 0 {
                address += 0x100;
            }
            Op::BitMath {
                rhs: DataValue::Immediate(1 >> (bit & 0x7F) as u32),
                lhs: DataValue::Temp(address),
                operation: BitMathOp::Or,
            }
        },
        0x66 => {
            let bit = data.read_u8().unwrap();
            let mut address = data.read_u8().unwrap() as usize;
            if bit & 0x80 > 0 {
                address += 0x100;
            }
            Op::BitMath {
                rhs: DataValue::Immediate(1 >> (bit & 0x7F) as u32),
                lhs: DataValue::Temp(address),
                operation: BitMathOp::And,
            }
        },
        0x67 => Op::BitMath {
            rhs: DataValue::Immediate(data.read_u8().unwrap() as u32),
            lhs: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            operation: BitMathOp::And,
        },
        0x69 => Op::BitMath {
            rhs: DataValue::Immediate(data.read_u8().unwrap() as u32),
            lhs: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            operation: BitMathOp::Or,
        },
        0x6B => Op::BitMath {
            rhs: DataValue::Immediate(data.read_u8().unwrap() as u32),
            lhs: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            operation: BitMathOp::Xor,
        },
        0x6F => Op::BitMath {
            rhs: DataValue::Immediate(data.read_u8().unwrap() as u32),
            lhs: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            operation: BitMathOp::ShiftRight,
        },

        // Write to storyline counter.
        0x55 => Op::Copy {
            source: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            destination: DataValue::StoredLower(0x00),
            byte_count: 1,
        },
        0x5A => Op::Copy {
            source: DataValue::Immediate(data.read_u8().unwrap() as u32),
            destination: DataValue::StoredLower(0x00),
            byte_count: 1,
        },

        // Character load.
        0x57 => Op::LoadCharacter {
            character_type: ChararacterType::PC,
            character_index: 0,
            is_in_party: true,
        },
        0x5C => Op::LoadCharacter {
            character_type: ChararacterType::PC,
            character_index: 1,
            is_in_party: true,
        },
        0x62 => Op::LoadCharacter {
            character_type: ChararacterType::PC,
            character_index: 2,
            is_in_party: true,
        },
        0x68 => Op::LoadCharacter {
            character_type: ChararacterType::PC,
            character_index: 3,
            is_in_party: true,
        },
        0x6A => Op::LoadCharacter {
            character_type: ChararacterType::PC,
            character_index: 4,
            is_in_party: true,
        },
        0x6C => Op::LoadCharacter {
            character_type: ChararacterType::PC,
            character_index: 5,
            is_in_party: true,
        },
        0x6D => Op::LoadCharacter {
            character_type: ChararacterType::PC,
            character_index: 6,
            is_in_party: true,
        },

        // Actor coordinates.
        // From actor.
        0x21 => Op::LoadCoordinates {
            actor: ActorRef::Actor(data.read_u8().unwrap() as usize / 2),
            x_to: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            y_to: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
        },
        // From party member actor.
        0x22 => Op::LoadCoordinates {
            actor: ActorRef::PartyMember(data.read_u8().unwrap() as usize),
            x_to: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
            y_to: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
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
            source: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
        },
        0x24 => Op::LoadDirection {
            actor: ActorRef::PartyMember(data.read_u8().unwrap() as usize),
            source: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
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
            lhs: DataValue::ActorFlag(ActorRef::Actor(data.read_u8().unwrap() as usize * 2), ActorFlags::VISIBLE),
            rhs: DataValue::Immediate(0),
            byte_width: 1,
            conditional_op: ConditionalOp::Eq,
            jump_by: data.read_u8().unwrap() as isize,
        },
        // If actor is in battle.
        0x28 => Op::JumpConditional {
            lhs: DataValue::ActorFlag(ActorRef::Actor(data.read_u8().unwrap() as usize * 2), ActorFlags::IN_BATTLE),
            rhs: DataValue::Immediate(1),
            byte_width: 1,
            conditional_op: ConditionalOp::Eq,
            jump_by: data.read_u8().unwrap() as isize,
        },

        // Palette.
        0x2E => {
            let mode = data.read_u8().unwrap();
            if mode & 0x40 > 0 {
                let b = ((mode & 0x4) >> 2) > 0;
                let g = ((mode & 0x2) >> 1) > 0;
                let r = ((mode & 0x1) >> 0) > 0;

                let color_start = data.read_u8().unwrap();
                let color_count = data.read_u8().unwrap();

                let intensity_bits = data.read_u8().unwrap();
                let intensity_end: f64 = (intensity_bits & 0xF) as f64 * (1.0 / 15.0);
                let intensity_start: f64 = ((intensity_bits & 0xF0) >> 4) as f64 * (1.0 / 15.0);

                // todo what unit is this in? Assuming 60 Hz frames for now.
                let duration = data.read_u8().unwrap() as f64 * (1.0 / 60.0);

                Op::ColorMath {
                    mode: if mode & 0x50 > 0 { ColorMathMode::Additive } else { ColorMathMode::Subtractive },
                    r, g, b,
                    color_start, color_count,
                    intensity_start, intensity_end,
                    duration,
                }

            } else if mode & 0x80 > 0 {
                let bits = data.read_u8().unwrap() as usize;
                let color_index = bits & 0xF;
                let sub_palette = (bits & 0xF0) >> 4;

                // todo this stores up to 32 bytes 16 colors because we cannot copy a Vec.
                let data_len = data.read_u16::<LittleEndian>().unwrap() as usize - 2;
                if data_len > 32 {
                    panic!("SetPaletteData color data is too large.");
                }
                let mut color_data = vec![0u8; data_len];
                data.read_exact(&mut color_data).unwrap();

                Op::SetPaletteData {
                    sub_palette,
                    color_index,
                    data: color_data.first_chunk::<32>().unwrap().clone(),
                }
            } else {
                println!("Mode for op 0x2E is unknown.");
                Op::NOP
            }
        },
        0x33 => Op::LoadPalette {
            palette_index: data.read_u8().unwrap() as usize,
        },

        // Input tests.
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
        // Input tests, changed since last test.
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
        0x32 => Op::Unimplemented {
            code: 0x32,
            data: [0, 0, 0, 0],
        },

        // Unknown purpose.
        0x2F => Op::Unimplemented {
            code: 0x2F,
            data: [data.read_u8().unwrap(), 0, 0, 0],
        },
        0x47 => Op::Unimplemented {
            code: 0x47,
            data: [data.read_u8().unwrap(), 0, 0, 0],
        },

        _ => {
            println!("Decoding unimplemented opcode 0x{:02X} as NOP", op_byte);
            Op::NOP
        },
    }
}
