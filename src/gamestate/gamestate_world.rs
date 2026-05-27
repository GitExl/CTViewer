use std::collections::HashMap;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::mouse::MouseButton;
use crate::camera::Camera;
use crate::{Context, GameEvent};
use crate::actor::{Actor, ActorClass, ActorFlags};
use crate::character::CharacterId;
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
use crate::sprites::sprite_assets::WORLD_SPRITE_INDEX;
use crate::util::vec2df64::Vec2Df64;
use crate::util::vec2di32::Vec2Di32;
use crate::world::world::World;
use crate::world::world_map::WorldMap;
use crate::world::world_renderer::{WorldDebugLayer, WorldRenderer};

/// Mutable state for a world.
pub struct WorldState {
    pub actors: Vec<Actor>,
    pub player_actors: HashMap<CharacterId, usize>,
    pub next_destination: NextDestination,
    pub camera: Camera,
    pub world_map: WorldMap,
    pub map: Map,
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
        ctx.sprites_states.clear();

        let mut world = ctx.fs.read_world(world_index);
        ctx.sprite_assets.load_world_sprite_asset(&ctx.fs, world_index, world.sprite_graphics, &world.palette.palette);
        ctx.sprite_assets.load_world_player_sprites_asset(&ctx.fs, [0, 1, 2]);


        let mut actors = Vec::new();

        // Test sprites.
        let mut actor = Actor::new(0);
        let state = ctx.sprites_states.add_state();
        actor.pos = Vec2Df64::new(128.0, 128.0);
        actor.flags.remove(ActorFlags::DEAD);
        actor.class = ActorClass::NPC;
        state.sprite_index = WORLD_SPRITE_INDEX;
        state.anim_index = 1;
        state.palette = world.palette.palette.clone();
        actors.push(actor);

        let mut actor = Actor::new(1);
        actor.pos = Vec2Df64::new(192.0, 32.0);
        actor.flags.remove(ActorFlags::DEAD);
        actor.class = ActorClass::NPC;
        let state = ctx.sprites_states.add_state();
        state.sprite_index = WORLD_SPRITE_INDEX;
        state.anim_index = 34;
        state.palette = world.palette.palette.clone();
        actors.push(actor);

        let mut actor = Actor::new(2);
        actor.pos = Vec2Df64::new(32.0, 192.0);
        actor.flags.remove(ActorFlags::DEAD);
        actor.class = ActorClass::NPC;
        let state = ctx.sprites_states.add_state();
        state.sprite_index = WORLD_SPRITE_INDEX;
        state.anim_index = 81;
        state.palette = world.palette.palette.clone();
        actors.push(actor);

        let mut actor = Actor::new(3);
        actor.pos = Vec2Df64::new(32.0, 256.0);
        actor.flags.remove(ActorFlags::DEAD);
        actor.class = ActorClass::NPC;
        let state = ctx.sprites_states.add_state();
        state.sprite_index = WORLD_SPRITE_INDEX;
        state.anim_index = 155;
        state.palette = world.palette.palette.clone();
        actors.push(actor);


        // Create new shared world state.
        let mut state = WorldState {
            next_destination: NextDestination::new(),
            camera: Camera::new(
                0.0, 0.0,
                ctx.render.target.width as f64, ctx.render.target.height as f64,
                0.0, 0.0,
                (world.world_map.width * 8) as f64, (world.world_map.height * 8) as f64,
            ),
            actors,
            player_actors: HashMap::new(),
            world_map: world.world_map.clone(),
            map: world.map.clone(),
        };


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
        self.world.tick(ctx, delta);

        for (index, actor) in self.state.actors.iter_mut().enumerate() {
            // /actor.tick(delta, &self.state.scene_map);
            let state = ctx.sprites_states.get_state_mut(index);
            actor.update_sprite_state(state);
            ctx.sprites_states.tick(&ctx.sprite_assets, index, actor);
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

        if self.next_game_event.is_some() {
            let event = self.next_game_event;
            self.next_game_event = None;
            return event;
        }

        None
    }

    fn render(&mut self, ctx: &mut Context, lerp: f64) {
        self.state.camera.lerp(lerp);

        self.map_renderer.render(
            lerp,
            &self.state.camera,
            &mut ctx.render.target,
            &self.world.map,
            &self.world.tileset_l12,
            &self.world.tileset_l3,
            &self.world.palette,
            &ctx.sprites_states,
            &ctx.sprite_assets,
        );
        self.world_renderer.render(
            lerp,
            &self.state.camera,
            &mut self.world,
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

        ctx.sprite_assets.dump_world_sprite_graphics();

        // for set in self.sprites.anim_sets.iter() {
        //     set.dump();
        // }
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
