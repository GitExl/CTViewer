use bitflags::bitflags;
use crate::tileset::TileSet;
use crate::util::vec2df64::Vec2Df64;

#[derive(Clone)]
pub enum LayerScrollMode {
    Normal,
    IgnoreCamera,
    Parallax,
}

impl LayerScrollMode {
    pub fn to_string(&self) -> &str {
        match self {
            LayerScrollMode::Normal => "Normal",
            LayerScrollMode::IgnoreCamera => "Ignore camera",
            LayerScrollMode::Parallax => "Parallax",
        }
    }
}

bitflags! {
    #[derive(Clone)]
    pub struct ScreenFlags: u32 {
        const SCREEN_L1_MAIN  = 0x01;
        const SCREEN_L2_MAIN  = 0x02;
        const SCREEN_L3_MAIN  = 0x04;
        const SCREEN_SPR_MAIN = 0x08;

        const SCREEN_L1_SUB  = 0x10;
        const SCREEN_L2_SUB  = 0x20;
        const SCREEN_L3_SUB  = 0x40;
        const SCREEN_SPR_SUB = 0x80;
    }
}

bitflags! {
    #[derive(Clone)]
    pub struct EffectFlags: u32 {
        const EFFECT_L1 = 0x01;
        const EFFECT_L2 = 0x02;
        const EFFECT_L3 = 0x04;
        const EFFECT_UNKNOWN = 0x08;
        const EFFECT_SPR = 0x10;
        const EFFECT_DEFAULT_COL = 0x20;
        const EFFECT_HALF_INTENSITY = 0x40;
        const EFFECT_SUBTRACT = 0x80;
    }
}

bitflags! {
    #[derive(Clone, Copy, Default)]
    pub struct MapChipFlags: u32 {
        const PRIORITY = 0x01;
        const FLIP_X = 0x02;
        const FLIP_Y = 0x04;
    }
}

#[derive(Default, Copy, Clone)]
pub struct MapChip {
    pub chip: usize,
    pub flags: MapChipFlags,
    pub palette: usize,
}

#[derive(Default, Copy, Clone)]
pub struct ScrollState {
    pub speed: Vec2Df64,
    pub time: f64,
}

#[derive(Clone)]
pub struct MapLayer {
    pub chip_width: u32,
    pub chip_height: u32,
    pub chips: Vec<MapChip>,

    pub tile_width: u32,
    pub tile_height: u32,
    pub tiles: Vec<usize>,

    pub scroll_mode: LayerScrollMode,
    pub scroll: Vec2Df64,
    pub scroll_last: Vec2Df64,
    pub scroll_lerp: Vec2Df64,

    pub scroll_states: [ScrollState; 2],
}

impl MapLayer {
    pub fn new(chip_width: u32, chip_height: u32) -> MapLayer {
        let len = (chip_width * chip_height) as usize;

        MapLayer {
            chip_width,
            chip_height,
            chips: vec![MapChip::default(); len],

            tile_width: chip_width / 2,
            tile_height: chip_height / 2,
            tiles: vec![0; len / 4],

            scroll_mode: LayerScrollMode::Normal,
            scroll: Vec2Df64::default(),
            scroll_last: Vec2Df64::default(),
            scroll_lerp: Vec2Df64::default(),
            scroll_states: [ScrollState::default(); 2],
        }
    }

    pub fn tick(&mut self, delta: f64) {
        self.scroll_last = self.scroll;
        self.scroll = self.scroll + (self.scroll_states[0].speed + self.scroll_states[1].speed) * delta;

        for state in self.scroll_states.iter_mut() {
            if state.time <= 0.0 {
                continue;
            }

            state.time -= delta;
            if state.time <= 0.0 {
                state.speed.x = 0.0;
                state.speed.y = 0.0;
                state.time = 0.0;
            }
        }

    }

    pub fn lerp(&mut self, lerp: f64) {
        self.scroll_lerp = Vec2Df64::interpolate(self.scroll_last, self.scroll, lerp);
    }

