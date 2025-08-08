use std::path::Path;
use crate::actor::Actor;
use crate::game_palette::GamePalette;
use crate::l10n::{IndexedType, L10n};
use crate::map::Map;
use crate::map_renderer::MapRendererSprite;
use crate::palette_anim::PaletteAnimSet;
use crate::scene::scene_map::SceneMap;
use crate::sprites::sprite_manager::SpriteManager;
use crate::tileset::TileSet;

pub struct ScrollMask {
    pub left: isize,
    pub top: isize,
    pub right: isize,
    pub bottom: isize,
}

#[derive(Debug)]
pub enum SceneExitFacing {
    Up,
    Down,
    Left,
    Right,
}

pub struct SceneExit {
    pub index: usize,

    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,

    pub destination_index: usize,
    pub destination_x: i32,
    pub destination_y: i32,
    pub facing: SceneExitFacing,
}

impl SceneExit {
    pub fn dump(&self, l10n: &L10n) {
        println!("Scene exit {}", self.index);
        println!("  At {} x {}, {} by {}", self.x, self.y, self.width, self.height);
        println!("  To 0x{:03X} - {}", self.destination_index, l10n.get_indexed(IndexedType::Scene, self.destination_index));
        println!("  At {} x {} facing {:?}", self.destination_x, self.destination_y, self.facing);
        println!();
    }
}

pub struct SceneTreasure {
    pub id: String,
    pub tile_x: i32,
    pub tile_y: i32,
    pub gold: u32,
    pub item: usize,
}

impl SceneTreasure {
    pub fn dump(&self, l10n: &L10n) {
        println!("Treasure '{}'", self.id);
        println!("  At {} x {}", self.tile_x, self.tile_y);
        if self.gold > 0 {
            println!("  Contains {} gold", self.gold);
        }
        if self.item > 0 {
            println!("  Contains '{}'", l10n.get_indexed(IndexedType::Item, self.item));
        }
        println!();
    }
}

pub struct Scene {
    pub index: usize,

    pub music_index: usize,
    pub script_index: usize,

    pub scene_map: SceneMap,
    pub map: Map,
    pub scroll_mask: ScrollMask,
    pub tileset_l12: TileSet,
    pub tileset_l3: TileSet,
    pub palette: GamePalette,
    pub palette_anims: PaletteAnimSet,
    pub exits: Vec<SceneExit>,
    pub treasure: Vec<SceneTreasure>,
    pub actors: Vec<Actor>,
    pub render_sprites: Vec<MapRendererSprite>,
}

impl Scene {
    pub fn dump(&self, l10n: &L10n) {
        println!("Scene {} - {}", self.index, l10n.get_indexed(IndexedType::Scene, self.index));
        println!("  Music 0x{:02X}, map {}, script 0x{:02X}",
            self.music_index,
            self.map.index,
            self.script_index,
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
        println!();

        self.scene_map.dump();
        self.map.dump();
        self.tileset_l12.dump();
        self.tileset_l3.dump();
        self.palette.dump();
        self.palette_anims.dump();

        for exit in &self.exits {
            exit.dump(l10n);
        }

        for treasure in &self.treasure {
            treasure.dump(l10n);
        }

        self.tileset_l12.render_chips_to_surface(&self.tileset_l12.chip_bitmaps).write_to_bmp(Path::new("debug_output/scene_chips_l12.bmp"));
        self.tileset_l12.render_chips_to_surface(&self.tileset_l12.animated_chip_bitmaps).write_to_bmp(Path::new("debug_output/scene_chips_l12_anim.bmp"));
        self.tileset_l3.render_chips_to_surface(&self.tileset_l3.chip_bitmaps).write_to_bmp(Path::new("debug_output/scene_chips_l3.bmp"));

        self.tileset_l12.render_tiles_to_surface(&self.palette.palette).write_to_bmp(Path::new("debug_output/scene_tiles_l12.bmp"));
        self.tileset_l3.render_tiles_to_surface(&self.palette.palette).write_to_bmp(Path::new("debug_output/scene_tiles_l3.bmp"));
    }

    pub fn tick(&mut self, delta: f64, sprites: &SpriteManager) {
        self.map.tick(delta);

        for (index, actor) in self.actors.iter_mut().enumerate() {
            actor.tick(delta);
            sprites.tick_sprite(delta, &mut actor.sprite_state);
            update_render_sprite(&actor, &mut self.render_sprites[index]);
        }

        self.tileset_l12.tick(delta);
        self.palette_anims.tick(delta, &mut self.palette.palette);
    }

    pub fn lerp(&mut self, lerp: f64) {
        self.map.lerp(lerp);
    }

    pub fn add_actor(&mut self, actor: Actor) {
        let mut render_sprite = MapRendererSprite::new();
        update_render_sprite(&actor, &mut render_sprite);
        self.render_sprites.push(render_sprite);
        
        self.actors.push(actor);
    }
}

fn update_render_sprite(actor: &Actor, sprite: &mut MapRendererSprite) {
    sprite.sprite_index = actor.sprite_state.sprite_index;
    sprite.frame = actor.sprite_state.sprite_frame;
    sprite.x = actor.x;
    sprite.y = actor.y;
    sprite.priority = actor.priority;
    sprite.palette_offset = actor.sprite_state.palette_offset;
}
