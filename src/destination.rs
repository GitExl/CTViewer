use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use crate::{Context, GameMode};
use crate::facing::Facing;
use crate::l10n::IndexedType;
use crate::util::vec2di32::Vec2Di32;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Destination {
    Scene {
        index: usize,
        pos: Vec2Di32,
        facing: Facing,
        data: u8,
    },
    World {
        index: usize,
        pos: Vec2Di32,
        data: u8,
    },
}

impl Destination {
    pub fn title(&self, ctx: &Context) -> String {
        match self {
            Destination::Scene { index, .. } => {
                ctx.l10n.get_indexed(IndexedType::Scene, *index)
            },
            Destination::World { index, .. } => {
                ctx.l10n.get_indexed(IndexedType::World, *index)
            },
        }
    }

    pub fn info(&self, ctx: &Context) -> String {
        match self {
            Destination::Scene { index, .. } => {
                format!("Scene {} {}", index, ctx.l10n.get_indexed(IndexedType::Scene, *index))
            },
            Destination::World { index, .. } => {
                format!("World {} {}", index, ctx.l10n.get_indexed(IndexedType::World, *index))
            },
        }
    }

    pub fn dump(&self, ctx: &Context) {
        match self {
            Destination::Scene { index, pos, facing, data } => {
                println!("  To scene {} - '{}', {} facing {:?}, data 0x{:02X}", index, ctx.l10n.get_indexed(IndexedType::Scene, *index), pos, facing, data);
            },
            Destination::World { index, pos, data } => {
                println!("  To world {} - '{}', {}, data 0x{:02X}", index, ctx.l10n.get_indexed(IndexedType::World, *index), pos, data);
            },
        }
    }

    pub fn from_data(index: usize, facing: Facing, tile_x: i32, tile_y: i32, shift_x: i32, shift_y: i32, data: u8) -> Destination {
        if index >= 0x1F0 && index <= 0x1FF {
            Destination::World {
                index: index - 0x1F0,
                pos: Vec2Di32::new(
                    tile_x * 8 + shift_x,
                    tile_y * 8 + shift_y,
                ),
                data,
            }
        } else {
            Destination::Scene {
                index,
                facing,
                pos: Vec2Di32::new(
                    tile_x * 16 + 8 + shift_x,
                    tile_y * 16 + 15 + shift_y,
                ),
                data,
            }
        }
    }

    pub fn from_cursor(data: &mut Cursor<Vec<u8>>, mode: GameMode) -> Destination {
        match mode {
            GameMode::Snes => {
                let index_facing = data.read_u16::<LittleEndian>().unwrap() as usize;
                let index = index_facing & 0x01FF;
                let last_facing_byte = (index_facing >> 8) as u8;
                let facing = ((last_facing_byte >> 1) & 0x0F) | (last_facing_byte & 0x80);
                let tile_x = data.read_u8().unwrap() as i32;
                let tile_y = data.read_u8().unwrap() as i32;

                Destination::from_data(index, Facing::from_data(facing), tile_x, tile_y, 0, 0, facing)
            },
            GameMode::Pc => {
                let index = data.read_u16::<LittleEndian>().unwrap() as usize;
                let facing = data.read_u8().unwrap();
                let tile_x = data.read_u8().unwrap() as i32;
                let tile_y = data.read_u8().unwrap() as i32;

                Destination::from_data(index, Facing::from_data(facing), tile_x, tile_y, 0, 0, facing)
            }
        }
    }

    pub fn get_index(&self) -> usize {
        match self {
            Destination::World { index, .. } => *index + 0x1FF,
            Destination::Scene { index, .. } => *index,
        }
    }

    pub fn as_string(&self) -> String {
        match self {
            Destination::World { index, pos, data } => {
                format!("World({}, {}, {}, 0x{:02X})", index, pos.x, pos.y, data)
            },
            Destination::Scene { index, pos, facing, data } => {
                format!("Scene({}, {}, {}, {:?}, 0x{:02})", index, pos.x, pos.y, facing, data)
            }
        }
    }
}
