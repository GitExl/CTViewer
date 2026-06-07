use crate::Context;
use crate::destination::Destination;
use crate::util::vec2di32::Vec2Di32;

pub struct SceneExit {
    pub index: usize,

    pub pos: Vec2Di32,
    pub size: Vec2Di32,
    pub destination: Destination
}

impl SceneExit {
    pub fn dump(&self, ctx: &Context) {
        println!("Scene exit {}", self.index);
        println!("  At {}, size {}", self.pos, self.size);
        self.destination.dump(ctx);

        println!();
    }
}
