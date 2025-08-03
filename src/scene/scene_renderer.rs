use std::path::Path;

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
use crate::software_renderer::draw::draw_box;
use crate::software_renderer::palette::render_palette;
use crate::software_renderer::surface::Surface;

#[derive(PartialEq, Eq)]
pub enum SceneDebugLayer {
    Disabled,
    SpritePriority,
    DoorTrigger,
    Movement,
    Collision,
    CollisionNpc,
    CollisionBattle,
    ZPlane,
    ZPlaneFlags,
    Exits,
    Treasure,
}

pub struct SceneRenderer {
    pub debug_tiles: Surface,
    pub debug_layer: SceneDebugLayer,
    pub debug_palette: bool,
}

impl SceneRenderer {
    pub fn new() -> SceneRenderer {
        SceneRenderer {
            debug_tiles: Surface::from_png(Path::new("data/scene_debug_tiles.png")),
            debug_layer: SceneDebugLayer::Disabled,
            debug_palette: false,
        }
    }

    pub fn render(&mut self, lerp: f64, camera: &Camera, scene: &mut Scene, surface: &mut Surface) {
        scene.lerp(lerp);

        if self.debug_layer != SceneDebugLayer::Disabled {
            self.render_debug_layer(&scene.scene_map, &camera, surface);
        }

        if self.debug_layer == SceneDebugLayer::Exits {
            self.render_debug_exits(&scene.exits, &camera, surface);
        } else if self.debug_layer == SceneDebugLayer::Treasure {
            self.render_debug_treasure(&scene.treasure, &camera, surface);
        }

        if self.debug_palette {
            render_palette(&scene.palette.palette, surface, 8);
        }
    }

    fn render_debug_exits(&mut self, exits: &Vec<SceneExit>, camera: &Camera, surface: &mut Surface) {
        for exit in exits {
            let x = exit.x - camera.lerp_x.floor() as i32;
            let y = exit.y - camera.lerp_y.floor() as i32;
            draw_box(surface, Rect::new(x, y, x + exit.width as i32, y + exit.height as i32), [255, 255, 0, 191], SurfaceBlendOps::Blend);
        }
    }

    fn render_debug_treasure(&mut self, treasure: &Vec<SceneTreasure>, camera: &Camera, surface: &mut Surface) {
        for item in treasure {
            let x = item.tile_x as i32 * 16 - camera.lerp_x.floor() as i32;
            let y = item.tile_y as i32 * 16 - camera.lerp_y.floor() as i32;
            draw_box(surface, Rect::new(x, y, x + 16, y + 16), [0, 255, 0, 191], SurfaceBlendOps::Blend);
        }
    }

    fn render_debug_layer(&mut self, scene_map: &SceneMap, camera: &Camera, surface: &mut Surface) {
        let tile_x1 = (camera.lerp_x / 16.0).floor() as i32;
        let tile_y1 = (camera.lerp_y / 16.0).floor() as i32;
        let tile_x2 = ((camera.lerp_x + camera.width) / 16.0).ceil() as i32;
        let tile_y2 = ((camera.lerp_y + camera.height) / 16.0).ceil() as i32;
        
        let props_width = scene_map.props.width as i32;
        let props_height = scene_map.props.height as i32;

        for tile_y in tile_y1..tile_y2 {
            let tile_y_wrap: i32;
            if tile_y < 0 {
                tile_y_wrap = props_height - (tile_y.abs() % props_height) - 1;
            } else {
                tile_y_wrap = tile_y % props_height;
            }

            for tile_x in tile_x1..tile_x2 {
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
                    if tile.flags.contains(SceneTileFlags::SPRITE_OVER_L1) && tile.flags.contains(SceneTileFlags::SPRITE_OVER_L2) {
                        (src_x, src_y) = (7, 10);
                    } else if tile.flags.contains(SceneTileFlags::SPRITE_OVER_L1) {
                        (src_x, src_y) = (5, 10);
                    } else if tile.flags.contains(SceneTileFlags::SPRITE_OVER_L2) {
                        (src_x, src_y) = (6, 10);
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

                } else if self.debug_layer == SceneDebugLayer::ZPlaneFlags {
                    if tile.flags.contains(SceneTileFlags::COLLISION_IGNORE_Z) {
                        (src_x, src_y) = (5, 5);
                    } else if tile.flags.contains(SceneTileFlags::Z_NEUTRAL) {
                        (src_x, src_y) = (6, 5);
                    } else {
                        continue;
                    }

                } else if self.debug_layer == SceneDebugLayer::CollisionNpc {
                    if tile.flags.contains(SceneTileFlags::COLLISION_NPC) {
                        (src_x, src_y) = (3, 0);
                    } else {
                        continue;
                    }

                } else if self.debug_layer == SceneDebugLayer::CollisionBattle {
                    if tile.flags.contains(SceneTileFlags::COLLISION_BATTLE) {
                        (src_x, src_y) = (4, 0);
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

                } else if self.debug_layer == SceneDebugLayer::Collision {
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

                let px = tile_x * 16 - camera.lerp_x.floor() as i32;
                let py = tile_y * 16 - camera.lerp_y.floor() as i32;
                blit_surface_to_surface(&self.debug_tiles, surface, src_x * 16, src_y * 16, 16, 16, px, py, SurfaceBlendOps::Blend);
            }
        }
    }

}
