use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use crate::scene_script::ops::Op;
use crate::scene_script::scene_script::SceneScriptMode;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum CharacterType {
    PC,
    PCAsNPC,
    NPC,
    Enemy,
}

pub fn op_decode_char_load(op: u8, data: &mut Cursor<Vec<u8>>, mode: SceneScriptMode) -> Op {
    match op {
        0x57 => Op::LoadCharacterPlayer {
            character_index: 0,
            must_be_active: true,
            battle_index: 0,
        },
        0x5C => Op::LoadCharacterPlayer {
            character_index: 1,
            must_be_active: true,
            battle_index: 0,
        },
        0x62 => Op::LoadCharacterPlayer {
            character_index: 2,
            must_be_active: true,
            battle_index: 0,
        },
        0x68 => Op::LoadCharacterPlayer {
            character_index: 4,
            must_be_active: true,
            battle_index: 0,
        },
        0x6A => Op::LoadCharacterPlayer {
            character_index: 3,
            must_be_active: true,
            battle_index: 0,
        },
        0x6C => Op::LoadCharacterPlayer {
            character_index: 5,
            must_be_active: true,
            battle_index: 0,
        },
        0x6D => Op::LoadCharacterPlayer {
            character_index: 6,
            must_be_active: true,
            battle_index: 0,
        },

        0x80 => Op::LoadCharacterPlayer {
            character_index: data.read_u8().unwrap() as usize,
            must_be_active: true,
            battle_index: 0,
        },
        0x81 => Op::LoadCharacter {
            char_type: CharacterType::PCAsNPC,
            index: data.read_u8().unwrap() as usize,
            is_static: false,
            battle_index: 0,
        },
        0x82 => Op::LoadCharacter {
            char_type: CharacterType::NPC,
            index: data.read_u8().unwrap() as usize,
            is_static: false,
            battle_index: 0,
        },
        0x83 => {
            let index = match mode {
                SceneScriptMode::Snes => data.read_u8().unwrap() as usize,
                SceneScriptMode::Pc => data.read_u16::<LittleEndian>().unwrap() as usize,
            };
            let bits = data.read_u8().unwrap();
            Op::LoadCharacter {
                char_type: CharacterType::Enemy,
                index: match mode {
                    SceneScriptMode::Snes => index,
                    SceneScriptMode::Pc => index + 7,
                },
                is_static: bits & 0x80 > 0,
                battle_index: bits as usize & 0x7F,
            }
        },

        _ => panic!("Unknown character load op."),
    }
}
