use std::path::Path;
use crate::actor::{Actor, ActorFlags};
use crate::destination::Destination;
use crate::game_palette::GamePalette;
use crate::l10n::{IndexedType, L10n};
use crate::map::Map;
use crate::map_renderer::MapSprite;
use crate::palette_anim::PaletteAnimSet;
use crate::scene::scene_map::SceneMap;
use crate::scene::scene_script::SceneScript;
use crate::sprites::sprite_manager::SpriteManager;
use crate::tileset::TileSet;

pub struct ScrollMask {
    pub left: isize,
    pub top: isize,
    pub right: isize,
    pub bottom: isize,
}

pub struct SceneExit {
    pub index: usize,

    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,

    pub destination: Destination
}

impl SceneExit {
    pub fn dump(&self, l10n: &L10n) {
        println!("Scene exit {}", self.index);
        println!("  At {} x {}, {} by {}", self.x, self.y, self.width, self.height);
        self.destination.dump(&l10n);

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
    pub script: SceneScript,
    pub actors: Vec<Actor>,
    pub map_sprites: Vec<MapSprite>,
}

impl Scene {
    pub fn init(&mut self) {
        for actor_script in self.script.actors.iter() {
            let mut actor = Actor::spawn();
            actor.script_state = Some(actor_script.get_initial_state());
            self.actors.push(actor);
        }

        // Run first actor script until it yields (first return op).
        for actor in self.actors.iter_mut() {
            if let Some(state) = &mut actor.script_state {
            self.script.run_until_yield(state);
            }
        }
    }

    pub fn dump(&self, l10n: &L10n) {
        println!("Scene {} - {}", self.index, l10n.get_indexed(IndexedType::Scene, self.index));
        println!("  Music {}, map {}",
            self.music_index,
            self.map.index,
        );
        println!("  Script {}",
            self.script_index
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
        self.script.dump();

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

        for actor in self.actors.iter_mut() {
            if actor.flags.contains(ActorFlags::DISABLED) {
                continue;
            }

            actor.tick(delta);
            if let Some(state) = &mut actor.sprite_state {
                sprites.tick_sprite(delta, state);
            }
        }

        self.tileset_l12.tick(delta);
        self.palette_anims.tick(delta, &mut self.palette.palette);
    }

    pub fn lerp(&mut self, lerp: f64) {
        self.map.lerp(lerp);

        for actor in self.actors.iter_mut() {
            if !actor.flags.contains(ActorFlags::RENDERED) {
                continue;
            }

            actor.lerp(lerp);

            // Update map sprite properties from this actor's properties.
            if let Some(state) = &actor.sprite_state {
                actor.update_map_sprite(&mut self.map_sprites[state.map_sprite_index]);
            }
        }
    }

    // todo allocate sprite when needed
    // let mut render_sprite = MapSprite::new();
    // self.map_sprites.push(render_sprite);

    // todo set animation with
    // sprites.set_animation(&mut actor.sprite_state, 23);
}
