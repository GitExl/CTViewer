use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use crate::GameMode;
use crate::scene::textbox::TextBoxPosition;
use crate::scene_script::scene_script_ops::Op;
use crate::util::data_read::read_24_bit_address;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum UiType {
    CharacterSwitch,
    Load(bool),
    Save(bool),
    Shop(usize),
    RenamePC(usize),
}

impl UiType {
    pub fn from_value(value: u8) -> UiType {
        if value >= 0x80 && value <= 0xBF {
            UiType::Shop(value as usize - 0x80)
        } else if value >= 0xC0 && value <= 0xC7 {
            UiType::RenamePC(value as usize - 0xC0)
        } else if value == 0 {
            UiType::CharacterSwitch
        } else if value == 1 || value == 0x41 {
            UiType::Load(value & 0x40 > 0)
        } else if value == 2 || value == 0x40 {
            UiType::Save(value & 0x40 > 0)
        } else {
            panic!("Cannot determine special dialogue type from 0x{:02X}.", value)
        }
    }
}

pub fn op_decode_textbox(op: u8, data: &mut Cursor<Vec<u8>>, mode: GameMode) -> Op {
    match op {

        // Set string table address.
        // "msegg"
        0xB8 => {
            match mode {
                GameMode::Snes => Op::TextSetTable {
                    address: read_24_bit_address(data) - 0xC00000,
                },
                GameMode::Pc => Op::TextSetTable {
                    address: data.read_u8().unwrap() as usize,
                },
            }
        },

        // Textboxes.
        // "mes"
        0xBB => Op::TextBoxShow {
            index: if matches!(mode, GameMode::Snes) {
                data.read_u8().unwrap() as usize
            } else {
                data.read_u16::<LittleEndian>().unwrap() as usize
            },
            position: TextBoxPosition::Auto,
            choice_lines: None,
        },
        // "query"
        0xC0 => Op::TextBoxShow {
            index: if matches!(mode, GameMode::Snes) {
                data.read_u8().unwrap() as usize
            } else {
                data.read_u16::<LittleEndian>().unwrap() as usize
            },
            position: TextBoxPosition::Auto,
            choice_lines: get_textbox_choice_lines(data.read_u8().unwrap()),
        },
        // "mesu"
        0xC1 => Op::TextBoxShow {
            index: if matches!(mode, GameMode::Snes) {
                data.read_u8().unwrap() as usize
            } else {
                data.read_u16::<LittleEndian>().unwrap() as usize
            },
            position: TextBoxPosition::Top,
            choice_lines: None,
        },
        // "mesl"
        0xC2 => Op::TextBoxShow {
            index: if matches!(mode, GameMode::Snes) {
                data.read_u8().unwrap() as usize
            } else {
                data.read_u16::<LittleEndian>().unwrap() as usize
            },
            position: TextBoxPosition::Bottom,
            choice_lines: None,
        },
        // "queryu"
        0xC3 => Op::TextBoxShow {
            index: if matches!(mode, GameMode::Snes) {
                data.read_u8().unwrap() as usize
            } else {
                data.read_u16::<LittleEndian>().unwrap() as usize
            },
            position: TextBoxPosition::Top,
            choice_lines: get_textbox_choice_lines(data.read_u8().unwrap()),
        },
        // "queryl"
        0xC4 => Op::TextBoxShow {
            index: if matches!(mode, GameMode::Snes) {
                data.read_u8().unwrap() as usize
            } else {
                data.read_u16::<LittleEndian>().unwrap() as usize
            },
            position: TextBoxPosition::Bottom,
            choice_lines: get_textbox_choice_lines(data.read_u8().unwrap()),
        },

        // Special UI.
        // "menu"
        0xC8 => Op::OpenUi {
            ui: UiType::from_value(data.read_u8().unwrap()),
        },

        _ => panic!("Unknown textbox op."),
    }
}

fn get_textbox_choice_lines(value: u8) -> Option<[usize; 2]> {
    Some([
        (value >> 2) as usize & 0x3,
        (value >> 0) as usize & 0x3,
    ])
}
