use std::path::Path;
use crate::Context;
use crate::game_palette::GamePalette;
use crate::map::Map;
use crate::tileset::TileSet;
use crate::world::world_exit::{ScriptedWorldExit, WorldExit};
use crate::world::world_map::WorldMap;

pub struct World {
    pub index: usize,

    pub tileset_l12: TileSet,
    pub tileset_l3: TileSet,

    pub palette: GamePalette,
    pub palette_anim: GamePalette,

    pub map: Map,
    pub world_map: WorldMap,

    pub script_data: Vec<u8>,

    pub exits: Vec<WorldExit>,
    pub scripted_exits: Vec<ScriptedWorldExit>,

    pub sprite_graphics: [usize; 4],
}

impl World {

    pub fn dump(&self, ctx: &Context) {
        println!("World {}", self.index);
        println!("  Tileset layer 1/2: {}", self.tileset_l12.index);
        println!("  Tileset 3: {}", self.tileset_l3.index);
        println!("  Palette: {}", self.palette.index);
        println!("  Map: {}", self.map.index);
        println!();

        self.world_map.dump();
        self.map.dump();
        self.tileset_l12.dump();
        self.tileset_l3.dump();
        self.palette.dump();

        for exit in &self.exits {
            exit.dump(ctx);
        }

        for scripted_exit in &self.scripted_exits {
            scripted_exit.dump();
        }

        self.tileset_l12.render_chips_to_surface(&self.tileset_l12.chip_bitmaps).write_to_bmp(Path::new("debug_output/world_chips_l12.bmp"));
        self.tileset_l3.render_chips_to_surface(&self.tileset_l3.chip_bitmaps).write_to_bmp(Path::new("debug_output/world_chips_l3.bmp"));
        self.tileset_l12.render_tiles_to_surface(&self.palette.palette).write_to_bmp(Path::new("debug_output/world_tiles_l12.bmp"));
        self.tileset_l3.render_tiles_to_surface(&self.palette.palette).write_to_bmp(Path::new("debug_output/world_tiles_l3.bmp"));
    }
}
