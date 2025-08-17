use crate::l10n::{IndexedType, L10n};

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
    pub fn info(&self, l10n: &L10n) -> String {
        match self {
            Destination::Scene { index, .. } => {
                format!("Scene {} {}", index, l10n.get_indexed(IndexedType::Scene, *index))
            },
            Destination::World { index, .. } => {
                format!("World {} {}", index, l10n.get_indexed(IndexedType::World, *index))
            },
        }
    }

    pub fn dump(&self, l10n: &L10n) {
        match self {
            Destination::Scene { index, x, y, facing } => {
                println!("  To scene {} - '{}', {} x {} facing {:?}", index, l10n.get_indexed(IndexedType::Scene, *index), x, y, facing);
            },
            Destination::World { index, x, y } => {
                println!("  To world {} - '{}', {} x {}", index, l10n.get_indexed(IndexedType::World, *index), x, y);
            },
        }
    }
}
