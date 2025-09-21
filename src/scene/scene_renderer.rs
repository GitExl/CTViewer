use std::path::Path;
use crate::actor::{Actor, DebugSprite, ActorTask};
use crate::camera::Camera;
use crate::scene::scene::{Scene, SceneTreasure};
use crate::scene::scene::SceneExit;
use crate::scene::scene_map::SceneMap;
use crate::scene::scene_map::SceneMoveDirection;
use crate::scene::scene_map::SceneTileCollision;
use crate::scene::scene_map::SceneTileFlags;
use crate::software_renderer::blit::blit_surface_to_surface;
use crate::software_renderer::blit::SurfaceBlendOps;
use crate::software_renderer::clip::Rect;
use crate::software_renderer::draw::{draw_box, draw_line};
use crate::software_renderer::palette::render_palette;
use crate::software_renderer::surface::Surface;
use crate::sprites::sprite_renderer::SpritePriority;

#[derive(PartialEq, Eq)]
pub enum SceneDebugLayer {
    Disabled,
    SpritePriority,
    DoorTrigger,
    Movement,
    PcCollision,
    NpcCollision,
    ZPlane,
    Exits,
    Treasure,
    Actors,
}

pub struct SceneRenderer {
    pub debug_tiles: Surface,
    pub debug_sprites: Surface,
    pub debug_layer: SceneDebugLayer,
    pub debug_palette: bool,
}

impl SceneRenderer {
    pub fn new() -> SceneRenderer {
        SceneRenderer {
            debug_sprites: Surface::from_png(Path::new("data/scene_debug_sprites.png")),
            debug_tiles: Surface::from_png(Path::new("data/scene_debug_tiles.png")),
            debug_layer: SceneDebugLayer::Disabled,
            debug_palette: false,
        }
    }

    pub fn render(&mut self, _lerp: f64, camera: &Camera, scene: &mut Scene, surface: &mut Surface) {
        if self.debug_layer != SceneDebugLayer::Disabled {
            self.render_debug_layer(&scene.scene_map, &camera, surface);
        }

        if self.debug_layer == SceneDebugLayer::Exits {
            self.render_debug_exits(&scene.exits, &camera, surface);
        } else if self.debug_layer == SceneDebugLayer::Treasure {
            self.render_debug_treasure(&scene.treasure, &camera, surface);
        } else if self.debug_layer == SceneDebugLayer::Actors {
            self.render_debug_actors(&scene.actors, &camera, surface);
        }

        if self.debug_palette {
            render_palette(&scene.palette.palette, surface, 8);
        }
    }

    fn render_debug_exits(&mut self, exits: &Vec<SceneExit>, camera: &Camera, surface: &mut Surface) {
        for exit in exits {
            let pos = exit.pos - camera.pos_lerp.as_vec2d_i32();
            draw_box(surface, Rect::new(pos.x, pos.y, pos.x + exit.size.x, pos.y + exit.size.y), [255, 0, 255, 127], SurfaceBlendOps::Blend);
        }
    }

    fn render_debug_treasure(&mut self, treasure: &Vec<SceneTreasure>, camera: &Camera, surface: &mut Surface) {
        for item in treasure {
            let pos = item.tile_pos * 16 - camera.pos_lerp.as_vec2d_i32();
            draw_box(surface, Rect::new(pos.x, pos.y, pos.x + 16, pos.y + 16), [0, 255, 0, 127], SurfaceBlendOps::Blend);
        }
    }

    fn render_debug_actors(&mut self, actors: &Vec<Actor>, camera: &Camera, surface: &mut Surface) {
        for actor in actors {
            let x = (actor.pos_lerp.x - camera.pos_lerp.x).floor() as i32;
            let y = (actor.pos_lerp.y - camera.pos_lerp.y).floor() as i32;

            draw_box(surface, Rect::new(x - 8, y - 16, x + 8, y), [0, 255, 255, 127], SurfaceBlendOps::Blend);

            // If the actor is moving, draw the movement data.
            match actor.task {
                ActorTask::MoveByAngle { move_by, .. } => {
                    draw_line(surface, x, y, x + (move_by.x * 8.0) as i32, y + (move_by.y *8.0) as i32, [255, 0, 0, 191], SurfaceBlendOps::Blend);
                },
                ActorTask::MoveToTile { tile_pos, move_by, .. } => {
                    let (x2, y2) = (
                        (tile_pos.x as f64 * 16.0 + 8.0 - camera.pos_lerp.x).floor() as i32,
                        (tile_pos.y as f64 * 16.0 + 15.0 - camera.pos_lerp.y).floor() as i32,
                    );
                    draw_line(surface, x, y, x2, y2, [0, 255, 0, 191], SurfaceBlendOps::Blend);
                    draw_line(surface, x, y, x + (move_by.x * 8.0) as i32, y + (move_by.y *8.0) as i32, [255, 0, 0, 191], SurfaceBlendOps::Blend);
                },
                _ => {},
            }

            // Draw debug sprite.
            let (src_x, src_y) = match actor.debug_sprite {
                DebugSprite::Moving => (0, 0),
                DebugSprite::Waiting => (8, 0),
                DebugSprite::Animating => (16, 0),
                DebugSprite::None => continue,
            };
            blit_surface_to_surface(&self.debug_sprites, surface, src_x, src_y, 8, 8, x - 4, y - 12, SurfaceBlendOps::CopyAlpha);
        }
    }

