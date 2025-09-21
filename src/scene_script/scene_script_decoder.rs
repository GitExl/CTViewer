use std::io::{Cursor, Read};
use bitflags::bitflags;
use byteorder::{LittleEndian, ReadBytesExt};
use crate::scene_script::decoder::ops_inventory::op_decode_inventory;
use crate::scene_script::ops::Op;
use crate::scene_script::decoder::ops_actor_props::op_decode_actor_props;
use crate::scene_script::decoder::ops_animation::op_decode_animation;
use crate::scene_script::decoder::ops_audio::op_decode_audio;
use crate::scene_script::decoder::ops_call::op_decode_call;
use crate::scene_script::decoder::ops_char_load::op_decode_char_load;
use crate::scene_script::decoder::ops_copy::op_decode_copy;
use crate::scene_script::decoder::ops_dialogue::op_decode_dialogue;
use crate::scene_script::decoder::ops_facing::op_decode_facing;
use crate::scene_script::decoder::ops_jump::op_decode_jump;
use crate::scene_script::decoder::ops_location::op_decode_location;
use crate::scene_script::decoder::ops_math::op_decode_math;
use crate::scene_script::decoder::ops_movement::ops_decode_movement;
use crate::scene_script::decoder::ops_palette::{op_decode_palette, ColorMathMode};
use crate::scene_script::decoder::ops_party::op_decode_party;
use crate::scene_script::scene_script::SceneScriptMode;
use crate::scene_script::scene_script_memory::DataDest;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum SpecialEffect {
    ScreenOpenLeftRight,
    ScreenOpenRightLeft,
    ScreenCloseLeftRight,
    ScreenCloseRightLeft,
    Reset,
    PortalHuge,
    ResetMemBits,
    RealityDistortion,
    NewGamePlus,
    Unknown (u8, [u8; 3])
}

bitflags! {
    #[derive(Clone, Default, Copy, Debug, PartialEq)]
    pub struct ScrollLayerFlags: u32 {
        const SCROLL_L1 = 0x01;
        const SCROLL_L2 = 0x02;
        const SCROLL_L3 = 0x04;
    }
}

bitflags! {
    #[derive(Clone, Default, Copy, Debug, PartialEq)]
    pub struct CopyTilesFlags: u32 {
        const COPY_L1 = 0x01;
        const COPY_L2 = 0x02;
        const COPY_L3 = 0x04;
        const COPY_PROPS = 0x08;
        const UNKNOWN1 = 0x10;
        const UNKNOWN2 = 0x20;
        const COPY_Z_PLANE = 0x40;
        const COPY_MOVEMENT = 0x80;
    }
}

bitflags! {
    #[derive(Clone, Default, Copy, Debug, PartialEq)]
    pub struct BattleFlags: u32 {
        const AUTO_REGROUP = 0x8000;
        const KEEP_MUSIC = 0x4000;
        const NO_GAME_OVER = 0x2000;
        const CANNOT_RUN = 0x0080;
        const ATTRACT_MODE = 0x0020;
        const STATIC_ENEMIES = 0x0010;
        const SMALL_PC_SOL = 0x0004;
        const BOTTOM_UI = 0x0002;
        const NO_WIN_POSE = 0x0001;
    }
}

/// Type of reference to an actor.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ActorRef {
    This,
    ScriptActor(usize),
    ScriptActorStoredUpper(usize),
    PartyMember(usize),
}

