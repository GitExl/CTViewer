use std::io::{Cursor, Read};
use byteorder::{LittleEndian, ReadBytesExt};
use crate::actor::ActorFlags;
use crate::scene::ops_copy::op_decode_copy;
use crate::scene::ops_jump::op_decode_jump;
use crate::scene::ops_math::op_decode_math;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum SubPalette {
    This,
    Index(usize),
}

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
pub enum CharacterType {
    PC,
    NPC,
    Enemy,
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
    ScriptActor(usize),
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
    Bytes([u8; 32]),

    // Value from random table.
    Random,
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
    SetDirection {
        actor: ActorRef,
        direction: usize,
    },
    Jump {
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
    PaletteSetImmediate {
        sub_palette: SubPalette,
        color_index: usize,
        data: [u8; 32],
    },
    PaletteSet {
        palette_index: usize,
    },
    PaletteRestore,
    LoadCharacter {
        character_type: CharacterType,
        character_index: usize,
        must_be_in_party: bool,
        is_static: bool,
        battle_index: usize,
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
    MovementJump {
        actor: ActorRef,
        x: i32,
        y: i32,
        height: u32,
    },
    SetScriptSpeed {
        speed: u8,
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
                actor: ActorRef::ScriptActor(actor_index),
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
                actor: ActorRef::ScriptActor(actor_index),
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
                actor: ActorRef::ScriptActor(actor_index),
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
            actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            flags_set: ActorFlags::DISABLED,
            flags_remove: ActorFlags::RENDERED,
        },

        // Disable/enable script execution.
        0x0B => Op::UpdateActorFlags {
            actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            flags_set: ActorFlags::DISABLED,
            flags_remove: ActorFlags::empty(),
        },
        0x0C => Op::UpdateActorFlags {
            actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            flags_set: ActorFlags::empty(),
            flags_remove: ActorFlags::DISABLED,
        },

        0x7C => Op::UpdateActorFlags {
            actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            flags_set: ActorFlags::RENDERED,
            flags_remove: ActorFlags::HIDDEN,
        },
        0x7D => Op::UpdateActorFlags {
            actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            flags_set: ActorFlags::empty(),
            flags_remove: ActorFlags::RENDERED | ActorFlags::HIDDEN,
        },
        0x7E => Op::UpdateActorFlags {
            actor: ActorRef::This,
            flags_set: ActorFlags::RENDERED | ActorFlags::HIDDEN,
            flags_remove: ActorFlags::empty(),
        },

        // Set actor solidity.
        0x84 => {
            let bits = data.read_u8().unwrap();
            let mut flags_set = ActorFlags::empty();
            if bits & 0x01 > 0 {
                flags_set |= ActorFlags::SOLID;
            }
            if bits & 0x02 > 0 {
                flags_set |= ActorFlags::PUSHABLE;
            }

            Op::UpdateActorFlags {
                actor: ActorRef::This,
                flags_set,
                flags_remove: flags_set.complement(),
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

        // Copy.
        0x19 | 0x1C | 0x20 | 0x48 | 0x49 | 0x4A | 0x4B | 0x4C | 0x4D | 0x4E | 0x4F | 0x50 | 0x51 |
        0x52 | 0x53 | 0x54 | 0x55 | 0x56 | 0x58 | 0x59 | 0x5A | 0x75 | 0x76 | 0x77 | 0x7F => op_decode_copy(op_byte, data),

        // Byte math.
        0x5B | 0x5D | 0x5E | 0x5F | 0x60 | 0x61 | 0x71 | 0x72 | 0x73 | 0x63 | 0x64 | 0x65 | 0x66 |
        0x67 | 0x69 | 0x6B | 0x6F => op_decode_math(op_byte, data),

        // Character load.
        0x57 => Op::LoadCharacter {
            character_type: CharacterType::PC,
            character_index: 0,
            must_be_in_party: true,
            is_static: false,
            battle_index: 0,
        },
        0x5C => Op::LoadCharacter {
            character_type: CharacterType::PC,
            character_index: 1,
            must_be_in_party: true,
            is_static: false,
            battle_index: 0,
        },
        0x62 => Op::LoadCharacter {
            character_type: CharacterType::PC,
            character_index: 2,
            must_be_in_party: true,
            is_static: false,
            battle_index: 0,
        },
        0x68 => Op::LoadCharacter {
            character_type: CharacterType::PC,
            character_index: 3,
            must_be_in_party: true,
            is_static: false,
            battle_index: 0,
        },
        0x6A => Op::LoadCharacter {
            character_type: CharacterType::PC,
            character_index: 4,
            must_be_in_party: true,
            is_static: false,
            battle_index: 0,
        },
        0x6C => Op::LoadCharacter {
            character_type: CharacterType::PC,
            character_index: 5,
            must_be_in_party: true,
            is_static: false,
            battle_index: 0,
        },
        0x6D => Op::LoadCharacter {
            character_type: CharacterType::PC,
            character_index: 6,
            must_be_in_party: true,
            is_static: false,
            battle_index: 0,
        },
        0x80 => Op::LoadCharacter {
            character_type: CharacterType::PC,
            character_index: data.read_u8().unwrap() as usize,
            must_be_in_party: true,
            is_static: false,
            battle_index: 0,
        },
        0x81 => Op::LoadCharacter {
            character_type: CharacterType::NPC,
            character_index: data.read_u8().unwrap() as usize,
            must_be_in_party: false,
            is_static: false,
            battle_index: 0,
        },
        0x82 => Op::LoadCharacter {
            character_type: CharacterType::NPC,
            character_index: data.read_u8().unwrap() as usize,
            must_be_in_party: false,
            is_static: false,
            battle_index: 0,
        },
        0x83 => {
            let index = data.read_u8().unwrap() as usize;
            let bits = data.read_u8().unwrap();
            Op::LoadCharacter {
                character_type: CharacterType::Enemy,
                character_index: index,
                must_be_in_party: false,
                is_static: bits & 0x80 > 0,
                battle_index: bits as usize & 0x7F,
            }
        },

        // Actor coordinates.
        // From actor.
        0x21 => Op::LoadCoordinates {
            actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
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
        0x0F => Op::SetDirection {
            actor: ActorRef::This,
            direction: 0,
        },
        0x17 => Op::SetDirection {
            actor: ActorRef::This,
            direction: 1,
        },
        0x1B => Op::SetDirection {
            actor: ActorRef::This,
            direction: 2,
        },
        0x1D => Op::SetDirection {
            actor: ActorRef::This,
            direction: 3,
        },
        0x1E => Op::SetDirection {
            actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            direction: 0,
        },
        0x1F => Op::SetDirection {
            actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            direction: 1,
        },
        0x25 => Op::SetDirection {
            actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            direction: 2,
        },
        0x26 => Op::SetDirection {
            actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            direction: 3,
        },
        0x23 => Op::LoadDirection {
            actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            source: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
        },
        0x24 => Op::LoadDirection {
            actor: ActorRef::PartyMember(data.read_u8().unwrap() as usize),
            source: DataValue::StoredUpper(data.read_u8().unwrap() as usize * 2),
        },

        // Jumps.
        0x10 | 0x11 | 0x12 | 0x13 | 0x14 | 0x15 | 0x16 | 0x18 | 0x1A | 0x27 | 0x28 | 0x2D | 0x30 |
        0x31 | 0x34 | 0x35 | 0x36 | 0x37 | 0x38 | 0x39 | 0x3B | 0x3C | 0x3F | 0x40 | 0x41 | 0x42 |
        0x43 | 0x44 => op_decode_jump(op_byte, data),

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

                Op::PaletteSetImmediate {
                    sub_palette: SubPalette::Index(sub_palette),
                    color_index,
                    data: read_script_blob(data),
                }
            } else {
                println!("Mode for op 0x2E is unknown.");
                Op::NOP
            }
        },
        0x33 => Op::PaletteSet {
            palette_index: data.read_u8().unwrap() as usize,
        },

        // 0x88 sub ops.
        0x88 => {
            let cmd = data.read_u8().unwrap();
            if cmd == 0 {
                Op::PaletteRestore
            } else if cmd == 0x20 {
                Op::Unimplemented {
                    code: 0x88,
                    data: [cmd, data.read_u8().unwrap(), data.read_u8().unwrap(), 0],
                }
            } else if cmd == 0x30 {
                Op::Unimplemented {
                    code: 0x88,
                    data: [cmd, data.read_u8().unwrap(), data.read_u8().unwrap(), 0],
                }
            } else if cmd > 0x40 && cmd < 0x60 {
                Op::Unimplemented {
                    code: 0x88,
                    data: [cmd, data.read_u8().unwrap(), data.read_u8().unwrap(), data.read_u8().unwrap()],
                }
            } else if cmd > 0x80 && cmd < 0x90 {
                Op::PaletteSetImmediate {
                    color_index: cmd as usize & 0xF,
                    sub_palette: SubPalette::This,
                    data: read_script_blob(data),
                }
            } else {
                panic!("Unknown 0x88 command {}.", cmd);
            }
        },

        // Script speed.
        0x87 => Op::SetScriptSpeed {
            speed: data.read_u8().unwrap(),
        },

        // Physical movement related.
        0x7A => Op::MovementJump {
            actor: ActorRef::This,
            x: data.read_i8().unwrap() as i32,
            y: data.read_i8().unwrap() as i32,
            height: data.read_u8().unwrap() as u32,
        },
        0x7B => Op::Unimplemented {
            code: 0x7B,
            data: [data.read_u8().unwrap(), data.read_u8().unwrap(), data.read_u8().unwrap(), data.read_u8().unwrap()],
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

pub fn read_script_blob(data: &mut Cursor<Vec<u8>>) -> [u8; 32] {
    let data_len = data.read_u16::<LittleEndian>().unwrap() as usize - 2;
    if data_len > 32 {
        panic!("Blob data is larger than 32 bytes.");
    }

    let mut blob = vec![0u8; data_len];
    data.read_exact(&mut blob).unwrap();

    blob.first_chunk::<32>().unwrap().clone()
}
