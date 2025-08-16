use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::mouse::MouseButton;
use crate::actor::Actor;
use crate::camera::Camera;
use crate::filesystem::filesystem::FileSystem;
use crate::GameEvent;
use crate::gamestate::gamestate::GameStateTrait;
use crate::l10n::{IndexedType, L10n};
use crate::map_renderer::LayerFlags;
use crate::map_renderer::MapRenderer;
use crate::renderer::{Renderer, TextFlags, TextRenderable};
use crate::scene::scene::Scene;
use crate::scene::scene_renderer::{SceneDebugLayer, SceneRenderer};
use crate::software_renderer::blit::SurfaceBlendOps;
use crate::software_renderer::clip::Rect;
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

    mouse_x: i32,
    mouse_y: i32,

    debug_text: Option<TextRenderable>,
    debug_text_x: i32,
    debug_text_y: i32,
    debug_box: Option<Rect>,

    next_game_event: Option<GameEvent>,
}

impl GameStateScene<'_> {
    pub fn new<'a>(fs: &'a FileSystem, l10n: &'a L10n, renderer: &mut Renderer, scene_index: usize, x: i32, y: i32) -> GameStateScene<'a> {
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


        let mut camera = Camera::new(
            scene.scroll_mask.left as f64, scene.scroll_mask.top as f64,
            renderer.target.width as f64, renderer.target.height as f64 - 12.0,
            scene.scroll_mask.left as f64, scene.scroll_mask.top as f64,
            scene.scroll_mask.right as f64, scene.scroll_mask.bottom as f64,
        );

        renderer.target.clip.bottom = renderer.target.height as i32 - 12;

        let scene_renderer = SceneRenderer::new();
        let mut map_renderer = MapRenderer::new(renderer.target.width, renderer.target.height - 12);
        map_renderer.layer_enabled.remove(LayerFlags::Sprites);
        map_renderer.setup_for_map(&mut scene.map);

        camera.center_to(x as f64, y as f64);

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

            mouse_x: 0,
            mouse_y: 0,

            debug_text: None,
            debug_text_x: 0,
            debug_text_y: 0,
            debug_box: None,

            next_game_event: None,
        }
    }
}

