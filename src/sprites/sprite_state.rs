use bitflags::bitflags;
use crate::software_renderer::palette::Palette;
use crate::sprites::sprite_renderer::SpritePriority;
use crate::util::vec2df64::Vec2Df64;

bitflags! {
    #[derive(Copy, Clone, Default)]
    pub struct SpriteStateFlags: u8 {
        const ENABLED = 0x01;
        const CAMERA_RELATIVE = 0x02;
    }
}

#[derive(Clone,PartialEq,Debug)]
pub enum AnimationMode {
    None,
    Loop,
    LoopCount,
    Static,
}

#[derive(Clone)]
pub struct SpriteState {
    pub flags: SpriteStateFlags,

    pub pos: Vec2Df64,

    pub priority_top: SpritePriority,
    pub priority_bottom: SpritePriority,
    pub sort_weight: i32,

    pub assembly_key: u64,
    pub bitmap_key: u64,

    pub palette: Palette,
    pub palette_offset: usize,
}

impl SpriteState {
    pub fn new() -> SpriteState {
        SpriteState {
            flags: SpriteStateFlags::empty(),

            pos: Vec2Df64::default(),

            priority_top: SpritePriority::default(),
            priority_bottom: SpritePriority::default(),
            sort_weight: 0,

            assembly_key: 0,
            bitmap_key: 0,

            palette: Palette::new(0),
            palette_offset: 0,
        }
    }



    pub fn dump(&self) {
        println!("Sprite state - {}", if self.flags.contains(SpriteStateFlags::ENABLED) { "enabled" } else { "disabled" });
        println!("  Assembly key 0x{:X}", self.assembly_key);
        println!("  At {}", self.pos);
        println!("  Priority top {:?}", self.priority_top);
        println!("  Priority bottom {:?}", self.priority_bottom);
        println!("  Sort weight {}", self.sort_weight);
        println!("  Palette {} colors", self.palette.colors.len());
        println!();
    }
}
