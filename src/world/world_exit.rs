use crate::Context;
use crate::destination::Destination;
use crate::l10n::IndexedType;
use crate::util::vec2di32::Vec2Di32;

#[derive(Clone)]
pub struct WorldExit {
    pub index: usize,

    pub pos: Vec2Di32,
    pub is_available: bool,
    pub name_index: usize,

    pub destination: Destination,
    pub unknown: u32,
}

impl WorldExit {
    pub fn dump(&self, ctx: &Context) {
        println!("World exit {} - {}", self.index, ctx.l10n.get_indexed(IndexedType::WorldExit, self.name_index));
        println!("  At {}", self.pos);
        println!("  Available: {}", self.is_available);
        self.destination.dump(ctx);

        println!("  Unknown: {}", self.unknown);
        println!();
    }
}

#[derive(Clone, Copy)]
pub struct ScriptedWorldExit {
    pub index: usize,
    pub pos: Vec2Di32,
    pub script_offset_index: usize,
}

impl ScriptedWorldExit {
    pub fn dump(&self) {
        println!("Scripted world exit {}", self.index);
        println!("  At {}", self.pos);
        println!("  Script offset index: {}", self.script_offset_index);
        println!();
    }
}