impl GameStateTrait for GameStateScene<'_> {
    fn tick(&mut self, delta: f64) -> Option<GameEvent> {
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

        if self.next_game_event.is_some() {
            let event = self.next_game_event;
            self.next_game_event = None;
            return event;
        }

        None
    }

    fn render(&mut self, lerp: f64, renderer: &mut Renderer) {
        self.camera.lerp(lerp);
        self.map_renderer.render(lerp, &self.camera, &mut renderer.target, &self.scene.map, &self.scene.tileset_l12, &self.scene.tileset_l3, &self.scene.palette, &self.scene.render_sprites, &self.sprites);
        self.scene_renderer.render(lerp, &self.camera, &mut self.scene, &mut renderer.target);

        if self.debug_text.is_some() {
            renderer.render_text(
                &mut self.debug_text.as_mut().unwrap(),
                self.debug_text_x - self.camera.lerp_x as i32, self.debug_text_y - self.camera.lerp_y as i32,
                TextFlags::AlignHCenter | TextFlags::AlignVEnd | TextFlags::ClampToTarget,
            );
        }
        if self.debug_box.is_some() {
            renderer.render_box(
                self.debug_box.as_mut().unwrap().moved_by(-self.camera.lerp_x as i32, -self.camera.lerp_y as i32),
                [255, 255, 255, 127],
                SurfaceBlendOps::Blend,
            );
        }
    }

    fn get_title(&self, l10n: &L10n) -> String {
        format!("0x{:03X} - {}", self.scene.index, l10n.get_indexed(IndexedType::Scene, self.scene.index))
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
                        println!("Debug layer for Z plane data & flags.");
                    },
                    Some(Keycode::B) => {
                        self.scene_renderer.debug_layer = SceneDebugLayer::Movement;
                        println!("Debug layer for movement.");
                    },
                    Some(Keycode::N) => {
                        self.scene_renderer.debug_layer = SceneDebugLayer::DoorTrigger;
                        println!("Debug layer for door triggers.");
                    },
                    Some(Keycode::M) => {
                        self.scene_renderer.debug_layer = SceneDebugLayer::SpritePriority;
                        println!("Debug layer for sprite priority data.");
                    },
                    Some(Keycode::Comma) => {
                        self.scene_renderer.debug_layer = SceneDebugLayer::Exits;
                        println!("Debug layer for exits.");
                    },
                    Some(Keycode::Period) => {
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

            Event::MouseButtonDown { mouse_btn, .. } => {
                if *mouse_btn == MouseButton::Left {
                    let index = self.get_exit_at(self.mouse_x, self.mouse_y);
                    if index.is_some() {
                        let exit = &self.scene.exits[index.unwrap()];
                        self.next_game_event = Some(GameEvent::GotoDestination {
                            destination: exit.destination,
                        });
                    }
                }
            },

            _ => {},
        }
    }

    fn mouse_motion(&mut self, x: i32, y: i32) {

        // Keep world coordinate mouse position.
        self.mouse_x = (x as f64 + self.camera.x) as i32;
        self.mouse_y = (y as f64 + self.camera.y) as i32;

        let mut index = self.get_exit_at(self.mouse_x, self.mouse_y);
        if index.is_some() {
            let exit = &self.scene.exits[index.unwrap()];
            let text = exit.destination.info(&self.l10n);

            self.debug_text = Some(TextRenderable::new(
                text,
                [223, 223, 223, 255],
                TextDrawFlags::SHADOW,
                0,
            ));
            self.debug_text_x = exit.x + exit.width / 2;
            self.debug_text_y = exit.y;
            self.debug_box = Some(Rect::new(
                exit.x, exit.y,
                exit.x + exit.width, exit.y + exit.height,
            ));
        }

        if index.is_none() {
            index = self.get_treasure_at(self.mouse_x, self.mouse_y);
            if index.is_some() {
                let treasure = &self.scene.treasure[index.unwrap()];
                let text = if treasure.gold > 0 {
                    format!("{} gold", treasure.gold)
                } else if treasure.item > 0 {
                    format!("0x{:03X} '{}'", treasure.item, self.l10n.get_indexed(IndexedType::Item, treasure.item))
                } else {
                    "Empty".to_string()
                };
                self.debug_text = Some(TextRenderable::new(
                    text,
                    [223, 223, 223, 255],
                    TextDrawFlags::SHADOW,
                    0,
                ));
                self.debug_text_x = treasure.tile_x * 16 + 8;
                self.debug_text_y = treasure.tile_y * 16;
                self.debug_box = Some(Rect::new(
                    treasure.tile_x * 16, treasure.tile_y * 16,
                    treasure.tile_x * 16 + 16, treasure.tile_y * 16 + 16,
                ));
            }
        }

        if index.is_none() {
            self.debug_text = None;
            self.debug_box = None;
            self.debug_text_x = 0;
            self.debug_text_y = 0;
        }
    }

    fn dump(&mut self) {
        self.scene.dump(self.l10n);

        // for (_, set) in sprites.anim_sets.iter() {
        //     set.dump();
        // }
    }
}

impl GameStateScene<'_> {

    fn get_exit_at(&self, x: i32, y: i32) -> Option<usize> {
        for (index, exit) in self.scene.exits.iter().enumerate() {
            if x < exit.x - 4 || x >= exit.x + exit.width + 4 || y < exit.y - 4 || y >= exit.y + exit.height + 4 {
                continue;
            }
            return Some(index);
        }

        None
    }

    fn get_treasure_at(&self, x: i32, y: i32) -> Option<usize> {
        for (index, treasure) in self.scene.treasure.iter().enumerate() {
            if x < treasure.tile_x * 16 || x >= treasure.tile_x * 16 + 16 || y < treasure.tile_y * 16 || y >= treasure.tile_y * 16 + 16 {
                continue;
            }
            return Some(index);
        }

        None
    }
}
