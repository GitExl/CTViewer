use std::path::Path;
use crate::actor::{Actor, ActorFlags};
use crate::Context;
use crate::destination::Destination;
use crate::game_palette::GamePalette;
use crate::l10n::IndexedType;
use crate::map::Map;
use crate::palette_anim::PaletteAnimSet;
use crate::scene::scene_map::SceneMap;
use crate::scene_script::scene_script::SceneScript;
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
    pub fn dump(&self, ctx: &Context) {
        println!("Scene exit {}", self.index);
        println!("  At {} x {}, {} by {}", self.x, self.y, self.width, self.height);
        self.destination.dump(ctx);

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
    pub fn dump(&self, ctx: &Context) {
        println!("Treasure '{}'", self.id);
        println!("  At {} x {}", self.tile_x, self.tile_y);
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
}

impl Scene {
    pub fn init(&mut self, ctx: &mut Context) {

        // Create actors and related state.
        for actor_script_index in 0..self.script.actor_scripts.len() {
            let actor = Actor::new(actor_script_index);
            self.script.add_initial_state(actor_script_index);
            let state = ctx.sprites_states.add_state();
            actor.update_sprite_state(state);
            self.actors.push(actor);
        }

        // Run first actor script until it yields (first return op).
        self.script.run_until_yield(ctx, &mut self.actors, &mut self.map, &mut self.scene_map);
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
        println!();

        self.scene_map.dump();
        self.map.dump();
        self.tileset_l12.dump();
        self.tileset_l3.dump();
        self.palette.dump();
        self.palette_anims.dump();
        self.script.dump();

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

    pub fn tick(&mut self, ctx: &mut Context, delta: f64) {
        self.map.tick(delta);

        for (index, actor) in self.actors.iter_mut().enumerate() {
            if actor.flags.contains(ActorFlags::DISABLED) {
                continue;
            }

            actor.tick(delta);
            ctx.sprites_states.tick(&ctx.sprite_assets, delta, index);
        }

        self.script.run(ctx, &mut self.actors, &mut self.map, &mut self.scene_map);

        self.tileset_l12.tick(delta);
        self.palette_anims.tick(delta, &mut self.palette.palette);
    }

    pub fn lerp(&mut self, ctx: &mut Context, lerp: f64) {
        self.map.lerp(lerp);

        for (actor_index, actor) in self.actors.iter_mut().enumerate() {
            if !actor.flags.contains(ActorFlags::RENDERED) {
                continue;
            }

            actor.lerp(lerp);

            let state = ctx.sprites_states.get_state_mut(actor_index);
            actor.update_sprite_state(state);
        }
    }
}