impl ActorRef {
    pub fn deref(self, current_actor_index: usize) -> usize {
        match self {
            ActorRef::This => current_actor_index,
            ActorRef::ScriptActor(index) => index,
            ActorRef::PartyMember(_index) => 0,  // todo
            ActorRef::ScriptActorStoredUpper(_address) => 0,  // todo
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

/// Opcodes.
pub fn op_decode(data: &mut Cursor<Vec<u8>>, mode: SceneScriptMode) -> Option<Op> {
    let op_byte = match data.read_u8() {
        Ok(op_byte) => op_byte,
        Err(_) => {
            println!("Script execution past end of data at 0x{:04X}.", data.position());
            return None;
        }
    };

    let op = match op_byte {

        // Function calls.
        0x02 | 0x03 | 0x04 | 0x05 | 0x06 | 0x07 => op_decode_call(op_byte, data),

        // Actor properties.
        0x08 | 0x09 | 0x0A | 0x0B | 0x0C | 0x19 | 0x1C | 0x7C | 0x7D | 0x90 | 0x91 | 0x7E | 0x8E |
        0x84 | 0x0D | 0x0E | 0x89 | 0x8A | 0x21 | 0x22 | 0x8B | 0x8C | 0x8D | 0xF8 | 0xF9 | 0xFA => op_decode_actor_props(op_byte, data),

        // Actor movement.
        0x8F | 0x92 | 0x94 | 0x95 | 0x96 | 0x97 | 0x98 | 0x99 | 0x9A | 0x9C | 0x9D | 0x9E | 0x9F |
        0xA0 | 0xA1 | 0x7A | 0x7B | 0xB5 | 0xD9 | 0xB6 => ops_decode_movement(op_byte, data),

        // Data copy.
        0x20 | 0x48 | 0x49 | 0x4A | 0x4B | 0x4C | 0x4D | 0x4E | 0x4F | 0x50 | 0x51 | 0x52 | 0x53 |
        0x54 | 0x55 | 0x56 | 0x58 | 0x59 | 0x5A | 0x75 | 0x76 | 0x77 | 0x3A | 0x3D | 0x3E | 0x70 |
        0x74 | 0x78 => op_decode_copy(op_byte, data, mode),

        // Byte math.
        0x5B | 0x5D | 0x5E | 0x5F | 0x60 | 0x61 | 0x71 | 0x72 | 0x73 | 0x63 | 0x64 | 0x65 | 0x66 |
        0x67 | 0x69 | 0x6B | 0x6F | 0x2A | 0x2B | 0x32 | 0x45 | 0x46 => op_decode_math(op_byte, data),

        // Load character.
        0x57 | 0x5C | 0x62 | 0x68 | 0x6A | 0x6C | 0x6D | 0x80 | 0x81 | 0x82 | 0x83 => op_decode_char_load(op_byte, data, mode),

        // Actor facing.
        0x0F | 0x17 | 0x1B | 0x1D | 0x1E | 0x1F | 0x25 | 0x26 | 0x23 | 0x24 | 0xA6 | 0xA7 | 0xA8 |
        0xA9 => op_decode_facing(op_byte, data),

        // Code jumps.
        0x10 | 0x11 | 0x12 | 0x13 | 0x14 | 0x15 | 0x16 | 0x18 | 0x1A | 0x27 | 0x28 | 0x2D | 0x30 |
        0x31 | 0x34 | 0x35 | 0x36 | 0x37 | 0x38 | 0x39 | 0x3B | 0x3C | 0x3F | 0x40 | 0x41 | 0x42 |
        0x43 | 0x44 | 0xC9 | 0xCC | 0xCF | 0xD2 | 0x6E => op_decode_jump(op_byte, data),

        // Dialogue.
        0xB8 | 0xBB | 0xC0 | 0xC1 | 0xC2 | 0xC3 | 0xC4 | 0xC8 => op_decode_dialogue(op_byte, data, mode),

        // Animation.
        0xAA | 0xAB | 0xAC | 0xAE | 0xB3 | 0xB4 | 0xB7 | 0x47 => op_decode_animation(op_byte, data),

        // Party management.
        0xD0 | 0xD1 | 0xD3 | 0xD4 | 0xD6 | 0xD5 | 0xDA | 0xE3 => op_decode_party(op_byte, data),

        // Palettes.
        0x2E | 0x33 | 0x88 => op_decode_palette(op_byte, data, mode),

        // Move to another location.
        0xDC | 0xDD | 0xDE | 0xDF | 0xE0 | 0xE1 | 0xE2 => op_decode_location(op_byte, data, mode),

        // Inventory.
        0xC7 | 0xCA | 0xCB | 0xCD | 0xCE | 0xD7 => op_decode_inventory(op_byte, data, mode),

        // Sound and music.
        0xE8 | 0xEA | 0xEB | 0xEC | 0xED | 0xEE => op_decode_audio(op_byte, data),

        // Screen effects.
        0xF0 => Op::ScreenDarken {
            duration: data.read_u8().unwrap() as f64 / (1.0 / 60.0),
        },
        0xF1 => {
            let bits = data.read_u8().unwrap();
            if bits == 0 {
                Op::ScreenColorMath {
                    r: 0,
                    g: 0,
                    b: 0,
                    intensity: 0.0,
                    duration: 0.0,
                    mode: ColorMathMode::Additive,
                };
            }

            let b = if bits & 0x80 > 0 { 255 } else { 0 };
            let g = if bits & 0x40 > 0 { 255 } else { 0 };
            let r = if bits & 0x20 > 0 { 255 } else { 0 };
            let intensity = (bits & 0x1F) as f64 * (1.0 / 32.0);

            let params = data.read_u8().unwrap();
            let mode = if params & 0x80 > 0 { ColorMathMode::Additive } else { ColorMathMode::Subtractive };
            let duration = (params & 0x7F) as f64 * (1.0 / 60.0);

            Op::ScreenColorMath {
                r, g, b,
                intensity,
                mode,
                duration,
            }
        },
        0xF2 => Op::ScreenFadeOut,
        0xF3 => Op::ScreenWaitForColorMath,
        0xF4 => Op::ScreenShake {
            enabled: data.read_u8().unwrap() == 1,
        },
        0xFE => Op::ScreenColorMathGeometry {
            unknown: data.read_u8().unwrap(),

            x1_src: data.read_u8().unwrap(),
            x1_dest: data.read_u8().unwrap(),
            y1_src: data.read_u8().unwrap(),
            y1_dest: data.read_u8().unwrap(),

            x2_src: data.read_u8().unwrap(),
            x2_dest: data.read_u8().unwrap(),
            y2_src: data.read_u8().unwrap(),
            y2_dest: data.read_u8().unwrap(),

            x3_src: data.read_u8().unwrap(),
            x3_dest: data.read_u8().unwrap(),
            y3_src: data.read_u8().unwrap(),
            y3_dest: data.read_u8().unwrap(),

            x4_src: data.read_u8().unwrap(),
            x4_dest: data.read_u8().unwrap(),
            y4_src: data.read_u8().unwrap(),
            y4_dest: data.read_u8().unwrap(),
        },

        0x7F => Op::Random {
            dest: DataDest::for_local_memory(data.read_u8().unwrap() as usize * 2),
        },

        // Special effects or scenes.
        0xFF => {
            let mode = data.read_u8().unwrap();
            if mode <= 0x8F {
                Op::SpecialScene {
                    scene: mode as usize & 0x0F,
                    flags: (mode & 0xF0) >> 4,
                }
            } else if mode == 0x90 {
                Op::SpecialOpenPortal {
                    value1: data.read_u8().unwrap(),
                    value2: data.read_u8().unwrap(),
                    value3: data.read_u8().unwrap(),
                }
            } else if mode == 0x92 {
                Op::SpecialEffect(SpecialEffect::ScreenOpenLeftRight)
            } else if mode == 0x93 {
                Op::SpecialEffect(SpecialEffect::ScreenOpenRightLeft)
            } else if mode == 0x94 {
                Op::SpecialEffect(SpecialEffect::ScreenCloseLeftRight)
            } else if mode == 0x95 {
                Op::SpecialEffect(SpecialEffect::ScreenCloseRightLeft)
            } else if mode == 0x96 {
                Op::SpecialEffect(SpecialEffect::Reset)
            } else if mode == 0x97 {
                Op::SpecialEffect(
                    SpecialEffect::Unknown(
                        mode,
                        [data.read_u8().unwrap(), data.read_u8().unwrap(), data.read_u8().unwrap()],
                    ),
                )
            } else if mode == 0x9B {
                Op::SpecialEffect(SpecialEffect::PortalHuge)
            } else if mode == 0x9D {
                Op::SpecialEffect(SpecialEffect::ResetMemBits)
            } else if mode == 0x9E {
                Op::SpecialEffect(SpecialEffect::RealityDistortion)
            } else if mode == 0x9F {
                Op::SpecialEffect(SpecialEffect::NewGamePlus)
            } else {
                Op::SpecialEffect(
                    SpecialEffect::Unknown(
                        mode,
                        [0, 0, 0],
                    ),
                )
            }
        },

        // Copy tiles from somewhere else in the map.
        0xE4 => Op::CopyTiles {
            left: data.read_u8().unwrap() as u32 * 2,
            top: data.read_u8().unwrap() as u32 * 2,
            right: data.read_u8().unwrap() as u32 * 2 + 2,
            bottom: data.read_u8().unwrap() as u32 * 2 + 2,
            dest_x: data.read_u8().unwrap() as u32 * 2,
            dest_y: data.read_u8().unwrap() as u32 * 2,
            flags: CopyTilesFlags::from_bits_truncate(data.read_u8().unwrap() as u32),
        },
        // What is different in this version?
        0xE5 => Op::CopyTiles {
            left: data.read_u8().unwrap() as u32 * 2,
            top: data.read_u8().unwrap() as u32 * 2,
            right: data.read_u8().unwrap() as u32 * 2 + 2,
            bottom: data.read_u8().unwrap() as u32 * 2 + 2,
            dest_x: data.read_u8().unwrap() as u32 * 2,
            dest_y: data.read_u8().unwrap() as u32 * 2,
            flags: CopyTilesFlags::from_bits_truncate(data.read_u8().unwrap() as u32),
        },

        // Scroll map layers.
        0x2F => Op::ScrollLayers {
            x: data.read_i8().unwrap() as i32,
            y: data.read_i8().unwrap() as i32,
            flags: ScrollLayerFlags::SCROLL_L1 | ScrollLayerFlags::SCROLL_L2 | ScrollLayerFlags::SCROLL_L3,
            duration: 0,
        },
        0xE6 => Op::ScrollLayers {
            x: data.read_i8().unwrap() as i32,
            y: data.read_i8().unwrap() as i32,
            flags: ScrollLayerFlags::from_bits_truncate(data.read_u8().unwrap() as u32),
            duration: data.read_u8().unwrap() as u32,
        },

        // Move camera.
        0xE7 => Op::MoveCameraTo {
            x: data.read_i8().unwrap() as i32,
            y: data.read_i8().unwrap() as i32,
        },

        // Yield to the function with the next higher priority number.
        // If there is none, simply yield.
        0x00 => Op::Return,

        // Yield once or forever.
        0xB1 => Op::Yield {
            forever: false,
        },
        0xB2 => Op::Yield {
            forever: true,
        },

        // Wait durations are 1/16th of a second for NPCs, 1/64th for PCs?
        0xAD => Op::Wait {
            ticks: data.read_u8().unwrap() as u32,
            actor: ActorRef::This,
        },
        0xB9 => Op::Wait {
            ticks: 4,
            actor: ActorRef::This,
        },
        0xBA => Op::Wait {
            ticks: 8,
            actor: ActorRef::This,
        },
        0xBC => Op::Wait {
            ticks: 16,
            actor: ActorRef::This,
        },
        0xBD => Op::Wait {
            ticks: 32,
            actor: ActorRef::This,
        },

        // Script execution delay in ticks.
        0x87 => Op::SetScriptDelay {
            delay: data.read_u8().unwrap() as u32,
        },

        // Handle player character controls.
        // PC1 will respond to input. Other player characters will imitate the previous member
        // with a delay.
        0xAF => Op::Control {
            forever: false,
        },
        0xB0 => Op::Control {
            forever: true,
        },

        // Start battle.
        0xD8 => Op::Battle {
            flags: BattleFlags::from_bits_truncate(data.read_u16::<LittleEndian>().unwrap() as u32),
        },

        // Ascii text related (???)
        0x29 => Op::Unknown {
            code: 0x29,
            data: [data.read_u8().unwrap(), 0, 0, 0],
        },

        // Unknown.
        0x2C => Op::Unknown {
            code: 0x2C,
            data: [data.read_u8().unwrap(), data.read_u8().unwrap(), 0, 0],
        },

        // Unknown PC ops.
        0xFB => Op::Unknown {
            code: 0xFB,
            data: [0, 0, 0, 0],
        },
        0xFD => Op::Unknown {
            code: 0xFD,
            data: [0, 0, 0, 0],
        },
        0xA2 => Op::Unknown {
            code: 0xA2,
            data: [0, 0, 0, 0],
        },

        _ => {
            println!("Decoding unimplemented opcode 0x{:02X} as NOP", op_byte);
            Op::NOP
        },
    };

    Some(op)
}

pub fn read_script_blob(data: &mut Cursor<Vec<u8>>) -> ([u8; 32], usize) {
    let data_len = data.read_u16::<LittleEndian>().unwrap() as usize - 2;
    if data_len > 32 {
        panic!("Blob data of {} bytes is larger than the supported 32 bytes.", data_len);
    }

    let mut blob = vec![0u8; data_len];
    data.read_exact(&mut blob).unwrap();

    let mut blob_out = [0u8; 32];
    for i in 0..data_len {
        blob_out[i] = blob[i];
    }
    (blob_out, data_len)
}

pub fn read_24_bit_address(data: &mut Cursor<Vec<u8>>) -> usize {
    data.read_u8().unwrap() as usize |
        (data.read_u8().unwrap() as usize) << 8 |
        (data.read_u8().unwrap() as usize) << 16
}

pub fn read_segmented_address(data: &mut Cursor<Vec<u8>>) -> usize {
    data.read_u8().unwrap() as usize |
        (data.read_u8().unwrap() as usize) << 8
}
