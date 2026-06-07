use crate::Context;
use crate::l10n::IndexedType;
use crate::util::vec2di32::Vec2Di32;

pub struct Treasure {
    pub id: String,
    pub tile_pos: Vec2Di32,
    pub gold: u32,
    pub item: usize,
}

impl Treasure {
    pub fn dump(&self, ctx: &Context) {
        println!("Treasure '{}'", self.id);
        println!("  At tile {}", self.tile_pos);
        if self.gold > 0 {
            println!("  Contains {} gold", self.gold);
        }
        if self.item > 0 {
            println!("  Contains '{}'", ctx.l10n.get_indexed(IndexedType::Item, self.item));
        }
        println!();
    }
}
