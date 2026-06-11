use std::collections::HashMap;
use std::io::Cursor;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::mouse::MouseButton;
use crate::camera::Camera;
use crate::{Context, GameEvent};
use crate::character::CharacterId;
use crate::game_palette::GamePalette;
use crate::gamestate::gamestate::GameStateTrait;
use crate::l10n::IndexedType;
use crate::map::Map;
use crate::map_renderer::LayerFlags;
use crate::map_renderer::MapRenderer;
use crate::next_destination::NextDestination;
use crate::renderer::{TextFlags, TextFont, TextRenderable};
use crate::software_renderer::blit::SurfaceBlendOps;
use crate::util::rect::Rect;
use crate::software_renderer::text::TextDrawFlags;
use crate::sprites::sprite_renderer::SpritePriority;
use crate::sprites::sprite_state::SpriteStateFlags;
use crate::tileset::TileSet;
use crate::util::vec2df64::Vec2Df64;
use crate::util::vec2di32::Vec2Di32;
use crate::world::world::World;
use crate::world::world_exit::{ScriptedWorldExit, WorldExit};
use crate::world::world_map::WorldMap;
use crate::world::world_renderer::{WorldDebugLayer, WorldRenderer};
use crate::world::world_sprites::WorldSprites;
use crate::world_script::task_dispatch::WorldActorTask;
use crate::world_script::world_actor::WorldActor;
use crate::world_script::world_animation_script::WorldAnimationScript;
use crate::world_script::world_script::{world_script_disassemble, world_script_initialize, world_script_run};

/// Mutable state for a world.
pub struct WorldState {
    pub player_actors: HashMap<CharacterId, usize>,
    pub next_destination: NextDestination,
    pub camera: Camera,
    pub world_map: WorldMap,
    pub map: Map,
    pub animations: WorldAnimationScript,
    pub sprites: WorldSprites,
    pub tileset_l12: TileSet,
    pub tileset_l3: TileSet,
    pub palette: GamePalette,
    pub palette_animation: GamePalette,
    pub exits: Vec<WorldExit>,
    pub scripted_exits: Vec<ScriptedWorldExit>,
    pub script_data: Cursor<Vec<u8>>,
    pub actors: [WorldActor; 64],
}

pub struct GameStateWorld {
    world: World,
    state: WorldState,

    map_renderer: MapRenderer,
    world_renderer: WorldRenderer,

    key_up: bool,
    key_down: bool,
    key_left: bool,
    key_right: bool,

    mouse_pos: Vec2Di32,

    debug_text: Option<TextRenderable>,
    debug_text_x: i32,
    debug_text_y: i32,
    debug_box: Option<Rect>,

    next_game_event: Option<GameEvent>,
}

impl GameStateWorld {
    pub fn new(ctx: &mut Context, world_index: usize, pos: Vec2Df64, fade_in: bool) -> GameStateWorld {
        ctx.memory.put_u8(0x7E1B59, 0);
        ctx.memory.put_u8(0x7E1BF7, 0);

        // Copy storyflags from scene memory to world memory.
        let story_flags = ctx.memory.get_bytes(0x7F01F0, 0x0F);
        ctx.memory.put_bytes(0x7E1BA7, &story_flags);
        ctx.memory.put_u8(0x7E1BA6, ctx.memory.get_u8(0x7F0000));

        let mut world = ctx.fs.read_world(world_index);

        let mut sprites = WorldSprites::new(ctx, world_index, world.sprite_graphics);
        sprites.load_player_sprites(ctx, [0, 1, 2]);

        // Create new shared world state.
        let mut state = WorldState {
            next_destination: NextDestination::new(),
            camera: Camera::new(
                0.0, 0.0,
                ctx.render.target.width as f64, ctx.render.target.height as f64,
                0.0, 0.0,
                (world.world_map.width * 8) as f64, (world.world_map.height * 8) as f64,
            ),
            actors: std::array::from_fn::<_, 64, _>(|_| WorldActor::new()),
            player_actors: HashMap::new(),
            world_map: world.world_map.clone(),
            map: world.map.clone(),
            tileset_l12: world.tileset_l12.clone(),
            tileset_l3: world.tileset_l3.clone(),
            palette: world.palette.clone(),
            palette_animation: world.palette_anim.clone(),
            exits: world.exits.clone(),
            scripted_exits: world.scripted_exits.clone(),
            animations: ctx.fs.read_world_animation_script(),
            sprites,
            script_data: Cursor::new(world.script_data.clone()),
        };

        world_script_initialize(&mut state);

        let world_renderer = WorldRenderer::new();
        let mut map_renderer = MapRenderer::new(ctx.render.target.width, ctx.render.target.height);
        map_renderer.setup_for_map(&mut world.map);

        state.camera.center_to(pos);
        if fade_in {
            ctx.screen_fade.start(1.0, 2);
        }

        println!("Entering world {}: {}", world.index, ctx.l10n.get_indexed(IndexedType::World, world.index));

        GameStateWorld {
            world,
            state,

            world_renderer,
            map_renderer,

            key_down: false,
            key_left: false,
            key_right: false,
            key_up: false,

            mouse_pos: Vec2Di32::default(),

            debug_text: None,
            debug_text_x: 0,
            debug_text_y: 0,
            debug_box: None,

            next_game_event: None,
        }
    }
}

