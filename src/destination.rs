use crate::Context;
use crate::facing::Facing;
use crate::l10n::IndexedType;
use crate::util::vec2di32::Vec2Di32;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Destination {
    Scene {
        index: usize,
        pos: Vec2Di32,
        facing: Facing,
    },
    World {
        index: usize,
        pos: Vec2Di32,
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
            Destination::Scene { index, pos, facing } => {
                println!("  To scene {} - '{}', {} facing {:?}", index, ctx.l10n.get_indexed(IndexedType::Scene, *index), pos, facing);
            },
            Destination::World { index, pos } => {
                println!("  To world {} - '{}', {}", index, ctx.l10n.get_indexed(IndexedType::World, *index), pos);
            },
        }
    }

    pub fn from_data(index: usize, facing: Facing, tile_x: i32, tile_y: i32, shift_x: i32, shift_y: i32) -> Destination {
        if index >= 0x1F0 && index <= 0x1FF {
            Destination::World {
                index: index - 0x1F0,
                pos: Vec2Di32::new(
                    tile_x * 8 + shift_x,
                    tile_y * 8 + shift_y,
                ),
            }
        } else {
            Destination::Scene {
                index,
                facing,
                pos: Vec2Di32::new(
                    tile_x * 16 + 8 + shift_x,
                    tile_y * 16 + 15 + shift_y,
                ),
            }
        }
    }

    pub fn get_index(&self) -> usize {
        match self {
            Destination::World { index, .. } => *index + 0x1FF,
            Destination::Scene { index, .. } => *index,
        }
    }
}