    fn render_debug_layer(&mut self, scene_map: &SceneMap, camera: &Camera, surface: &mut Surface) {
        let tile1 = (camera.pos_lerp / 16.0).floor().as_vec2d_i32();
        let tile2 = ((camera.pos_lerp + camera.size) / 16.0).ceil().as_vec2d_i32();

        let props_width = scene_map.props.width as i32;
        let props_height = scene_map.props.height as i32;

        for tile_y in tile1.y..tile2.y {
            let tile_y_wrap: i32;
            if tile_y < 0 {
                tile_y_wrap = props_height - (tile_y.abs() % props_height) - 1;
            } else {
                tile_y_wrap = tile_y % props_height;
            }

            for tile_x in tile1.x..tile2.x {
                let tile_x_wrap: i32;
                if tile_x < 0 {
                    tile_x_wrap = props_width - (tile_x.abs() % props_width) - 1;
                } else {
                    tile_x_wrap = tile_x % props_width;
                }

                let tile_offset = (tile_x_wrap + tile_y_wrap * props_width) as usize;
                let tile = &scene_map.props.props[tile_offset];

                let src_x;
                let src_y;
                if self.debug_layer == SceneDebugLayer::SpritePriority {
                    if tile.sprite_priority_top == SpritePriority::BelowL2AboveL1 && tile.sprite_priority_bottom == SpritePriority::BelowL2AboveL1 {
                        (src_x, src_y) = (6, 10);
                    } else if tile.sprite_priority_top == SpritePriority::BelowL2AboveL1 && tile.sprite_priority_bottom == SpritePriority::AboveAll {
                        (src_x, src_y) = (5, 10);
                    } else if tile.sprite_priority_top == SpritePriority::AboveAll && tile.sprite_priority_bottom == SpritePriority::BelowL2AboveL1 {
                        (src_x, src_y) = (4, 10);
                    } else if tile.sprite_priority_top == SpritePriority::AboveAll && tile.sprite_priority_bottom == SpritePriority::AboveAll {
                        (src_x, src_y) = (7, 10);
                    } else {
                        continue;
                    }
                } else if self.debug_layer == SceneDebugLayer::ZPlane {
                    (src_x, src_y) = match tile.z_plane {
                        0 => (1, 5),
                        1 => (2, 5),
                        2 => (3, 5),
                        3 => (4, 5),
                        _ => continue,
                    };
                } else if self.debug_layer == SceneDebugLayer::NpcCollision {
                    if tile.flags.contains(SceneTileFlags::NPC_COLLISION_BATTLE) && tile.flags.contains(SceneTileFlags::NPC_COLLISION) {
                        (src_x, src_y) = (2, 0);
                    } else if tile.flags.contains(SceneTileFlags::NPC_COLLISION_BATTLE) {
                        (src_x, src_y) = (4, 0);
                    } else if tile.flags.contains(SceneTileFlags::NPC_COLLISION) {
                        (src_x, src_y) = (3, 0);
                    } else {
                        continue;
                    }
                } else if self.debug_layer == SceneDebugLayer::DoorTrigger {
                    if tile.flags.contains(SceneTileFlags::DOOR_TRIGGER) {
                        (src_x, src_y) = (0, 6);
                    } else {
                        continue;
                    }
                } else if self.debug_layer == SceneDebugLayer::Movement {
                    if tile.move_speed > 0 {
                        (src_x, src_y) = match tile.move_direction {
                            SceneMoveDirection::North => (5, 4),
                            SceneMoveDirection::South => (6, 4),
                            SceneMoveDirection::East => (7, 4),
                            SceneMoveDirection::West => (0, 5),
                        }
                    } else {
                        continue;
                    }
                } else if self.debug_layer == SceneDebugLayer::PcCollision {
                    if tile.flags.contains(SceneTileFlags::COLLISION_INVERTED) {
                        (src_x, src_y) = match tile.collision {
                            SceneTileCollision::None => continue,

                            SceneTileCollision::Full => continue,

                            SceneTileCollision::Corner45NW => (0, 7),
                            SceneTileCollision::Corner45NE => (1, 7),
                            SceneTileCollision::Corner45SW => (2, 7),
                            SceneTileCollision::Corner45SE => (3, 7),

                            SceneTileCollision::Corner30NW => (4, 7),
                            SceneTileCollision::Corner30NE => (5, 7),
                            SceneTileCollision::Corner30SW => (6, 7),
                            SceneTileCollision::Corner30SE => (7, 7),

                            SceneTileCollision::Corner22NW => (0, 8),
                            SceneTileCollision::Corner22NE => (1, 8),
                            SceneTileCollision::Corner22SW => (2, 8),
                            SceneTileCollision::Corner22SE => (3, 8),

                            SceneTileCollision::Corner75NW => (4, 8),
                            SceneTileCollision::Corner75NE => (5, 8),
                            SceneTileCollision::Corner75SW => (6, 8),
                            SceneTileCollision::Corner75SE => (7, 8),

                            SceneTileCollision::Corner75NWDup => (0, 9),
                            SceneTileCollision::Corner75NEDup => (1, 9),
                            SceneTileCollision::Corner75SWDup => (2, 9),
                            SceneTileCollision::Corner75SEDup => (3, 9),

                            SceneTileCollision::StairsSWNE => (4, 3),
                            SceneTileCollision::StairsSENW => (5, 3),

                            SceneTileCollision::LeftHalf => (6, 9),
                            SceneTileCollision::TopHalf => (7, 9),

                            SceneTileCollision::SW => (0, 10),
                            SceneTileCollision::SE => (1, 10),
                            SceneTileCollision::NE => (2, 10),
                            SceneTileCollision::NW => (3, 10),

                            SceneTileCollision::Ladder => (4, 4),

                            SceneTileCollision::Invalid => (7, 5),
                        };
                    } else {
                        (src_x, src_y) = match tile.collision {
                            SceneTileCollision::None => continue,

                            SceneTileCollision::Full => (7, 0),

                            SceneTileCollision::Corner45NW => (0, 1),
                            SceneTileCollision::Corner45NE => (1, 1),
                            SceneTileCollision::Corner45SW => (2, 1),
                            SceneTileCollision::Corner45SE => (3, 1),

                            SceneTileCollision::Corner30NW => (4, 1),
                            SceneTileCollision::Corner30NE => (5, 1),
                            SceneTileCollision::Corner30SW => (6, 1),
                            SceneTileCollision::Corner30SE => (7, 1),

                            SceneTileCollision::Corner22NW => (0, 2),
                            SceneTileCollision::Corner22NE => (1, 2),
                            SceneTileCollision::Corner22SW => (2, 2),
                            SceneTileCollision::Corner22SE => (3, 2),

                            SceneTileCollision::Corner75NW => (4, 2),
                            SceneTileCollision::Corner75NE => (5, 2),
                            SceneTileCollision::Corner75SW => (6, 2),
                            SceneTileCollision::Corner75SE => (7, 2),

                            SceneTileCollision::Corner75NWDup => (0, 3),
                            SceneTileCollision::Corner75NEDup => (1, 3),
                            SceneTileCollision::Corner75SWDup => (2, 3),
                            SceneTileCollision::Corner75SEDup => (3, 3),

                            SceneTileCollision::StairsSWNE => (4, 3),
                            SceneTileCollision::StairsSENW => (5, 3),

                            SceneTileCollision::LeftHalf => (6, 3),
                            SceneTileCollision::TopHalf => (7, 3),

                            SceneTileCollision::SW => (0, 4),
                            SceneTileCollision::SE => (1, 4),
                            SceneTileCollision::NE => (2, 4),
                            SceneTileCollision::NW => (3, 4),

                            SceneTileCollision::Ladder => (4, 4),

                            SceneTileCollision::Invalid => (7, 5),
                        };
                    }
                } else {
                    continue;
                }

                let px = tile_x * 16 - camera.pos_lerp.x.floor() as i32;
                let py = tile_y * 16 - camera.pos_lerp.y.floor() as i32;
                blit_surface_to_surface(&self.debug_tiles, surface, src_x * 16, src_y * 16, 16, 16, px, py, SurfaceBlendOps::Blend);

                // Second pass for some layers.
                let src_x;
                let src_y;
                if self.debug_layer == SceneDebugLayer::ZPlane {
                    if tile.flags.contains(SceneTileFlags::COLLISION_IGNORE_Z) && tile.flags.contains(SceneTileFlags::Z_NEUTRAL) {
                        (src_x, src_y) = (7, 5);
                    } else if tile.flags.contains(SceneTileFlags::COLLISION_IGNORE_Z) {
                        (src_x, src_y) = (5, 5);
                    } else if tile.flags.contains(SceneTileFlags::Z_NEUTRAL) {
                        (src_x, src_y) = (6, 5);
                    } else {
                        continue;
                    }
                    blit_surface_to_surface(&self.debug_tiles, surface, src_x * 16, src_y * 16, 16, 16, px, py, SurfaceBlendOps::Blend);
                }
            }
        }
    }

}
