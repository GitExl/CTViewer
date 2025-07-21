use bitflags::bitflags;

bitflags! {
    #[derive(Clone, Default)]
    pub struct SpriteAssemblyTileFlags: u32 {
        const FLIP_X = 0x01;
        const FLIP_Y = 0x02;
        const UNUSED = 0x04;
        const UNKNOWN = 0x08;
    }
}

// An assembly tile is drawn at a position, analogous to an SNES sprite.
#[derive(Clone, Default)]
pub struct SpriteAssemblyTile {
    pub x: i32,
    pub y: i32,
    pub src_x: i32,
    pub src_y: i32,
    pub width: i32,
    pub height: i32,
    pub chip: usize,
    pub flags: SpriteAssemblyTileFlags,
}

// A frame lists tiles to draw.
#[derive(Clone)]
pub struct SpriteAssemblyFrame {
    pub tiles: Vec<SpriteAssemblyTile>,
}

impl SpriteAssemblyFrame {
    pub fn new() -> SpriteAssemblyFrame {
        SpriteAssemblyFrame {
            tiles: Vec::new(),
        }
    }
}

// A sprite assembly lists sprite frames, which in turn are assembled from 16x16 tiles of
// graphics data.
pub struct SpriteAssembly {
    pub index: usize,
    pub chip_max: usize,
    pub frames: Vec<SpriteAssemblyFrame>,
}

impl SpriteAssembly {
    pub fn new(index: usize) -> SpriteAssembly {
        SpriteAssembly {
            index,
            chip_max: 0,
            frames: Vec::new(),
        }
    }

    pub fn dump(&self) {
        println!("Sprite assembly {}", self.index);

        println!("  Chip max {}, {} frames",
            self.chip_max,
            self.frames.len(),
        );

        for (frame_index, frame) in self.frames.iter().enumerate() {
            println!("    Frame {}, {} tiles",
            frame_index,
                frame.tiles.len(),
            );

            for tile in &frame.tiles {
                println!("      Tile {:>5} {:0>16b}, x {:>4}, y {:>4}, {:>2}x{:>2} {:>6} {:>6} {:>6} {:>7}",
                    tile.chip, tile.chip,
                    tile.x, tile.y,
                    tile.width, tile.height,
                    if tile.flags.contains(SpriteAssemblyTileFlags::FLIP_X) { "FLIP_X" } else { "" },
                    if tile.flags.contains(SpriteAssemblyTileFlags::FLIP_Y) { "FLIP_Y" } else { "" },
                    if tile.flags.contains(SpriteAssemblyTileFlags::UNUSED) { "UNUSED" } else { "" },
                    if tile.flags.contains(SpriteAssemblyTileFlags::UNKNOWN) { "UNKNOWN" } else { "" },
                );
            }
        }

        println!();
    }
}
