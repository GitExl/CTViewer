use sdl3::event::Event;
use sdl3::keyboard::Keycode;

use crate::actor::Actor;
use crate::camera::Camera;
use crate::filesystem::filesystem::FileSystem;
use crate::gamestate::gamestate::GameStateTrait;
use crate::l10n::{IndexedType, L10n};
use crate::map_renderer::LayerFlags;
use crate::map_renderer::MapRenderer;
use crate::renderer::{Renderer, TextFlags, TextRenderable};
use crate::scene::scene::Scene;
use crate::scene::scene_renderer::{SceneDebugLayer, SceneRenderer};
use crate::software_renderer::text::TextDrawFlags;
use crate::sprites::sprite_manager::SpriteManager;

pub struct GameStateScene<'a> {
    pub scene: Scene,
    pub sprites: SpriteManager<'a>,
    l10n: &'a L10n,

    pub camera: Camera,
    map_renderer: MapRenderer,
    scene_renderer: SceneRenderer,

    key_up: bool,
    key_down: bool,
    key_left: bool,
    key_right: bool,

    debug_text: Option<TextRenderable>,
    debug_text_x: i32,
    debug_text_y: i32,
}

impl GameStateScene<'_> {
    pub fn new<'a>(fs: &'a FileSystem, l10n: &'a L10n, renderer: &mut Renderer, scene_index: usize) -> GameStateScene<'a> {
        let mut sprites = SpriteManager::new(&fs);
        let mut scene = fs.read_scene(scene_index);

        // Test sprites.
        sprites.load(0);
        sprites.load(2);
        sprites.load(6);
        sprites.load(16);
        sprites.load(128);
        sprites.load(402);

        let mut actor = Actor::spawn(70.0, 100.0, 0, 2);
        sprites.set_animation(&mut actor.sprite_state, 23);
        scene.add_actor(actor);

        let mut actor = Actor::spawn(50.0, 190.0, 2, 3);
        sprites.set_animation(&mut actor.sprite_state, 1);
        scene.add_actor(actor);

        let mut actor = Actor::spawn(170.0, 170.0, 6, 1);
        sprites.set_animation(&mut actor.sprite_state, 6);
        scene.add_actor(actor);

        let mut actor = Actor::spawn(230.0, 150.0, 16, 0);
        sprites.set_animation(&mut actor.sprite_state, 1);
        scene.add_actor(actor);

        let mut actor = Actor::spawn(110.0, 70.0, 128, 0);
        sprites.set_animation(&mut actor.sprite_state, 0);
        scene.add_actor(actor);

        let mut actor = Actor::spawn(120.0, 192.0, 402, 1);
        sprites.set_animation(&mut actor.sprite_state, 6);
        scene.add_actor(actor);


        let camera = Camera::new(
            scene.scroll_mask.left as f64, scene.scroll_mask.top as f64,
            renderer.target.width as f64, renderer.target.height as f64 - 12.0,
            scene.scroll_mask.left as f64, scene.scroll_mask.top as f64,
            scene.scroll_mask.right as f64, scene.scroll_mask.bottom as f64,
        );
        renderer.target.clip.bottom = renderer.target.height as i32 - 12;

        let scene_renderer = SceneRenderer::new();
        let mut map_renderer = MapRenderer::new(renderer.target.width, renderer.target.height - 12);
        map_renderer.setup_for_map(&mut scene.map);

        GameStateScene {
            scene,
            sprites,
            l10n,

            camera,
            scene_renderer,
            map_renderer,

            key_down: false,
            key_left: false,
            key_right: false,
            key_up: false,

            debug_text: None,
            debug_text_x: 0,
            debug_text_y: 0,
        }
    }
}

