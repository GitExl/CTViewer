use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use crate::assets::Assets;
use crate::Context;
use crate::sprites::sprite_assembly::{SpriteAssemblyChip, SpriteAssemblyChipFlags, SpriteAssemblyFrame};
use crate::util::data_read::read_24_bit_address;
use crate::world_script::world_script::WorldActorState;

pub struct WorldAnimationScript {
    offsets: Vec<u64>,
    data: Cursor<Vec<u8>>,
}

#[derive(Debug)]
pub enum WorldAnimationOp {
    Reset {
        address: usize,
    },
    Increment {
        address: usize,
    },
    Decrement {
        address: usize,
    },
    Goto {
        offset: i64,
    },
    Animate {
        assembly_address: u64,
        duration: u8,
    },
    Wait {
        duration: u8,
    },
    Transfer {
        address: u64,
        unknown1: u16,
        unknown2: u16,
    },
    Unknown7,
}

// World animation scripts mix animation frame and sprite assembly data. The world loads the
// graphics tile data at the top of VRAM. At various points, the game loads in different graphics
// data. Animations require the correct graphics data to be loaded for them to look as intended.
impl WorldAnimationScript {
    pub fn new(data: &Vec<u8>, count: usize) -> Self {
        let mut local_data = Cursor::new(data.clone());

        let mut offsets = Vec::new();
        for _ in 0..count {
            offsets.push(local_data.read_u16::<LittleEndian>().unwrap() as u64 - 0xE000);
        }

        WorldAnimationScript {
            offsets,
            data: local_data,
        }
    }

    pub fn get_animation_address(&self, animation_index: usize) -> u64 {
        self.offsets[animation_index]
    }

    pub fn run(&mut self, ctx: &mut Context, state: &mut WorldActorState) {
        if state.animation_address == 0 {
            return;
        }

        let op = self.decode(state.animation_address);
        match op {
            WorldAnimationOp::Reset { address } => {
                state.memory.put_u8(address, 0);
                state.animation_address += 2;
            },
            WorldAnimationOp::Increment { address } => {
                let value = state.memory.get_u8(address);
                state.memory.put_u8(address, value + 1);
                state.animation_address += 2;
            },
            WorldAnimationOp::Decrement { address } => {
                let value = state.memory.get_u8(address);
                state.memory.put_u8(address, value - 1);
                state.animation_address += 2;
            },
            WorldAnimationOp::Goto { offset } => {
                state.animation_address = (state.animation_address as i64 + offset) as u64
            },
            WorldAnimationOp::Animate { assembly_address, duration } => {

                // Always set frame.
                if state.palette_priority & 0x40 != 0 {
                    state.sprite_assembly_key = self.read_sprite_assembly(ctx, assembly_address);

                // Countdown.
                } else if state.animation_counter != 0 {
                    state.animation_counter -= 1;

                    // Countdown complete, advance to next op.
                    if state.animation_counter == 0 {
                        state.animation_address += 4;
                    }

                // Start wait.
                } else {
                    state.animation_counter = duration;
                    state.sprite_assembly_key = self.read_sprite_assembly(ctx, assembly_address);
                }
            },
            WorldAnimationOp::Wait { duration } => {

                // Countdown.
                if state.animation_counter != 0 {
                    state.animation_counter -= 1;

                    // Countdown complete, advance to next op.
                    if state.animation_counter == 0 {
                        state.animation_address += 2;
                    }

                // Start wait.
                } else {
                    state.animation_counter = duration;
                }
            },
            WorldAnimationOp::Transfer { .. } => {
                state.animation_address += 8;
            },
            WorldAnimationOp::Unknown7 => {
                state.animation_address += 1;
            },
        }
    }

    fn decode(&mut self, address: u64) -> WorldAnimationOp {
        self.data.set_position(address);
        let opcode = self.data.read_u8().unwrap();
        let decoded = match opcode {
            0 => WorldAnimationOp::Reset {
                address: self.data.read_u8().unwrap() as usize,
            },
            1 => WorldAnimationOp::Increment {
                address: self.data.read_u8().unwrap() as usize,
            },
            2 => WorldAnimationOp::Decrement {
                address: self.data.read_u8().unwrap() as usize,
            },
            3 => WorldAnimationOp::Goto {
                offset: self.data.read_i8().unwrap() as i64,
            },
            4 => WorldAnimationOp::Animate {
                assembly_address: self.data.read_u16::<LittleEndian>().unwrap() as u64,
                duration: self.data.read_u8().unwrap(),
            },
            5 => WorldAnimationOp::Wait {
                duration: self.data.read_u8().unwrap(),
            },
            6 => WorldAnimationOp::Transfer {
                address: read_24_bit_address(&mut self.data) as u64,
                unknown1: self.data.read_u16::<LittleEndian>().unwrap(),
                unknown2: self.data.read_u16::<LittleEndian>().unwrap(),
            },
            7 => WorldAnimationOp::Unknown7,
            _ => panic!("Unknown world animation opcode {} at 0x{:04X}", opcode, address),
        };
        decoded
    }

