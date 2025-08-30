use crate::Context;
use crate::l10n::IndexedType;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Facing {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Destination {
    Scene {
        index: usize,
        x: i32,
        y: i32,
        facing: Facing,
    },
    World {
        index: usize,
        x: i32,
        y: i32,
    },
}

impl Destination {
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
            Destination::Scene { index, x, y, facing } => {
                println!("  To scene {} - '{}', {} x {} facing {:?}", index, ctx.l10n.get_indexed(IndexedType::Scene, *index), x, y, facing);
            },
            Destination::World { index, x, y } => {
                println!("  To world {} - '{}', {} x {}", index, ctx.l10n.get_indexed(IndexedType::World, *index), x, y);
            },
        }
    }
}
