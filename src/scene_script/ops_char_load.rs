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
        0x57 => Op::LoadCharacter {
            char_type: CharacterType::PC,
            index: 0,
            must_be_in_party: true,
            is_static: false,
            battle_index: 0,
        },
        0x5C => Op::LoadCharacter {
            char_type: CharacterType::PC,
            index: 1,
            must_be_in_party: true,
            is_static: false,
            battle_index: 0,
        },
        0x62 => Op::LoadCharacter {
            char_type: CharacterType::PC,
            index: 2,
            must_be_in_party: true,
            is_static: false,
            battle_index: 0,
        },
        0x68 => Op::LoadCharacter {
            char_type: CharacterType::PC,
            index: 3,
            must_be_in_party: true,
            is_static: false,
            battle_index: 0,
        },
        0x6A => Op::LoadCharacter {
            char_type: CharacterType::PC,
            index: 4,
            must_be_in_party: true,
            is_static: false,
            battle_index: 0,
        },
        0x6C => Op::LoadCharacter {
            char_type: CharacterType::PC,
            index: 5,
            must_be_in_party: true,
            is_static: false,
            battle_index: 0,
        },
        0x6D => Op::LoadCharacter {
            char_type: CharacterType::PC,
            index: 6,
            must_be_in_party: true,
            is_static: false,
            battle_index: 0,
        },

        0x80 => Op::LoadCharacter {
            char_type: CharacterType::PC,
            index: data.read_u8().unwrap() as usize,
            must_be_in_party: true,
            is_static: false,
            battle_index: 0,
        },
        0x81 => Op::LoadCharacter {
            char_type: CharacterType::PCAsNPC,
            index: data.read_u8().unwrap() as usize,
            must_be_in_party: false,
            is_static: false,
            battle_index: 0,
        },
        0x82 => Op::LoadCharacter {
            char_type: CharacterType::NPC,
            index: data.read_u8().unwrap() as usize,
            must_be_in_party: false,
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
                index,
                must_be_in_party: false,
                is_static: bits & 0x80 > 0,
                battle_index: bits as usize & 0x7F,
            }
        },

        _ => panic!("Unknown character load op."),
    }
}