impl GameStateTrait for GameStateScene<'_> {
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
        self.camera.clamp();

        self.scene.tick(delta, &self.sprites);
    }

    fn render(&mut self, lerp: f64, renderer: &mut Renderer) {
        self.camera.lerp(lerp);
        self.map_renderer.render(lerp, &self.camera, &mut renderer.target, &self.scene.map, &self.scene.tileset_l12, &self.scene.tileset_l3, &self.scene.palette, &self.scene.render_sprites, &self.sprites);
        self.scene_renderer.render(lerp, &self.camera, &mut self.scene, &mut renderer.target);

        if self.debug_text.is_some() {
            renderer.render_text(&mut self.debug_text.as_mut().unwrap(), self.debug_text_x, self.debug_text_y, TextFlags::AlignHCenter | TextFlags::AlignVEnd | TextFlags::ClampToTarget);
        }
    }

    fn get_title(&self, l10n: &L10n) -> String {
        l10n.get_indexed(IndexedType::Scene, self.scene.index)
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
                        self.scene_renderer.debug_palette = !self.scene_renderer.debug_palette;
                        println!("Render palette.");
                    }

                    Some(Keycode::Z) => {
                        self.scene_renderer.debug_layer = SceneDebugLayer::Disabled;
                        println!("Debug layer disabled.");
                    },
                    Some(Keycode::X) => {
                        self.scene_renderer.debug_layer = SceneDebugLayer::PcCollision;
                        println!("Debug layer for player collision.");
                    },
                    Some(Keycode::C) => {
                        self.scene_renderer.debug_layer = SceneDebugLayer::NpcCollision;
                        println!("Debug layer for NPC collision.");
                    },
                    Some(Keycode::V) => {
                        self.scene_renderer.debug_layer = SceneDebugLayer::ZPlane;
                        println!("Debug layer for Z plane.");
                    },
                    Some(Keycode::B) => {
                        self.scene_renderer.debug_layer = SceneDebugLayer::ZPlaneFlags;
                        println!("Debug layer for Z plane flags.");
                    },
                    Some(Keycode::N) => {
                        self.scene_renderer.debug_layer = SceneDebugLayer::Movement;
                        println!("Debug layer for movement.");
                    },
                    Some(Keycode::M) => {
                        self.scene_renderer.debug_layer = SceneDebugLayer::DoorTrigger;
                        println!("Debug layer for door triggers.");
                    },
                    Some(Keycode::Comma) => {
                        self.scene_renderer.debug_layer = SceneDebugLayer::SpritePriority;
                        println!("Debug layer for sprite priority data.");
                    },
                    Some(Keycode::Period) => {
                        self.scene_renderer.debug_layer = SceneDebugLayer::Exits;
                        println!("Debug layer for exits.");
                    },
                    Some(Keycode::Slash) => {
                        self.scene_renderer.debug_layer = SceneDebugLayer::Treasure;
                        println!("Debug layer for treasure.");
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

    fn mouse_motion(&mut self, x: i32, y: i32) {
        let map_x = (x as f64 + self.camera.x) as i32;
        let map_y = (y as f64 + self.camera.y) as i32;

        // Output exit or treasure data at mouse position.
        let mut found = false;
        for exit in self.scene.exits.iter() {
            if map_x < exit.x || map_x >= exit.x + exit.width || map_y < exit.y || map_y >= exit.y + exit.height {
                continue;
            }

            let text = format!("To 0x{:03X} '{}'", exit.destination_index, self.l10n.get_indexed(IndexedType::Scene, exit.destination_index));
            self.debug_text = Some(TextRenderable::new(text, [223, 223, 223, 255], TextDrawFlags::SHADOW, 128));
            found = true;
            break;
        }

        if !found {
            for treasure in self.scene.treasure.iter() {
                if map_x < treasure.tile_x as i32 * 16 || map_x >= treasure.tile_x as i32 * 16 + 16 || map_y < treasure.tile_y as i32 * 16 || map_y >= treasure.tile_y as i32 * 16 + 16 {
                    continue;
                }

                let text = if treasure.gold > 0 {
                    format!("{} gold", treasure.gold)
                } else if treasure.item > 0 {
                    format!("0x{:03X} '{}'", treasure.item, self.l10n.get_indexed(IndexedType::Item, treasure.item))
                } else {
                    "Empty".to_string()
                };
                self.debug_text = Some(TextRenderable::new(text, [223, 223, 223, 255], TextDrawFlags::SHADOW, 192));
                found = true;
                break;
            }
        }

        if !found {
            self.debug_text = None;
        } else {
            self.debug_text_x = x;
            self.debug_text_y = y;
        }
    }

    fn dump(&mut self) {
        self.scene.dump(self.l10n);

        // for (_, set) in sprites.anim_sets.iter() {
        //     set.dump();
        // }
    }
}
