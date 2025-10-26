use std::io::{Cursor, Seek, SeekFrom};
use byteorder::{LittleEndian, ReadBytesExt};

// 8x8 font character mapping.
const FONT_8_MAP: [&str; 256] = [
    "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶",
    "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶",
    "<BLADE>", "<BOW>", "<GUN>", "<ARM>", "<SWORD>", "<FIST>", "<SCYTHE>", "<HELM>", "<ARMOR>", "<RING>", "<H>", "<M>", "<P>", "<:>", "<SHIELD>", "<STAR>",
    "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶",
    "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶",
    "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "<LEFT>", "<RIGHT>", "(", ")", ":",
    "<HAND1>", "<HAND2>", "<HAND3>", "<HAND4>", "<H>", "<M>", "<P>", "<HP0>", "<HP1>", "<HP2>", "<HP3>", "<HP4>", "<HP5>", "<HP6>", "<HP7>", "<HP8>",
    "¶", "¶", "°", "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "<D>", "<Z>", "<UP>",
    "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶",
    "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶", "¶",
    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P",
    "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z", "a", "b", "c", "d", "e", "f",
    "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v",
    "w", "x", "y", "z", "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "!", "?",
    "/", "“", "”", ":", "&", "(", ")", "'", ".", ",", "=", "-", "+", "%", "#", " ",
    "°", "<A>", "#", "#", "<L>", "<R>", "<H>", "<M>", "<P>", "", "<CORNER>", "(", ")", "¶", "¶", " "
];

pub struct TextDecoder {
    pub substrings: Vec<String>,
}

impl TextDecoder {
    pub fn from_cursor(data: &mut Cursor<Vec<u8>>, substring_count: usize, start: u16) -> Self {

        // Read substring pointers.
        let mut pointers = vec![0u16; substring_count];
        for i in 0..substring_count {
            pointers[i] = data.read_u16::<LittleEndian>().unwrap() - start;
        }

        // Decode all substrings.
        let mut substrings = Vec::<String>::with_capacity(substring_count);
        for pointer in pointers {
            data.seek(SeekFrom::Start(pointer as u64)).unwrap();
            let len = data.read_u8().unwrap() as usize;
            substrings.push(read_substring(data, len));
        }

        TextDecoder {
            substrings,
        }
    }

    pub fn decode_huffman_string(&self, data: &mut Cursor<Vec<u8>>) -> String {
        let mut parts = Vec::<String>::new();

        loop {
            let value = data.read_u8().unwrap();

            // Strings end on a NULL character.
            if value == 0 {
                break;

            } else if value == 1 {
                parts.push(parse_character(data.read_u8().unwrap()));
            } else if value == 2 {
                parts.push(parse_character(data.read_u8().unwrap()));


            // A delay of 0 (infinite) ends the string.
            } else if value == 3 {
                let delay = data.read_u8().unwrap();
                if delay == 0 {
                    break;
                }
                parts.push(format!("<WAIT>{:02x}</WAIT><AUTO_PAGE>", data.read_u8().unwrap()));

            // A word from the dictionary.
            } else if value >= 0x21 && value <= 0x9F {
                let index = value as usize - 0x21;
                parts.push(self.substrings[index].clone());

            // Direct characters.
            } else if value > 0x9F {
                parts.push(parse_character(value));
            } else {
                parts.push(read_special_character(value, data));
            }
        }

        parts.join("")
    }

    pub fn decode_mapped_string(&self, data: Vec<u8>) -> String {
        let mut parts = Vec::<String>::new();
        for char in data {
            parts.push(FONT_8_MAP[char as usize].to_string());
        }
        parts.join("").trim_end().to_string()
    }
}

fn read_substring(data: &mut Cursor<Vec<u8>>, len: usize) -> String {
    let mut parts = Vec::<String>::new();

    for _ in 0..len {
        let value = data.read_u8().unwrap();
        if value == 0 {
            break;
        } else if value == 1 {
            parts.push(parse_character(data.read_u8().unwrap()));
        } else if value == 2 {
            parts.push(parse_character(data.read_u8().unwrap()));

        // A delay of 0 (infinite) ends the string.
        } else if value == 3 {
            let delay = data.read_u8().unwrap();
            if delay == 0 {
                break;
            }
            parts.push(format!("<WAIT>{:02x}</WAIT><AUTO_PAGE>", data.read_u8().unwrap()));

        } else if value >= 0x9F {
            parts.push(parse_character(value));
        } else {
            parts.push(read_special_character(value, data));
        }
    }

    parts.join("")
}

fn parse_character(value: u8) -> String {
    if value >= 0xA0 && value <= 0xB9 {
        return char::from(0x41 + (value - 0xA0)).to_string();
    } else if value >= 0xBA && value <= 0xD3 {
        return char::from(0x61 + (value - 0xBA)).to_string();
    } else if value >= 0xD4 && value <= 0xDD {
        return char::from(0x30 + (value - 0xD4)).to_string();
    }

    match value {
        0xDE => "!",
        0xDF => "?",
        0xE0 => "/",
        0xE1 => "“",
        0xE2 => "”",
        0xE3 => ":",
        0xE4 => "&",
        0xE5 => "(",
        0xE6 => ")",
        0xE7 => "'",
        0xE8 => ".",
        0xE9 => ",",
        0xEA => "=",
        0xEB => "-",
        0xEC => "+",
        0xED => "%",
        0xEE => "♫",
        0xEF => " ",
        0xF0 => "♥",
        0xF1 => "…",
        0xF2 => "∞",
        0xF3 => "#",
        _ => "<UNKNOWN>",
    }.to_string()
}

fn read_special_character(code: u8, data: &mut Cursor<Vec<u8>>) -> String {
    if code == 0x12 {
        return format!("<NAME_TEC>{:02x}</NAME_TEC>", data.read_u8().unwrap());
    }

    match code {
        0x04 => "<UNKNOWN04>",
        0x05 => "<BR>",
        0x06 => "<BR><INDENT>",
        0x07 => "<WAIT>03</WAIT><BR>",         // todo wait for how long exactly?
        0x08 => "<WAIT>03</WAIT><BR><INDENT>", // todo wait for how long exactly?
        0x09 => "<AUTO_PAGE>",
        0x0A => "<AUTO_PAGE><INDENT>",
        0x0B => "<PAGE>",
        0x0C => "<PAGE><INDENT>",
        0x0D => "<NUMBER 8>",
        0x0E => "<NUMBER 16>",
        0x0F => "<NUMBER 24>",
        0x10 => "<UNKNOWN10>",
        0x11 => "<SPCH 11>",  // todo display previous substring?
        0x13 => "<NAME_CRO>",
        0x14 => "<NAME_MAR>",
        0x15 => "<NAME_LUC>",
        0x16 => "<NAME_ROB>",
        0x17 => "<NAME_FRO>",
        0x18 => "<NAME_AYL>",
        0x19 => "<NAME_MAG>",
        0x1A => "<NICK_CRO>",
        0x1B => "<NAME_PT1>",
        0x1C => "<NAME_PT2>",
        0x1D => "<NAME_PT3>",
        0x1E => "<NAME_LEENE>",
        0x1F => "<NAME_ITM>",
        0x20 => "<NAME_SIL>",
        _ => "<UNKNOWN>",
    }.to_string()
}
