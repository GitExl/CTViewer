use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use crate::scene::textbox::TextBoxPosition;
use crate::scene_script::ops::Op;
use crate::scene_script::scene_script::SceneScriptMode;
use crate::scene_script::scene_script_decoder::read_24_bit_address;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum TextBoxInput {
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

pub fn op_decode_textbox(op: u8, data: &mut Cursor<Vec<u8>>, mode: SceneScriptMode) -> Op {
    match op {

        // Set string table address.
        0xB8 => {
            match mode {
                SceneScriptMode::Snes => Op::TextSetTable {
                    address: read_24_bit_address(data) - 0xC00000,
                },
                SceneScriptMode::Pc => Op::TextSetTable {
                    address: data.read_u8().unwrap() as usize,
                },
            }
        },

        // Textboxes.
        0xBB => Op::TextBoxShow {
            index: if matches!(mode, SceneScriptMode::Snes) {
                data.read_u8().unwrap() as usize
            } else {
                data.read_u16::<LittleEndian>().unwrap() as usize
            },
            position: TextBoxPosition::Auto,
            input: TextBoxInput::None,
        },
        0xC0 => Op::TextBoxShow {
            index: if matches!(mode, SceneScriptMode::Snes) {
                data.read_u8().unwrap() as usize
            } else {
                data.read_u16::<LittleEndian>().unwrap() as usize
            },
            position: TextBoxPosition::Auto,
            input: TextBoxInput::Line(data.read_u8().unwrap() as usize),
        },
        0xC1 => Op::TextBoxShow {
            index: if matches!(mode, SceneScriptMode::Snes) {
                data.read_u8().unwrap() as usize
            } else {
                data.read_u16::<LittleEndian>().unwrap() as usize
            },
            position: TextBoxPosition::Top,
            input: TextBoxInput::None,
        },
        0xC2 => Op::TextBoxShow {
            index: if matches!(mode, SceneScriptMode::Snes) {
                data.read_u8().unwrap() as usize
            } else {
                data.read_u16::<LittleEndian>().unwrap() as usize
            },
            position: TextBoxPosition::Bottom,
            input: TextBoxInput::None,
        },
        0xC3 => Op::TextBoxShow {
            index: if matches!(mode, SceneScriptMode::Snes) {
                data.read_u8().unwrap() as usize
            } else {
                data.read_u16::<LittleEndian>().unwrap() as usize
            },
            position: TextBoxPosition::Top,
            input: TextBoxInput::Line(data.read_u8().unwrap() as usize),
        },
        0xC4 => Op::TextBoxShow {
            index: if matches!(mode, SceneScriptMode::Snes) {
                data.read_u8().unwrap() as usize
            } else {
                data.read_u16::<LittleEndian>().unwrap() as usize
            },
            position: TextBoxPosition::Bottom,
            input: TextBoxInput::Line(data.read_u8().unwrap() as usize),
        },

        // Special dialogues.
        0xC8 => Op::DialogueSpecial {
            dialogue_type: DialogueSpecialType::from_value(data.read_u8().unwrap()),
        },

        _ => panic!("Unknown textbox op."),
    }
}
