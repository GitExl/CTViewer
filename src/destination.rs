use crate::Context;
use crate::l10n::IndexedType;
use crate::util::vec2di32::Vec2Di32;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Facing {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Copy, Clone)]
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
}
