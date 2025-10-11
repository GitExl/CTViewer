use std::path::Path;
use crate::Context;
use crate::destination::Destination;
use crate::game_palette::GamePalette;
use crate::l10n::IndexedType;
use crate::map::Map;
use crate::palette_anim::PaletteAnimSet;
use crate::scene::scene_map::SceneMap;
use crate::scene_script::scene_script::SceneScript;
use crate::tileset::TileSet;
use crate::util::vec2di32::Vec2Di32;

pub struct ScrollMask {
    pub left: isize,
    pub top: isize,
    pub right: isize,
    pub bottom: isize,
}

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

pub struct SceneTreasure {
    pub id: String,
    pub tile_pos: Vec2Di32,
    pub gold: u32,
    pub item: usize,
}

impl SceneTreasure {
    pub fn dump(&self, ctx: &Context) {
        println!("Treasure '{}'", self.id);
        println!("  At tile {}", self.tile_pos);
        if self.gold > 0 {
            println!("  Contains {} gold", self.gold);
        }
        if self.item > 0 {
            println!("  Contains '{}'", ctx.l10n.get_indexed(IndexedType::Item, self.item));
        }
        println!();
    }
}

pub struct Scene {
    pub index: usize,

    pub music_index: usize,
    pub unknown: u32,

    scene_map: SceneMap,
    map: Map,

    pub scroll_mask: ScrollMask,
    pub tileset_l12: TileSet,
    pub tileset_l3: TileSet,
    pub palette: GamePalette,
    pub palette_anims: PaletteAnimSet,
    pub exits: Vec<SceneExit>,
    pub treasure: Vec<SceneTreasure>,
    pub script: SceneScript,
}

impl Scene {

    pub fn new(
        index: usize,
        music_index: usize,
        unknown: u32,
        map: Map,
        scene_map: SceneMap,
        scroll_mask: ScrollMask,
        tileset_l12: TileSet,
        tileset_l3: TileSet,
        palette: GamePalette,
        palette_anims: PaletteAnimSet,
        exits: Vec<SceneExit>,
        treasure: Vec<SceneTreasure>,
        script: SceneScript,
    ) -> Scene {
        Scene {
            index,
            music_index,
            unknown,
            map,
            scene_map,
            scroll_mask,
            tileset_l12,
            tileset_l3,
            palette,
            palette_anims,
            exits,
            treasure,
            script,
        }
    }

    pub fn get_scene_map(&self) -> &SceneMap {
        &self.scene_map
    }

    pub fn get_map(&self) -> &Map {
        &self.map
    }

    pub fn dump(&self, ctx: &Context) {
        println!("Scene {} - {}", self.index, ctx.l10n.get_indexed(IndexedType::Scene, self.index));
        println!("  Music {}, map {}",
            self.music_index,
            self.map.index,
        );
        println!("  Palette {}",
            self.palette.index,
        );
        println!("  Layer 1 & 2: tileset {}, assembly {}",
            self.tileset_l12.index,
            self.tileset_l12.index_assembly,
        );
        println!("  Layer 3: tileset {}, assembly {}",
            self.tileset_l3.index,
            self.tileset_l3.index_assembly,
        );
        println!("  Scroll mask: {} x {} to {} x {}",
            self.scroll_mask.left,
            self.scroll_mask.top,
            self.scroll_mask.right,
            self.scroll_mask.bottom,
        );
        println!("  Unknown: {}", self.unknown);
        println!();

        self.scene_map.dump();
        self.map.dump();
        self.tileset_l12.dump();
        self.tileset_l3.dump();
        self.palette.dump();
        self.palette_anims.dump();

        for exit in &self.exits {
            exit.dump(ctx);
        }

        for treasure in &self.treasure {
            treasure.dump(ctx);
        }

        self.tileset_l12.render_chips_to_surface(&self.tileset_l12.chip_bitmaps).write_to_bmp(Path::new("debug_output/scene_chips_l12.bmp"));
        self.tileset_l12.render_chips_to_surface(&self.tileset_l12.animated_chip_bitmaps).write_to_bmp(Path::new("debug_output/scene_chips_l12_anim.bmp"));
        self.tileset_l3.render_chips_to_surface(&self.tileset_l3.chip_bitmaps).write_to_bmp(Path::new("debug_output/scene_chips_l3.bmp"));

        self.tileset_l12.render_tiles_to_surface(&self.palette.palette).write_to_bmp(Path::new("debug_output/scene_tiles_l12.bmp"));
        self.tileset_l3.render_tiles_to_surface(&self.palette.palette).write_to_bmp(Path::new("debug_output/scene_tiles_l3.bmp"));
    }
}
