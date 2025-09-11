use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use crate::scene_script::ops::Op;
use crate::scene_script::scene_script::SceneScriptMode;
use crate::scene_script::scene_script_decoder::read_24_bit_address;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum DialoguePosition {
    Top,
    Bottom,
    Auto,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum DialogueInput {
    None,
    Line(usize),
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum DialogueSpecialType {
    CharacterSwitch,
    Load(bool),
    Save(bool),
    Shop(usize),
    RenamePC(usize),
}

impl DialogueSpecialType {
    pub fn from_value(value: u8) -> DialogueSpecialType {
        if value >= 0x80 && value <= 0xBF {
            DialogueSpecialType::Shop(value as usize - 0x80)
        } else if value >= 0xC0 && value <= 0xC7 {
            DialogueSpecialType::RenamePC(value as usize - 0xC0)
        } else if value == 0 {
            DialogueSpecialType::CharacterSwitch
        } else if value == 1 || value == 0x41 {
            DialogueSpecialType::Load(value & 0x40 > 0)
        } else if value == 2 || value == 0x40 {
            DialogueSpecialType::Save(value & 0x40 > 0)
        } else {
            panic!("Cannot determine special dialogue type from 0x{:02X}.", value)
        }
    }
}

pub fn op_decode_dialogue(op: u8, data: &mut Cursor<Vec<u8>>, mode: SceneScriptMode) -> Op {
    match op {

        // Set string table address.
        0xB8 => {
            match mode {
                SceneScriptMode::Snes => Op::DialogueSetTable {
                    address: read_24_bit_address(data),
                },
                SceneScriptMode::Pc => Op::DialogueSetTable {
                    address: data.read_u8().unwrap() as usize,
                },
            }
        },

        // Dialogue boxes.
        0xBB => Op::DialogueShow {
            index: if matches!(mode, SceneScriptMode::Snes) {
                data.read_u8().unwrap() as usize
            } else {
                data.read_u16::<LittleEndian>().unwrap() as usize
            },
            position: DialoguePosition::Auto,
            input: DialogueInput::None,
        },
        0xC0 => Op::DialogueShow {
            index: if matches!(mode, SceneScriptMode::Snes) {
                data.read_u8().unwrap() as usize
            } else {
                data.read_u16::<LittleEndian>().unwrap() as usize
            },
            position: DialoguePosition::Auto,
            input: DialogueInput::Line(data.read_u8().unwrap() as usize),
        },
        0xC1 => Op::DialogueShow {
            index: if matches!(mode, SceneScriptMode::Snes) {
                data.read_u8().unwrap() as usize
            } else {
                data.read_u16::<LittleEndian>().unwrap() as usize
            },
            position: DialoguePosition::Top,
            input: DialogueInput::None,
        },
        0xC2 => Op::DialogueShow {
            index: if matches!(mode, SceneScriptMode::Snes) {
                data.read_u8().unwrap() as usize
            } else {
                data.read_u16::<LittleEndian>().unwrap() as usize
            },
            position: DialoguePosition::Bottom,
            input: DialogueInput::None,
        },
        0xC3 => Op::DialogueShow {
            index: if matches!(mode, SceneScriptMode::Snes) {
                data.read_u8().unwrap() as usize
            } else {
                data.read_u16::<LittleEndian>().unwrap() as usize
            },
            position: DialoguePosition::Top,
            input: DialogueInput::Line(data.read_u8().unwrap() as usize),
        },
        0xC4 => Op::DialogueShow {
            index: if matches!(mode, SceneScriptMode::Snes) {
                data.read_u8().unwrap() as usize
            } else {
                data.read_u16::<LittleEndian>().unwrap() as usize
            },
            position: DialoguePosition::Bottom,
            input: DialogueInput::Line(data.read_u8().unwrap() as usize),
        },

        0xC8 => Op::DialogueSpecial {
            dialogue_type: DialogueSpecialType::from_value(data.read_u8().unwrap()),
        },

        _ => panic!("Unknown dialogue op."),
    }
}
