use std::path::Path;
use crate::actor::Actor;
use crate::destination::Destination;
use crate::game_palette::GamePalette;
use crate::l10n::IndexedType;
use crate::l10n::L10n;
use crate::map::Map;
use crate::map_renderer::MapRendererSprite;
use crate::sprites::sprite_manager::SpriteManager;
use crate::tileset::TileSet;
use crate::world::world_map::WorldMap;

pub struct WorldExit {
    pub index: usize,

    pub x: i32,
    pub y: i32,
    pub is_available: bool,
    pub name_index: usize,

    pub destination: Destination,
    pub unknown: u32,
}

impl WorldExit {
    pub fn dump(&self, l10n: &L10n) {
        println!("World exit {} - {}", self.index, l10n.get_indexed(IndexedType::WorldExit, self.name_index));
        println!("  Position: {} x {}", self.x, self.y);
        println!("  Available: {}", self.is_available);
        self.destination.dump(&l10n);

        println!("  Unknown: {}", self.unknown);
        println!();
    }
}

pub struct ScriptedWorldExit {
    pub index: usize,
    pub x: i32,
    pub y: i32,
    pub script_offset_index: usize,
}

impl ScriptedWorldExit {
    pub fn dump(&self) {
        println!("Scripted world exit {}", self.index);
        println!("  Position: {} x {}", self.x, self.y);
        println!("  Script offset index: {}", self.script_offset_index);
        println!();
    }
}

pub struct World {
    pub index: usize,

    pub tileset_l12: TileSet,
    pub tileset_l3: TileSet,

    pub palette: GamePalette,
    pub palette_anim: GamePalette,
    pub palette_anim_index: usize,
    pub palette_anim_timer: f64,

    pub map: Map,
    pub world_map: WorldMap,

    pub script: usize,

    pub exits: Vec<WorldExit>,
    pub scripted_exits: Vec<ScriptedWorldExit>,
    pub script_offsets: Vec<usize>,

    pub actors: Vec<Actor>,
    pub sprite_graphics: [usize; 4],
    pub render_sprites: Vec<MapRendererSprite>,
}

impl World {
    pub fn tick(&mut self, delta: f64, sprites: &SpriteManager) {
        self.map.tick(delta);

        for (index, actor) in self.actors.iter_mut().enumerate() {
            actor.tick(delta);
            sprites.tick_sprite(delta, &mut actor.sprite_state);

            // todo better linkage
            let sprite = &mut self.render_sprites[index];
            sprite.sprite_index = actor.sprite_state.sprite_index;
            sprite.frame = actor.sprite_state.sprite_frame;
            sprite.x = actor.x;
            sprite.y = actor.y;
            sprite.priority = actor.priority;
            sprite.palette_offset = actor.sprite_state.palette_offset;
        }

        // todo clean up
        // todo move into palette anim for tileset?
        self.palette_anim_timer += delta;
        if self.palette_anim_timer >= 1.0 / 6.0 {
            self.palette_anim_timer -= 1.0 / 6.0;

            self.palette_anim_index += 1;
            if self.palette_anim_index >= 4 {
                self.palette_anim_index = 0;
            }

            if self.index == 3 {
                self.palette.palette.colors[48..64].copy_from_slice(&self.palette_anim.palette.colors[self.palette_anim_index * 16..self.palette_anim_index * 16 + 16]);
            } else if self.index == 5 {
                self.palette.palette.colors[112..128].copy_from_slice(&self.palette_anim.palette.colors[self.palette_anim_index * 16..self.palette_anim_index * 16 + 16]);
            } else {
                self.palette.palette.colors[32..48].copy_from_slice(&self.palette_anim.palette.colors[self.palette_anim_index * 16..self.palette_anim_index * 16 + 16]);
            }
        }
    }

    pub fn lerp(&mut self, lerp: f64) {
        self.map.lerp(lerp)
    }

    pub fn add_actor(&mut self, actor: Actor) {
        self.render_sprites.push(MapRendererSprite {
            x: actor.x,
            y: actor.y,
            frame: actor.sprite_state.sprite_frame,
            priority: actor.priority,
            sprite_index: actor.sprite_state.sprite_index,
            palette_offset: actor.sprite_state.palette_offset,
        });
        self.actors.push(actor);
    }

    pub fn dump(&self, l10n: &L10n) {
        println!("World {}", self.index);
        println!("  Tileset layer 1/2: {}", self.tileset_l12.index);
        println!("  Tileset 3: {}", self.tileset_l3.index);
        println!("  Palette: {}", self.palette.index);
        println!("  Map: {}", self.map.index);
        println!("  Script: {}", self.script);
        println!();

        self.world_map.dump();
        self.map.dump();
        self.tileset_l12.dump();
        self.tileset_l3.dump();
        self.palette.dump();

        for exit in &self.exits {
            exit.dump(&l10n);
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