impl GameStateTrait for GameStateWorld {

    fn tick(&mut self, ctx: &mut Context, delta: f64) -> Option<GameEvent> {
        self.state.map.tick(delta);

        world_script_run(ctx, &mut self.state);

        ctx.sprite_states.clear();
        for actor in self.state.actors.iter() {
            if actor.sprite_assembly_key == 0 || matches!(actor.task, WorldActorTask::None) {
                continue;
            }

            let state = ctx.sprite_states.add_state();
            state.pos.x = actor.x;
            state.pos.y = actor.y;
            state.palette = self.world.palette.palette.clone();
            state.palette_offset = 128 + ((actor.palette_priority >> 1) & 0x07) as usize * 16;
            state.assembly_key = actor.sprite_assembly_key;

            state.flags = SpriteStateFlags::empty();
            state.flags.insert(SpriteStateFlags::ENABLED);
            if actor.palette_priority & 0x01 != 0 {
                state.flags.insert(SpriteStateFlags::CAMERA_RELATIVE);
            }

            let priority = match actor.palette_priority & 0x30 {
                0x30 => SpritePriority::AboveAll,
                0x20 => SpritePriority::BelowL2AboveL1,
                0x10 => SpritePriority::BelowL1L2,
                _ => SpritePriority::BelowAll,
            };
            state.priority_top = priority;
            state.priority_bottom = priority;

            state.bitmap_key = self.state.sprites.get_bitmap_key();
        }

        self.state.camera.tick(delta);
        if self.key_up {
            self.state.camera.pos.y -= 300.0 * delta;
        }
        else if self.key_down {
            self.state.camera.pos.y += 300.0 * delta;
        }
        if self.key_left {
            self.state.camera.pos.x -= 300.0 * delta;
        }
        else if self.key_right {
            self.state.camera.pos.x += 300.0 * delta;
        }
        self.state.camera.wrap();

        if let Some(next_destination) = self.state.next_destination.destination {
            if !ctx.screen_fade.is_active() {
                self.next_game_event = Some(GameEvent::GotoDestination { destination: next_destination, fade_in: self.state.next_destination.fade_in });
                self.state.next_destination.clear();
            }
        }

        if let Some(next_game_event) = self.next_game_event {
            match next_game_event {
                // Copy storyflags back to scene memory.
                GameEvent::GotoDestination { .. } => {

                    let story_flags = ctx.memory.get_bytes(0x7E1BA7, 0x0F);
                    ctx.memory.put_bytes(0x7F01F0, &story_flags);
                }
            }

            self.next_game_event = None;
            return Some(next_game_event);
        }

        None
    }

    fn render(&mut self, ctx: &mut Context, lerp: f64) {
        self.state.map.lerp(lerp);
        self.state.camera.lerp(lerp);

        self.map_renderer.render(
            lerp,
            &self.state.camera,
            &mut ctx.render.target,
            &self.state.map,
            &self.state.tileset_l12,
            &self.state.tileset_l3,
            &self.state.palette,
            &ctx.sprite_states,
            &ctx.assets,
        );
        self.world_renderer.render(
            lerp,
            &self.state,
            &mut ctx.render.target,
        );

        if self.debug_text.is_some() {
            ctx.render.render_text(
                &mut self.debug_text.as_mut().unwrap(),
                self.debug_text_x - self.state.camera.pos_lerp.x as i32, self.debug_text_y - self.state.camera.pos_lerp.y as i32,
                TextFlags::AlignHCenter | TextFlags::AlignVEnd | TextFlags::ClampToTarget,
            );
        }
        if self.debug_box.is_some() {
            ctx.render.render_box_filled(
                self.debug_box.as_mut().unwrap().moved_by(-self.state.camera.pos_lerp.x as i32, -self.state.camera.pos_lerp.y as i32),
                [255, 255, 255, 127],
                SurfaceBlendOps::Blend,
            );
        }
    }

    fn get_title(&self, ctx: &Context) -> String {
        format!("{} - {}", self.world.index, ctx.l10n.get_indexed(IndexedType::World, self.world.index))
    }

