use std::path::Path;

use crate::camera::Camera;
use crate::map::Map;
use crate::software_renderer::blit::blit_surface_to_surface;
use crate::software_renderer::blit::SurfaceBlendOps;
use crate::software_renderer::palette::render_palette;
use crate::software_renderer::surface::Surface;
use crate::world::world::WorldExit;
use crate::world::world::ScriptedWorldExit;
use crate::world::world::World;
use crate::world::world_map::WorldChipFlags;
use crate::world::world_map::WorldMap;

#[derive(PartialEq, Eq)]
pub enum WorldDebugLayer {
    Disabled,
    Solidity,
    Exits,
    Music,
}

pub struct WorldRenderer {
    pub debug_tiles: Surface,
    pub debug_layer: WorldDebugLayer,
    pub debug_palette: bool,
}

impl WorldRenderer {
    pub fn new() -> WorldRenderer {
        WorldRenderer {
            debug_tiles: Surface::from_png(Path::new("data/world_debug_tiles.png")),
            debug_layer: WorldDebugLayer::Disabled,
            debug_palette: false,
        }
    }

    pub fn render(&mut self, lerp: f64, camera: &Camera, world: &mut World, surface: &mut Surface) {
        world.lerp(lerp);

        self.render_debug(camera, world, surface);
    }

    fn render_debug(&mut self, camera: &Camera, world: &World, surface: &mut Surface) {
        if self.debug_layer != WorldDebugLayer::Disabled {
            self.render_debug_layer(&world.map, &world.world_map, &camera, surface);
        }

        if self.debug_layer == WorldDebugLayer::Exits {
            self.render_debug_exits(&world.exits, &world.scripted_exits, &camera, surface);
        }

        if self.debug_palette {
            render_palette(&world.palette.palette, surface, 8);
        }
    }

    fn render_debug_exits(&mut self, exits: &Vec<WorldExit>, scripted_exits: &Vec<ScriptedWorldExit>, camera: &Camera, surface: &mut Surface) {
        for exit in exits {
            let x = exit.x - camera.lerp_x.floor() as i32;
            let y = exit.y - camera.lerp_y.floor() as i32;

            let (src_x, src_y) = if exit.is_available { (0, 8) } else { (16, 8) };
            blit_surface_to_surface(&self.debug_tiles, surface, src_x, src_y, 16, 16, x, y, SurfaceBlendOps::Blend);
        }

        for scripted_exit in scripted_exits {
            let x = scripted_exit.x - camera.lerp_x.floor() as i32;
            let y = scripted_exit.y - camera.lerp_y.floor() as i32;
            
            blit_surface_to_surface(&self.debug_tiles, surface, 32, 8, 16, 16, x, y, SurfaceBlendOps::Blend);
        }
    }

    fn render_debug_layer(&mut self, map: &Map, world_map: &WorldMap, camera: &Camera, surface: &mut Surface) {
        let chip_x1 = (camera.lerp_x / 8.0).floor() as i32;
        let chip_y1 = (camera.lerp_y / 8.0).floor() as i32;
        let chip_x2 = ((camera.lerp_x + camera.width) / 8.0).ceil() as i32;
        let chip_y2 = ((camera.lerp_y + camera.height) / 8.0).ceil() as i32;
        let layer = &map.layers[1];

        let chip_width = layer.chip_width as i32;
        let chip_height = layer.chip_height as i32;

        for chip_y in chip_y1..chip_y2 {
            let chip_y_wrap;
            if chip_y < 0 {
                chip_y_wrap = chip_height - (chip_y.abs() % chip_height) - 1;
            } else {
                chip_y_wrap = chip_y % chip_height;
            }

            for chip_x in chip_x1..chip_x2 {
                let chip_x_wrap;
                if chip_x < 0 {
                    chip_x_wrap = chip_width - (chip_x.abs() % chip_width) - 1;
                } else {
                    chip_x_wrap = chip_x % chip_width;
                }

                let chip_index = (chip_x_wrap + chip_y_wrap * chip_width) as usize;
                let chip = &world_map.chips[chip_index];

                let src_x;
                let src_y;
                if self.debug_layer == WorldDebugLayer::Exits {
                    if chip.flags.contains(WorldChipFlags::HAS_EXIT) {
                        (src_x, src_y) = (3, 0);
                    } else {
                        continue;
                    }
                } else if self.debug_layer == WorldDebugLayer::Solidity {
                    if chip.flags.contains(WorldChipFlags::BLOCK_WALK) {
                        (src_x, src_y) = (0, 0);
                    } else if chip.flags.contains(WorldChipFlags::BLOCK_HOVER) {
                        (src_x, src_y) = (1, 0);
                    } else if chip.flags.contains(WorldChipFlags::BLOCK_HOVER) {
                        (src_x, src_y) = (2, 0);
                    } else {
                        continue;
                    }
                } else if self.debug_layer == WorldDebugLayer::Music {
                    (src_x, src_y) = match chip.music {
                        0 => continue,
                        1 => (4, 0),
                        2 => (5, 0),
                        3 => (6, 0),
                        4 => (7, 0),
                        5 => (8, 0),
                        6 => (9, 0),
                        7 => (10, 0),
                        _ => (0, 0),
                    }
                } else {
                    continue;
                }

                let px = (chip_x * 8) - camera.lerp_x.floor() as i32;
                let py = (chip_y * 8) - camera.lerp_y.floor() as i32;
                blit_surface_to_surface(&self.debug_tiles, surface, src_x * 8, src_y * 8, 8, 8, px, py, SurfaceBlendOps::Blend);
            }
        }
    }

}
