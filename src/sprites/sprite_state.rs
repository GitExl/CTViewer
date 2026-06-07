use crate::software_renderer::palette::Palette;
use crate::sprites::sprite_renderer::SpritePriority;
use crate::util::vec2df64::Vec2Df64;

#[derive(Clone,PartialEq,Debug)]
pub enum AnimationMode {
    None,
    Loop,
    LoopCount,
    Static,
}

#[derive(Clone)]
pub struct SpriteState {
    pub enabled: bool,

    pub pos: Vec2Df64,

    pub priority_top: SpritePriority,
    pub priority_bottom: SpritePriority,

    pub assembly_key: u64,
    pub bitmap_index: usize,

    pub palette: Palette,
    pub palette_offset: usize,
}

impl SpriteState {
    pub fn new() -> SpriteState {
        SpriteState {
            enabled: false,

            pos: Vec2Df64::default(),

            priority_top: SpritePriority::default(),
            priority_bottom: SpritePriority::default(),

            assembly_key: 0,
            bitmap_index: 0,

            palette: Palette::new(0),
            palette_offset: 0,
        }
    }



    pub fn dump(&self) {
        println!("Sprite state - {}", if self.enabled { "enabled" } else { "disabled" });
        println!("  Assembly key 0x{:X}", self.assembly_key);
        println!("  At {}", self.pos);
        println!("  Priority top {:?}", self.priority_top);
        println!("  Priority bottom {:?}", self.priority_bottom);
        println!("  Palette {} colors", self.palette.colors.len());
        println!();
    }
}