    fn event(&mut self, ctx: &mut Context, event: &Event) {
        match event {
            Event::KeyDown { keycode, .. } => {
                match keycode {
                    Some(Keycode::W) => self.key_up = true,
                    Some(Keycode::A) => self.key_left = true,
                    Some(Keycode::S) => self.key_down = true,
                    Some(Keycode::D) => self.key_right = true,

                    Some(Keycode::_1) => {
                        self.map_renderer.layer_enabled.toggle(LayerFlags::Layer1);
                        println!("Render layer 1: {}.", self.map_renderer.layer_enabled.contains(LayerFlags::Layer1));
                    },
                    Some(Keycode::_2) => {
                        self.map_renderer.layer_enabled.toggle(LayerFlags::Layer2);
                        println!("Render layer 2: {}.", self.map_renderer.layer_enabled.contains(LayerFlags::Layer2));
                    },
                    Some(Keycode::_3) => {
                        self.map_renderer.layer_enabled.toggle(LayerFlags::Layer3);
                        println!("Render layer 3: {}.", self.map_renderer.layer_enabled.contains(LayerFlags::Layer3));
                    },
                    Some(Keycode::_4) => {
                        self.map_renderer.layer_enabled.toggle(LayerFlags::Sprites);
                        println!("Render sprites: {}.", self.map_renderer.layer_enabled.contains(LayerFlags::Sprites));
                    },
                    Some(Keycode::_5) => {
                        self.world_renderer.debug_palette = !self.world_renderer.debug_palette;
                        println!("Render palette.");
                    }

                    Some(Keycode::Z) => {
                        self.world_renderer.debug_layer = WorldDebugLayer::Disabled;
                        println!("Debug layer disabled.");
                    },
                    Some(Keycode::X) => {
                        self.world_renderer.debug_layer = WorldDebugLayer::Solidity;
                        println!("Debug layer for solidity.");
                    },
                    Some(Keycode::C) => {
                        self.world_renderer.debug_layer = WorldDebugLayer::Exits;
                        println!("Debug layer for exits.");
                    },
                    Some(Keycode::V) => {
                        self.world_renderer.debug_layer = WorldDebugLayer::Music;
                        println!("Debug layer for music transitions.");
                    },
                    _ => {},
                }
            },

            Event::KeyUp { keycode, .. } => {
                match keycode {
                    Some(Keycode::W) => self.key_up = false,
                    Some(Keycode::A) => self.key_left = false,
                    Some(Keycode::S) => self.key_down = false,
                    Some(Keycode::D) => self.key_right = false,
                    _ => {},
                }
            },

            Event::MouseButtonDown { mouse_btn, .. } => {
                if *mouse_btn == MouseButton::Left {
                    let index = self.get_exit_at(self.mouse_pos);
                    if index.is_some() {
                        let exit = &self.world.exits[index.unwrap()];
                        self.state.next_destination.set(exit.destination, true);
                        ctx.screen_fade.start(0.0, 2);
                    }
                }
            },

            _ => {},
        }
    }

    fn mouse_motion(&mut self, ctx: &Context, x: i32, y: i32) {
        self.mouse_pos = Vec2Di32::new(
            (x as f64 + self.state.camera.pos.x) as i32,
            (y as f64 + self.state.camera.pos.y) as i32,
        );

        // Output exit or treasure data at mouse position.
        let index = self.get_exit_at(self.mouse_pos);
        if index.is_some() {
            let exit = &self.world.exits[index.unwrap()];
            let text = exit.destination.title(&ctx);

            self.debug_text = Some(TextRenderable::new(
                text,
                TextFont::Regular,
                [231, 231, 231, 255],
                TextDrawFlags::SHADOW,
                0,
            ));
            self.debug_text_x = exit.pos.x + 8;
            self.debug_text_y = exit.pos.y;
            self.debug_box = Some(Rect::new(
                exit.pos.x, exit.pos.y,
                exit.pos.x + 16, exit.pos.y + 16,
            ));
        }

        if index.is_none() {
            self.debug_text = None;
            self.debug_text_x = 0;
            self.debug_text_y = 0;
            self.debug_box = None;
        }
    }

    fn dump(&mut self, ctx: &Context) {
        self.world.dump(ctx);

        self.state.sprites.dump(ctx, &self.world.palette.palette);
        self.state.animations.disassemble();
        world_script_disassemble(&ctx, self.state.script_data.get_ref());
    }
}

impl GameStateWorld {
    fn get_exit_at(&self, pos: Vec2Di32) -> Option<usize> {
        for (index, exit) in self.world.exits.iter().enumerate() {
            if pos.x < exit.pos.x || pos.x >= exit.pos.x + 16 ||
               pos.y < exit.pos.y || pos.y >= exit.pos.y + 16 {
                continue;
            }
            return Some(index);
        }

        None
    }
}
