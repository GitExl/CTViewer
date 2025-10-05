use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::mouse::MouseButton;
use crate::camera::Camera;
use crate::{Context, GameEvent};
use crate::gamestate::gamestate::GameStateTrait;
use crate::l10n::IndexedType;
use crate::map_renderer::LayerFlags;
use crate::map_renderer::MapRenderer;
use crate::renderer::{TextFlags, TextFont, TextRenderable};
use crate::scene::textbox::TextBox;
use crate::scene::scene::Scene;
use crate::scene::scene_renderer::{SceneDebugLayer, SceneRenderer};
use crate::software_renderer::blit::SurfaceBlendOps;
use crate::util::rect::Rect;
use crate::software_renderer::text::TextDrawFlags;
use crate::util::vec2df64::Vec2Df64;
use crate::util::vec2di32::Vec2Di32;

pub struct GameStateScene {
    pub scene: Scene,

    pub camera: Camera,
    map_renderer: MapRenderer,
    scene_renderer: SceneRenderer,

    textbox: TextBox,

    key_up: bool,
    key_down: bool,
    key_left: bool,
    key_right: bool,
    key_activate: bool,

    mouse_pos: Vec2Di32,

    debug_text: Option<TextRenderable>,
    debug_text_x: i32,
    debug_text_y: i32,
    debug_box: Option<Rect>,
    debug_actor: Option<usize>,

    next_game_event: Option<GameEvent>,
}

impl GameStateScene {
    pub fn new(ctx: &mut Context, scene_index: usize, camera_center: Vec2Df64) -> GameStateScene {
        ctx.sprites_states.clear();

        let mut scene = ctx.fs.read_scene(scene_index);

        let mut camera = Camera::new(
            scene.scroll_mask.left as f64, scene.scroll_mask.top as f64,
            ctx.render.target.width as f64, ctx.render.target.height as f64,
            scene.scroll_mask.left as f64, scene.scroll_mask.top as f64,
            scene.scroll_mask.right as f64, scene.scroll_mask.bottom as f64,
        );

        let scene_renderer = SceneRenderer::new();
        let mut map_renderer = MapRenderer::new(ctx.render.target.width, ctx.render.target.height);
        map_renderer.setup_for_map(&mut scene.map);

        camera.center_to(camera_center);

        let mut textbox = TextBox::new(ctx);
        scene.init(ctx, &mut textbox, &mut camera);

        println!("Entering scene {}: {}", scene.index, ctx.l10n.get_indexed(IndexedType::Scene, scene.index));

        GameStateScene {
            scene,

            camera,
            scene_renderer,
            map_renderer,

            textbox,

            key_down: false,
            key_left: false,
            key_right: false,
            key_up: false,
            key_activate: false,

            mouse_pos: Vec2Di32::default(),

            debug_text: None,
            debug_text_x: 0,
            debug_text_y: 0,
            debug_box: None,
            debug_actor: None,

            next_game_event: None,
        }
    }
}

impl GameStateTrait for GameStateScene {
    fn tick(&mut self, ctx: &mut Context, delta: f64) -> Option<GameEvent> {
        self.scene.tick(ctx, &mut self.textbox, &mut self.camera, delta);

        self.camera.tick(delta);
        if let Some(debug_actor) = self.debug_actor {
            self.camera.pos = self.scene.actors[debug_actor].pos - Vec2Df64::new(self.camera.size.x - 64.0, self.camera.size.y / 2.0);
            self.camera.clamp();
        } else {
            if self.key_up {
                self.camera.pos.y -= 300.0 * delta;
            }
            else if self.key_down {
                self.camera.pos.y += 300.0 * delta;
            }
            if self.key_left {
                self.camera.pos.x -= 300.0 * delta;
            }
            else if self.key_right {
                self.camera.pos.x += 300.0 * delta;
            }
            self.camera.clamp();
        }

        self.textbox.tick(delta);

        if self.next_game_event.is_some() {
            let event = self.next_game_event;
            self.next_game_event = None;
            return event;
        }

        None
    }

