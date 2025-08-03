use sdl3::event::Event;
use sdl3::keyboard::Keycode;

use crate::actor::Actor;
use crate::camera::Camera;
use crate::filesystem::filesystem::FileSystem;
use crate::gamestate::gamestate::GameStateTrait;
use crate::l10n::{IndexedType, L10n};
use crate::map_renderer::LayerFlags;
use crate::map_renderer::MapRenderer;
use crate::renderer::Renderer;
use crate::sprites::sprite_manager::SpriteManager;
use crate::sprites::sprite_manager::WORLD_SPRITE_INDEX;
use crate::world::world::World;
use crate::world::world_renderer::{WorldDebugLayer, WorldRenderer};

pub struct GameStateWorld<'a> {
    pub world: World,
    pub sprites: SpriteManager<'a>,
    l10n: &'a L10n,

    pub camera: Camera,
    map_renderer: MapRenderer,
    world_renderer: WorldRenderer,

    key_up: bool,
    key_down: bool,
    key_left: bool,
    key_right: bool,
}

impl GameStateWorld<'_> {
    pub fn new<'a>(fs: &'a FileSystem, l10n: &'a L10n, renderer: &mut Renderer, world_index: usize) -> GameStateWorld<'a> {
        let mut sprites = SpriteManager::new(&fs);
        let mut world = fs.read_world(world_index);
        sprites.load_world_sprite(world_index, world.sprite_graphics, &world.palette.palette);
        sprites.load_world_player_sprites([0, 1, 2]);


        // Test sprites.
        let mut actor = Actor::spawn(64.0, 64.0, WORLD_SPRITE_INDEX, 0);
        sprites.set_animation(&mut actor.sprite_state, 0);
        world.add_actor(actor);

        let mut actor = Actor::spawn(128.0, 96.0, WORLD_SPRITE_INDEX, 0);
        sprites.set_animation(&mut actor.sprite_state, 1);
        world.add_actor(actor);

        let mut actor = Actor::spawn(64.0, 128.0, WORLD_SPRITE_INDEX, 0);
        sprites.set_animation(&mut actor.sprite_state, 4);
        world.add_actor(actor);

        let mut actor = Actor::spawn(32.0, 192.0, WORLD_SPRITE_INDEX, 0);
        sprites.set_animation(&mut actor.sprite_state, 6);
        world.add_actor(actor);


        let camera = Camera::new(
            0.0, 0.0,
            renderer.target.width as f64, renderer.target.height as f64,
            0.0, 0.0,
            (world.world_map.width * 8) as f64, (world.world_map.height * 8) as f64,
        );

        let world_renderer = WorldRenderer::new();
        let mut map_renderer = MapRenderer::new(renderer.target.width, renderer.target.height);
        map_renderer.setup_for_map(&mut world.map);

        GameStateWorld {
            world,
            sprites,
            l10n,

            camera,
            world_renderer,
            map_renderer,

            key_down: false,
            key_left: false,
            key_right: false,
            key_up: false,
        }
    }
}

impl GameStateTrait for GameStateWorld<'_> {
    fn tick(&mut self, delta: f64) {
        self.camera.tick(delta);
        if self.key_up {
            self.camera.y -= 300.0 * delta;
        }
        else if self.key_down {
            self.camera.y += 300.0 * delta;
        }
        if self.key_left {
            self.camera.x -= 300.0 * delta;
        }
        else if self.key_right {
            self.camera.x += 300.0 * delta;
        }
        self.camera.wrap();

        self.world.tick(delta, &self.sprites);
    }

    fn render(&mut self, lerp: f64, renderer: &mut Renderer) {
        self.camera.lerp(lerp);
        self.map_renderer.render(lerp, &self.camera, &mut renderer.target, &self.world.map, &self.world.tileset_l12, &self.world.tileset_l3, &self.world.palette, &self.world.render_sprites, &self.sprites);
        self.world_renderer.render(lerp, &self.camera, &mut self.world, &mut renderer.target);
    }

    fn get_title(&self, l10n: &L10n) -> String {
        l10n.get_indexed(IndexedType::World, self.world.index)
    }

    fn event(&mut self, event: &Event) {
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

            _ => {},
        }
    }

    fn mouse_motion(&mut self, _x: i32, _y: i32) {

    }

    fn dump(&mut self) {
        self.world.dump(self.l10n);

        self.sprites.dump_world_sprite_graphics();

        // for set in self.sprites.anim_sets.iter() {
        //     set.dump();
        // }
    }
}
