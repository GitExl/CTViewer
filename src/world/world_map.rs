use bitflags::bitflags;

bitflags! {
    #[derive(Copy, Clone, Default)]
    pub struct WorldChipFlags: u32 {
        const HAS_EXIT     = 0x01;
        const BLOCK_WALK   = 0x02;
        const BLOCK_LANDING  = 0x04;
        const BLOCK_FLYING = 0x08;
    }
}

#[derive(Copy, Clone, Default)]
pub struct WorldChip {
    pub flags: WorldChipFlags,
    pub music: usize,
}

#[derive(Clone)]
pub struct WorldMap {
    pub index: usize,

    pub width: u32,
    pub height: u32,

    pub pixel_width: u32,
    pub pixel_height: u32,

    pub chips: Vec<WorldChip>,
}

impl WorldMap {
    pub fn is_walkable(&self, x: i32, y: i32, width: i32, height: i32) -> bool {
        for chip_y in y..y + height {
            for chip_x in x..x + width {
                let offset = (chip_x + chip_y * self.width as i32) as usize;
                if self.chips[offset].flags.contains(WorldChipFlags::BLOCK_WALK) || self.chips[offset].flags.contains(WorldChipFlags::BLOCK_LANDING) {
                    return false;
                }
            }
        }

        true
    }

    pub fn dump(&self) {
        println!("World map {}", self.index);
        println!();
    }
}
