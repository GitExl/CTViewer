use std::io::{Cursor, Read};
use byteorder::{LittleEndian, ReadBytesExt};
use crate::actor::ActorFlags;
use crate::scene::ops::Op;
use crate::scene::ops_actor_props::op_decode_actor_props;
use crate::scene::ops_call::op_decode_call;
use crate::scene::ops_char_load::op_decode_char_load;
use crate::scene::ops_copy::op_decode_copy;
use crate::scene::ops_direction::op_decode_direction;
use crate::scene::ops_jump::op_decode_jump;
use crate::scene::ops_math::op_decode_math;
use crate::scene::ops_movement::ops_decode_movement;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum SubPalette {
    This,
    Index(usize),
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ColorMathMode {
    Additive,
    Subtractive,
}

/// Type of reference to an actor.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ActorRef {
    This,
    ScriptActor(usize),
    ScriptActorStoredUpper(usize),
    PartyMember(usize),
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
pub enum DataRef {
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

    // Next value from random value table.
    Random,
}

/// Opcodes.
pub fn op_decode(data: &mut Cursor<Vec<u8>>) -> Op {
    let op_byte = data.read_u8().unwrap();

    match op_byte {

        // Yield to the function with the next higher priority number.
        // If there is none, simply yield.
        0x00 => Op::Yield,

        // Function calls.
        0x02 | 0x03 | 0x04 | 0x05 | 0x06 | 0x07 => op_decode_call(op_byte, data),

        // Actor properties.
        0x08 | 0x09 | 0x0A | 0x0B | 0x0C | 0x7C | 0x7D | 0x90 | 0x91 | 0x7E | 0x8E | 0x84 | 0x0D |
        0x0E | 0x89 | 0x8A => op_decode_actor_props(op_byte, data),

        // Actor movement.
        0x8F | 0x92 | 0x94 | 0x95 | 0x96 | 0x97 | 0x98 | 0x99 | 0x9A | 0x9C | 0x9D |
        0x9E | 0x9F | 0xA0 | 0xA1 | 0x7A | 0x7B => ops_decode_movement(op_byte, data),

        // Data copy.
        0x19 | 0x1C | 0x20 | 0x48 | 0x49 | 0x4A | 0x4B | 0x4C | 0x4D | 0x4E | 0x4F | 0x50 | 0x51 |
        0x52 | 0x53 | 0x54 | 0x55 | 0x56 | 0x58 | 0x59 | 0x5A | 0x75 | 0x76 | 0x77 | 0x7F => op_decode_copy(op_byte, data),

        // Byte math.
        0x5B | 0x5D | 0x5E | 0x5F | 0x60 | 0x61 | 0x71 | 0x72 | 0x73 | 0x63 | 0x64 | 0x65 | 0x66 |
        0x67 | 0x69 | 0x6B | 0x6F => op_decode_math(op_byte, data),

        // Load character.
        0x57 | 0x5C | 0x62 | 0x68 | 0x6A | 0x6C | 0x6D | 0x80 | 0x81 | 0x82 | 0x83 => op_decode_char_load(op_byte, data),

        // Actor direction.
        0x0F | 0x17 | 0x1B | 0x1D | 0x1E | 0x1F | 0x25 | 0x26 | 0x23 | 0x24 | 0xA6 | 0xA7 | 0xA8 |
        0xA9 => op_decode_direction(op_byte, data),

        // Code jumps.
        0x10 | 0x11 | 0x12 | 0x13 | 0x14 | 0x15 | 0x16 | 0x18 | 0x1A | 0x27 | 0x28 | 0x2D | 0x30 |
        0x31 | 0x34 | 0x35 | 0x36 | 0x37 | 0x38 | 0x39 | 0x3B | 0x3C | 0x3F | 0x40 | 0x41 | 0x42 |
        0x43 | 0x44 => op_decode_jump(op_byte, data),

        // Actor coordinates.
        // From actor.
        0x21 => Op::ActorCoordinatesGet {
            actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            x: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            y: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
        },
        // From party member actor.
        0x22 => Op::ActorCoordinatesGet {
            actor: ActorRef::PartyMember(data.read_u8().unwrap() as usize),
            x: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            y: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
        },
        0x8B => Op::ActorCoordinatesSet {
            actor: ActorRef::This,
            x: DataRef::Immediate(data.read_u8().unwrap() as u32),
            y: DataRef::Immediate(data.read_u8().unwrap() as u32),
            precise: false,
        },
        0x8C => Op::ActorCoordinatesSet {
            actor: ActorRef::This,
            x: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            y: DataRef::StoredUpper(data.read_u8().unwrap() as usize * 2),
            precise: false,
        },
        0x8D => Op::ActorCoordinatesSet {
            actor: ActorRef::This,
            x: DataRef::Immediate(data.read_u16::<LittleEndian>().unwrap() as u32),
            y: DataRef::Immediate(data.read_u16::<LittleEndian>().unwrap() as u32),
            precise: true,
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
            palette: data.read_u8().unwrap() as usize,
        },

        // 0x88 sub ops.
        0x88 => {
            let cmd = data.read_u8().unwrap();
            if cmd == 0 {
                Op::PaletteRestore
            } else if cmd == 0x20 {
                Op::Unknown {
                    code: 0x88,
                    data: [cmd, data.read_u8().unwrap(), data.read_u8().unwrap(), 0],
                }
            } else if cmd == 0x30 {
                Op::Unknown {
                    code: 0x88,
                    data: [cmd, data.read_u8().unwrap(), data.read_u8().unwrap(), 0],
                }
            } else if cmd > 0x40 && cmd < 0x60 {
                Op::Unknown {
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

        // Ascii text related (???)
        0x29 => Op::Unknown {
            code: 0x29,
            data: [data.read_u8().unwrap(), 0, 0, 0],
        },
        0x2A => Op::Unknown {
            code: 0x2A,
            data: [0, 0, 0, 0],
        },
        0x2B => Op::Unknown {
            code: 0x2B,
            data: [0, 0, 0, 0],
        },
        0x2C => Op::Unknown {
            code: 0x2C,
            data: [data.read_u8().unwrap(), data.read_u8().unwrap(), 0, 0],
        },
        0x32 => Op::Unknown {
            code: 0x32,
            data: [0, 0, 0, 0],
        },

        // Unknown purpose.
        0x2F => Op::Unknown {
            code: 0x2F,
            data: [data.read_u8().unwrap(), 0, 0, 0],
        },
        0x47 => Op::Unknown {
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