    pub fn disassemble(&mut self) {
        let offsets = self.offsets.clone();
        for (index, offset) in offsets.iter().enumerate() {
            let mut op_address = *offset;

            println!("Anim {} @ {:04X}", index, offset);
            loop {
                self.data.set_position(op_address);
                let op = self.decode(op_address);

                match op {
                    WorldAnimationOp::Reset { address } => {
                        println!("  {:04X} reset 0x{:02X}", op_address, address);
                    }
                    WorldAnimationOp::Increment { address } => {
                        println!("  {:04X} inc 0x{:02X}", op_address, address);
                    }
                    WorldAnimationOp::Decrement { address } => {
                        println!("  {:04X} dec 0x{:02X}", op_address, address);
                    }
                    WorldAnimationOp::Goto { offset } => {
                        println!("  {:04X} goto 0x{:04X}", op_address, op_address as i64 + offset);
                        break;
                    }
                    WorldAnimationOp::Wait { duration } => {
                        println!("  {:04X} wait {}", op_address, duration);
                        if duration == 0 {
                            break;
                        }
                    }
                    WorldAnimationOp::Animate { duration, assembly_address } => {
                        println!("  {:04X} animate 0x{:02X} {}", op_address, assembly_address, duration);
                        if duration == 0 {
                            break;
                        }
                    }
                    WorldAnimationOp::Transfer { address, unknown1, unknown2 } => {
                        println!("  {:04X} transfer 0x{:06X} {} {}", op_address, address, unknown1, unknown2);
                    }
                    WorldAnimationOp::Unknown7 => {
                        println!("  {:04X} unknown07", op_address);
                    }
                }

                op_address = self.data.position()
            }
            println!();
        }
    }

    fn read_sprite_assembly(&mut self, ctx: &mut Context, assembly_address: u64) -> u64 {

        // Re-use already loaded frame assembly.
        let assembly_frame_key = Assets::asset_key_sprite_assembly_frame_world(assembly_address);
        if ctx.assets.has_assembly_frame(assembly_frame_key) {
            return assembly_frame_key;
        }

        // Read frame assembly data from the position, but keep track of the current position so we
        // can return here later.
        let old_pos = self.data.position();
        self.data.set_position(assembly_address - 0xE000);

        let tile_count = self.data.read_u8().unwrap();
        let mut frame = SpriteAssemblyFrame {
            chips: Vec::new(),
        };

        // - A byte with number of tiles in this frame
        // ...then any number of tiles:
        // - A signed byte with the x offset of the tile
        // - A signed byte with the y offset of the tile
        // - An unsigned 16-bit integer that contains the SNES VRAM tile index and flags
        //   for this tile
        for _ in 0..tile_count {
            let x = self.data.read_i8().unwrap() as i32;
            let y = self.data.read_i8().unwrap() as i32;
            let mut chip_index = self.data.read_u16::<LittleEndian>().unwrap() as usize;

            let mut flags: SpriteAssemblyChipFlags = SpriteAssemblyChipFlags::default();
            if chip_index & 0x2000 > 0 {
                flags |= SpriteAssemblyChipFlags::UNKNOWN;
            }
            if chip_index & 0x4000 > 0 {
                flags |= SpriteAssemblyChipFlags::FLIP_X;
            }
            if chip_index & 0x8000 > 0 {
                flags |= SpriteAssemblyChipFlags::FLIP_Y;
            }

            chip_index &= 0x1FFF;
            let src_x = ((chip_index % 32) * 8) as i32;
            let src_y = ((chip_index / 32) * 16) as i32;

            frame.chips.push(SpriteAssemblyChip {
                x, y,
                width: 16, height: 16,
                src_x, src_y,
                src_index: chip_index,
                flags,
            });
        }

        self.data.set_position(old_pos);
        ctx.assets.add_assembly_frame(assembly_frame_key, frame);

        assembly_frame_key
    }

}