    fn render(&mut self, ctx: &mut Context, lerp: f64) {
        self.scene.lerp(ctx, lerp);
        self.camera.lerp(lerp);

        self.map_renderer.render(
            lerp,
            &self.camera,
            &mut ctx.render.target,
            &self.scene.map,
            &self.scene.tileset_l12,
            &self.scene.tileset_l3,
            &self.scene.palette,
            &ctx.sprites_states,
            &ctx.sprite_assets,
        );
        self.scene_renderer.render(
            lerp,
            &self.camera,
            &mut self.scene,
            &mut ctx.render.target,
        );

        if let Some(debug_text) = &mut self.debug_text {
            ctx.render.render_text(
                debug_text,
                self.debug_text_x - self.camera.pos_lerp.x as i32, self.debug_text_y - self.camera.pos_lerp.y as i32,
                TextFlags::AlignHCenter | TextFlags::AlignVEnd | TextFlags::ClampToTarget,
            );
        }

        if let Some(debug_box) = self.debug_box {
            ctx.render.render_box_filled(
                debug_box.moved_by(-self.camera.pos_lerp.x as i32, -self.camera.pos_lerp.y as i32),
                [255, 255, 255, 127],
                SurfaceBlendOps::Blend,
            );
        }

        if let Some(debug_actor) = self.debug_actor {
            let actor = &self.scene.actors[debug_actor];
            let pos = (actor.pos_lerp.floor() - self.camera.pos_lerp.floor()).as_vec2d_i32();
            ctx.render.render_box_filled(
                Rect::new(pos.x - 8, pos.y - 16, pos.x + 8, pos.y),
                [0, 255, 0, 127],
                SurfaceBlendOps::Blend,
            );

            ctx.render.render_box_filled(
                Rect::new(0, 0, 128, 224),
                [0, 0, 0, 191],
                SurfaceBlendOps::Blend,
            );

            let sprite_state = ctx.sprites_states.get_state(debug_actor);
            let script_state = &self.scene.script.script_states[debug_actor];
            let op: String = if let Some(current_op) = script_state.current_op {
                format!("{:?}", current_op)
            } else {
                "None".to_string()
            };

            // Spit out a bunch of internal state.
            let text_actor = format!(
                "Actor {}: {:?}\n{} {:.1} {:?}\nDrawMode::{:?}\n{:?}",
                debug_actor, actor.class,
                actor.pos, actor.move_speed, actor.facing,
                actor.draw_mode,
                actor.flags,
            );
            let text_sprite = format!(
                "Sprite {}, frame {}\nPalette {}\nTop: {:?}\nBottom: {:?}\nAnim {} frame {} delay {}\nAnimationMode::{:?}\nLoop anim {}, {} loops",
                sprite_state.sprite_index, sprite_state.sprite_frame,
                sprite_state.palette_offset,
                sprite_state.priority_top,
                sprite_state.priority_bottom,
                sprite_state.anim_index, sprite_state.anim_frame, sprite_state.anim_delay,
                sprite_state.anim_mode,
                sprite_state.anim_index_looped, sprite_state.anim_loops_remaining,
            );
            let text_script = format!(
                "0x{:04X}, d {} / {}, p {}\nPrio {}, waiting: {}\n{:04X?}\n\n{}",
                script_state.current_address, script_state.delay_counter, script_state.delay, script_state.pause_counter,
                script_state.current_priority, script_state.call_waiting,
                script_state.priority_return_ptrs,
                op,
            );

            let mut header = TextRenderable::new(format!("{}\n\n{}\n\n{}", text_actor, text_sprite, text_script), TextFont::Small, [255, 255, 255, 255], TextDrawFlags::empty(), 124);
            ctx.render.render_text(&mut header, 2, 2, TextFlags::empty());
        }

        self.textbox.render(ctx, lerp);
    }

    fn get_title(&self, ctx: &Context) -> String {
        format!("{} - {}", self.scene.index, ctx.l10n.get_indexed(IndexedType::Scene, self.scene.index))
    }

