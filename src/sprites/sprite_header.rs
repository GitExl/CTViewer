// Sprite descriptors contain data and references to data needed to load a sprite.
pub struct SpriteHeader {
    pub index: usize,
    pub assembly_set_count: u32,

    // Index into other data.
    pub bitmap_index: usize,
    pub assembly_index: usize,
    pub palette_index: usize,
    pub anim_index: usize,

    // Unknown flags.
    pub flags: u32,

    // Hand position for enemies in battle mode.
    pub hand_x: i32,
    pub hand_y: i32,

    // Unknown enemy-related data.
    pub enemy_unknown1: u32,
    pub enemy_unknown2: u32,
    pub enemy_unknown3: u32,
}

impl SpriteHeader {
    pub fn dump(&self) {
        println!("Sprite descriptor {}", self.index);

        println!("  Bitmap 0x{:03X}, assembly 0x{:03X}, palette 0x{:03X}",
            self.bitmap_index,
            self.assembly_index,
            self.palette_index,
        );
        println!("  Assembly set count {}, animations {:03X}, flags {:0>8b}",
            self.assembly_set_count,
            self.anim_index,
            self.flags,
        );
        println!("  Hand {} x {}",
            self.hand_x,
            self.hand_y,
        );
        println!("  Unknown 1 0x{:03X}, Unknown 2 0x{:03X}, Unknown 3 0x{:03X}",
            self.enemy_unknown1,
            self.enemy_unknown2,
            self.enemy_unknown3,
        );

        println!();
    }
}