pub fn get_animation_description(index: usize) -> String {
    let description = match index {
        0 => "PC facing down",
        1 => "PC walk down",
        2 => "PC walk down (half)",
        3 => "PC facing up",
        4 => "PC walking up",
        5 => "PC walking up (half)",
        6 => "PC facing left",
        7 => "PC walking left",
        8 => "PC walking left (half)",
        9 => "PC facing right",
        10 => "PC walking right",
        11 => "PC walking right (half)",

        12 => "PC facing down",
        13 => "PC walk down",
        14 => "PC walk down (half)",
        15 => "PC facing up",
        16 => "PC walking up",
        17 => "PC walking up (half)",
        18 => "PC facing left",
        19 => "PC walking left",
        20 => "PC walking left (half)",
        21 => "PC facing right",
        22 => "PC walking right",
        23 => "PC walking right (half)",

        24 => "Unknown",
        25 => "Unknown",
        26 => "Unknown",
        27 => "Unknown",
        28 => "Unknown",
        29 => "Unknown",
        30 => "Unknown",
        31 => "Shadow",
        32 => "Smoke puff",
        33 => "Smoke puff",
        34 => "Seagull up",
        35 => "Balloon",
        36 => "Dactyl sitting",
        37 => "Dactyl down",
        38 => "Dactyl up",
        39 => "Dactyl left",
        40 => "Dactyl right",
        41 => "Dactyl down",
        42 => "Dactyl down",
        43 => "Dactyl up",
        44 => "Dactyl left",
        45 => "Unknown",
        46 => "Null",
        47 => "Null",
        48 => "Null",
        49 => "Null",
        50 => "Unknown",
        51 => "Unknown",
        52 => "Unknown",
        53 => "Unknown",
        54 => "Unknown",
        55 => "Unknown",
        56 => "Unknown",
        57 => "Unknown",
        58 => "Unknown",
        59 => "Unknown",
        60 => "Unknown",
        61 => "Unknown",
        62 => "Unknown",
        63 => "Unknown",
        64 => "Unknown",
        65 => "Unknown",
        66 => "Unknown",
        67 => "Unknown",
        68 => "Unknown",
        69 => "Unknown",
        70 => "Unknown",
        71 => "Unknown",
        72 => "Steamboat facing up",
        73 => "Steamboat facing down",
        74 => "Steamboat facing left",
        75 => "Steamboat facing right",
        76 => "Vortex top",
        77 => "Vortex bottom",
        78 => "Epoch shadow",
        79 => "Seagull facing down",
        80 => "Seagull facing left",
        81 => "Magus’ castle",
        82 => "Ozzie’s castle",
        83 => "Bat",
        84 => "Debris impact",
        85 => "Lavos surface crack small",
        86 => "Lavos surface crack medium",
        87 => "Lavos surface crack large",
        88 => "Lavos surface hole background",
        89 => "Lavos surface hole foreground",
        90 => "Lavos lavarock large",
        91 => "Lavos lavarock medium",
        92 => "Lavos lavarock small",
        93 => "Lavos lavarock tiny",
        94 => "Lavos",
        95 => "Lavos",
        96 => "Lavos fire streak",
        97 => "Lavos fire streak",
        98 => "Lavos fire streak",
        99 => "Lavos fire streak",
        100 => "Lavos fire streak",
        101 => "Lavos fire streak",
        102 => "Lavos fire streak",
        103 => "Lavos fire streak",
        104 => "Lavos fire streak",
        105 => "Lavos fire streak",
        106 => "Debris impact",
        107 => "Lavos surface crack mask?",
        108 => "Unknown",
        109 => "Unknown",
        110 => "Unknown",
        111 => "Unknown",
        112 => "Dactyl down",
        113 => "Dactyl up",
        114 => "Dactyl left",
        115 => "Unknown",
        116 => "Unknown",
        117 => "AD1000 sign",
        118 => "AD600 sign",
        119 => "AD2300 sign",
        120 => "BC65000000 sign",
        121 => "BC12000 sign",
        122 => "AD1999 sign",
        123 => "? sign",
        124 => "Null",
        125 => "Unknown",
        126 => "Black Omen top",
        127 => "Black Omen bottom",
        128 => "Black Omen top",
        129 => "Black Omen bottom",
        130 => "Unknown",
        131 => "Blackbird down",
        132 => "Blackbird left",
        133 => "Blackbird right",
        134 => "Mt. of Woe",
        135 => "Mt. of Woe (duplicate?)",
        136 => "Mt. of Woe chain link",
        137 => "Twinkling star",
        138 => "600AD robo tractor right",
        139 => "600AD robo tractor left",
        140 => "600AD robo scarecrow",
        141 => "Unknown",
        142 => "Unknown",
        143 => "600AD robo seeding left",
        144 => "600AD robo seeding right",
        145 => "Unknown",
        146 => "Unknown",
        147 => "Unknown",
        148 => "Unknown",
        149 => "Unknown",
        150 => "Unknown",
        151 => "Unknown",
        152 => "Unknown",
        153 => "Unknown",
        154 => "Unknown",
        155 => "Desert vortex",
        156 => "Black Omen reveal lightning",
        157 => "Black Omen reveal lightning",
        158 => "Unknown",
        159 => "Unknown",
        160 => "Unknown",
        161 => "Unknown",
        162 => "PC idle?",
        163 => "PC idle?",
        164 => "PC idle?",
        165 => "Star",
        166 => "Dimensional vortex",
        167 => "Dimensional vortex",
        _ => "Index out of range"
    };
    String::from(description)
}
