use crate::Context;
use crate::destination::Destination;
use crate::l10n::IndexedType;
use crate::util::vec2di32::Vec2Di32;

#[derive(Clone)]
pub enum WorldExitType {
    Destination {
        destination: Destination,
    },
    Scripted {
        pointer_index: usize,
    }
}

#[derive(Clone)]
pub struct WorldExit {
    pub index: usize,

    pub pos: Vec2Di32,
    pub is_available: bool,
    pub name_index: usize,

    pub exit_type: WorldExitType,
    pub unknown: u32,
}

impl WorldExit {
    pub fn dump(&self, ctx: &Context) {
        println!("World exit {} - {}", self.index, ctx.l10n.get_indexed(IndexedType::WorldExit, self.name_index));
        println!("  At {}", self.pos);
        println!("  Available: {}", self.is_available);

        match self.exit_type {
            WorldExitType::Destination { destination } => destination.dump(ctx),
            WorldExitType::Scripted { pointer_index } => println!("Scripted pointer {}", pointer_index),
        }
        
        println!("  Unknown: {}", self.unknown);
        println!();
    }
}

#[derive(Clone, Copy)]
pub struct WorldTrigger {
    pub index: usize,
    pub pos: Vec2Di32,
    pub script_address_index: usize,
    pub is_available: bool,
}

impl WorldTrigger {
    pub fn dump(&self) {
        println!("World trigger {}", self.index);
        println!("  At {}", self.pos);
        println!("  Script address index: {}", self.script_address_index);
        println!("  Available: {}", self.is_available);
        println!();
    }
}