    fn event(&mut self, _ctx: &mut Context, event: &Event) {
        match event {
            Event::KeyDown { keycode, .. } => {
                match keycode {
                    Some(Keycode::W) => self.key_up = true,
                    Some(Keycode::A) => self.key_left = true,
                    Some(Keycode::S) => self.key_down = true,
                    Some(Keycode::D) => self.key_right = true,

                    Some(Keycode::F) => {
                        self.key_activate = true;
                        self.textbox.progress();
                    },

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
                    Some(Keycode::Slash) => {
                        self.scene_renderer.debug_layer = SceneDebugLayer::Actors;
                        println!("Debug layer for actors.");
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
                    Some(Keycode::F) => self.key_activate = false,
                    _ => {},
                }
            },

            Event::MouseButtonDown { mouse_btn, .. } => {
                if *mouse_btn == MouseButton::Left {
                    let index = self.get_actor_at(self.mouse_pos);
                    if let Some(index) = index {
                        self.debug_actor = Some(index);
                    } else {
                        self.debug_actor = None;
                    }

                    if index.is_none() {
                        let index = self.get_exit_at(self.mouse_pos);
                        if let Some(index) = index {
                            let exit = &self.scene.exits[index];
                            self.next_game_event = Some(GameEvent::GotoDestination {
                                destination: exit.destination,
                            });
                        }

                    }
                }
            },

            _ => {},
        }
    }

    fn mouse_motion(&mut self, ctx: &Context, x: i32, y: i32) {

        // Keep world coordinate mouse position.
        self.mouse_pos = Vec2Di32::new(
            (x as f64 + self.camera.pos.x) as i32,
            (y as f64 + self.camera.pos.y) as i32,
        );

        let mut index = self.get_actor_at(self.mouse_pos);
        if let Some(index) = index {
            let actor = &self.scene.actors[index];
            let text = format!("Actor {}", index);
            self.debug_text = Some(TextRenderable::new(
                text,
                TextFont::Small,
                [231, 231, 231, 255],
                TextDrawFlags::SHADOW,
                0,
            ));
            self.debug_text_x = actor.pos.x as i32;
            self.debug_text_y = actor.pos.y as i32 + 9;
            self.debug_box = Some(Rect::new(
                actor.pos.x as i32 - 8, actor.pos.y as i32 - 16,
                actor.pos.x as i32 + 8, actor.pos.y as i32,
            ));
        }

        if index.is_none() {
            index = self.get_exit_at(self.mouse_pos);
            if let Some(index) = index {
                let exit = &self.scene.exits[index];
                let text = exit.destination.info(&ctx);

                self.debug_text = Some(TextRenderable::new(
                    text,
                    TextFont::Small,
                    [231, 231, 231, 255],
                    TextDrawFlags::SHADOW,
                    0,
                ));
                self.debug_text_x = exit.pos.x + exit.size.x / 2;
                self.debug_text_y = exit.pos.y;
                self.debug_box = Some(Rect::new(
                    exit.pos.x, exit.pos.y,
                    exit.pos.x + exit.size.x, exit.pos.y + exit.size.y,
                ));
            }
        }

        if index.is_none() {
            index = self.get_treasure_at(self.mouse_pos);
            if let Some(index) = index {
                let treasure = &self.scene.treasure[index];
                let text = if treasure.gold > 0 {
                    format!("{} gold", treasure.gold)
                } else if treasure.item > 0 {
                    format!("{}", ctx.l10n.get_indexed(IndexedType::Item, treasure.item))
                } else {
                    "Empty".to_string()
                };
                self.debug_text = Some(TextRenderable::new(
                    text,
                    TextFont::Small,
                    [231, 231, 231, 255],
                    TextDrawFlags::SHADOW,
                    0,
                ));
                self.debug_text_x = treasure.tile_pos.x * 16 + 8;
                self.debug_text_y = treasure.tile_pos.y * 16;
                self.debug_box = Some(Rect::new(
                    treasure.tile_pos.x * 16, treasure.tile_pos.y * 16,
                    treasure.tile_pos.x * 16 + 16, treasure.tile_pos.y * 16 + 16,
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

    fn dump(&mut self, ctx: &Context) {
        self.scene.dump(ctx);

        // for (_, set) in sprites.anim_sets.iter() {
        //     set.dump();
        // }
    }
}

impl GameStateScene {

    fn get_exit_at(&self, pos: Vec2Di32) -> Option<usize> {
        for (index, exit) in self.scene.exits.iter().enumerate() {
            if pos.x < exit.pos.x - 8 || pos.x >= exit.pos.x + exit.size.x + 8 ||
               pos.y < exit.pos.y - 8 || pos.y >= exit.pos.y + exit.size.y + 8 {
                continue;
            }
            return Some(index);
        }

        None
    }

    fn get_treasure_at(&self, pos: Vec2Di32) -> Option<usize> {
        for (index, treasure) in self.scene.treasure.iter().enumerate() {
            if pos.x < treasure.tile_pos.x * 16 || pos.x >= treasure.tile_pos.x * 16 + 16 ||
               pos.y < treasure.tile_pos.y * 16 || pos.y >= treasure.tile_pos.y * 16 + 16 {
                continue;
            }
            return Some(index);
        }

        None
    }

    fn get_actor_at(&self, pos: Vec2Di32) -> Option<usize> {
        for (index, actor) in self.scene.actors.iter().enumerate() {
            if pos.x < actor.pos.x as i32 - 8 || pos.x >= actor.pos.x as i32 + 8 ||
               pos.y < actor.pos.y as i32 - 16 || pos.y >= actor.pos.y as i32 {
                continue;
            }
            return Some(index);
        }

        None
    }
}