    // Assemble map layer chips from a tileset's tiles.
    pub fn assemble_chips(&mut self, tileset: &TileSet, start_x: u32, start_y: u32, width: u32, height: u32) {

        // Convert each tile into 2x2 chips.
        for y in 0..height {
            for x in 0..width {
                let tile_x = start_x + x;
                let tile_y = start_y + y;
                let tile_index = tile_y * self.tile_width + tile_x;
                if tile_index >= self.tiles.len() as u32 {
                    continue;
                }

                let tile = self.tiles[tile_index as usize];
                if tile >= tileset.tiles.len() {
                    continue;
                }
                let tile_asy = &tileset.tiles[tile];

                let chip_x = tile_x * 2;
                let chip_y = tile_y * 2;
                let chip_index = (chip_y * self.chip_width + chip_x) as usize;

                self.chips[chip_index + 0].clone_from(&tile_asy.corners[0]);
                self.chips[chip_index + 1].clone_from(&tile_asy.corners[1]);
                self.chips[chip_index + self.chip_width as usize + 0].clone_from(&tile_asy.corners[2]);
                self.chips[chip_index + self.chip_width as usize + 1].clone_from(&tile_asy.corners[3]);
            }
        }
    }
}

#[derive(Clone)]
pub struct Map {
    pub index: usize,

    pub screen_flags: ScreenFlags,
    pub effect_flags: EffectFlags,
    pub layer_priorities: [u8; 4],
    pub layers: [MapLayer; 3],
}

impl Map {
    pub fn dump(&self) {
        println!("Map {}", self.index);

        println!("  Layer priorities: {:?}", self.layer_priorities);

        println!("  Translucency");
        println!("              Main   Sub    Enabled");
        println!("    Layer 1   {:<5}  {:<5}  {:<5}", self.screen_flags.contains(ScreenFlags::SCREEN_L1_MAIN), self.screen_flags.contains(ScreenFlags::SCREEN_L1_SUB), self.effect_flags.contains(EffectFlags::EFFECT_L1));
        println!("    Layer 2   {:<5}  {:<5}  {:<5}", self.screen_flags.contains(ScreenFlags::SCREEN_L2_MAIN), self.screen_flags.contains(ScreenFlags::SCREEN_L2_SUB), self.effect_flags.contains(EffectFlags::EFFECT_L2));
        println!("    Layer 3   {:<5}  {:<5}  {:<5}", self.screen_flags.contains(ScreenFlags::SCREEN_L3_MAIN), self.screen_flags.contains(ScreenFlags::SCREEN_L3_SUB), self.effect_flags.contains(EffectFlags::EFFECT_L3));
        println!("    Sprites   {:<5}  {:<5}  {:<5}", self.screen_flags.contains(ScreenFlags::SCREEN_SPR_MAIN), self.screen_flags.contains(ScreenFlags::SCREEN_SPR_SUB), self.effect_flags.contains(EffectFlags::EFFECT_SPR));

        for (i, layer) in self.layers.iter().enumerate() {
            println!("  Layer {}", i + 1);
            println!("    {} x {} tiles", layer.tile_width, layer.tile_height);
            println!("    {} x {} chips", layer.chip_width, layer.chip_height);
            println!("    {} scroll mode", layer.scroll_mode.to_string());
            println!("    Scroll 0 at {} by {} pixels/s", layer.scroll, layer.scroll_states[0].speed);
            println!("    Scroll 1 at {} by {} pixels/s", layer.scroll, layer.scroll_states[1].speed);
        }

        println!();
    }

    pub fn tick(&mut self, delta: f64) {
        for layer in self.layers.iter_mut() {
            layer.tick(delta);
        }
    }

    pub fn lerp(&mut self, lerp: f64) {
        for layer in self.layers.iter_mut() {
            layer.lerp(lerp);
        }
    }
}
