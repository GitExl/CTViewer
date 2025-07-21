use bitflags::bitflags;

bitflags! {
    #[derive(Clone, Default)]
    pub struct WorldChipFlags: u32 {
        const HAS_EXIT     = 0x01;
        const BLOCK_WALK   = 0x02;
        const BLOCK_HOVER  = 0x04;
        const BLOCK_FLYING = 0x08;
    }
}

#[derive(Clone, Default)]
pub struct WorldChip {
    pub flags: WorldChipFlags,
    pub music: usize,
}

pub struct WorldMap {
    pub index: usize,

    pub width: u32,
    pub height: u32,
    pub chips: Vec<WorldChip>,
}

impl WorldMap {
    pub fn dump(&self) {
        println!("World map {}", self.index);
        println!();
    }
}
