use bitflags::bitflags;

use crate::camera::Camera;
use crate::game_palette::GamePalette;
use crate::map::{EffectFlags, LayerScrollMode, ScreenFlags};
use crate::map::Map;
use crate::map::MapChipFlags;
use crate::map::MapLayer;
use crate::software_renderer::bitmap::Bitmap;
use crate::software_renderer::blit::blit_bitmap_to_surface_and_source;
use crate::software_renderer::blit::BitmapBlitFlags;
use crate::software_renderer::palette::Color;
use crate::software_renderer::palette::Palette;
use crate::software_renderer::surface::Surface;
use crate::sprites::sprite_list::SpriteList;
use crate::sprites::sprite_renderer::render_sprite;
use crate::tileset::TileSet;

// Data used in a main or subscreen render pass.
struct RenderData<'a> {
    camera: &'a Camera,
    map: &'a Map,
    tileset_l12: &'a TileSet,
    tileset_l3: &'a TileSet,
    palette: &'a GamePalette,
    layer3_priority: bool,
    map_sprites: &'a Vec<MapSprite>,
}

// Blend modes matching the SNES PPU modes.
#[derive(Clone, Copy)]
pub enum LayerBlendMode {
    Add,
    AddHalf,
    Sub,
    SubHalf,
}

// Flags for each type of layer. The background means nothing was rendered for a pixel.
bitflags! {
    #[derive(Clone, Default, Copy)]
    pub struct LayerFlags: u8 {
        const Layer1 = 0x01;
        const Layer2 = 0x02;
        const Layer3 = 0x04;
        const Sprites = 0x08;
        const Background = 0x10;
    }
}

// Data for rendering a sprite in a map.
pub struct MapSprite {
    pub x: f64,
    pub y: f64,
    pub sprite_index: usize,
    pub frame: usize,
    pub priority: u32,
    pub palette_offset: usize,
    pub visible: bool,
}

impl MapSprite {
    pub fn new() -> MapSprite {
        MapSprite {
            x: 0.0,
            y: 0.0,
            sprite_index: 0,
            frame: 0,
            priority: 0,
            palette_offset: 0,
            visible: true,
        }
    }
}

pub struct MapRenderer {
    pub screen_sub: Surface,
    pub pixels_main: Bitmap,
    pub pixels_sub: Bitmap,

    pub layer_enabled: LayerFlags,
    pub layer_target_main: LayerFlags,
    pub layer_target_sub: LayerFlags,
    pub layer3_priority: bool,
    pub layer_blend_enable: LayerFlags,
    pub layer_blend_mode: LayerBlendMode,
    pub layer_blend_color: Color,
}

impl MapRenderer {
    pub fn new(width: u32, height: u32) -> MapRenderer {
        MapRenderer {
            screen_sub: Surface::new(width, height),
            pixels_main: Bitmap::new(width, height),
            pixels_sub: Bitmap::new(width, height),

            layer3_priority: true,
            layer_enabled: LayerFlags::Layer1 | LayerFlags::Layer2 | LayerFlags::Layer3 | LayerFlags::Sprites,

            layer_target_main: LayerFlags::default(),
            layer_target_sub: LayerFlags::default(),
            layer_blend_enable: LayerFlags::default(),
            layer_blend_mode: LayerBlendMode::Add,
            layer_blend_color: [0, 0, 0, 0],
        }
    }

    pub fn setup_for_map(&mut self, map: &Map) {
        if map.screen_flags.contains(ScreenFlags::SCREEN_L1_MAIN) {
            self.layer_target_main |= LayerFlags::Layer1;
        }
        if map.screen_flags.contains(ScreenFlags::SCREEN_L2_MAIN) {
            self.layer_target_main |= LayerFlags::Layer2;
        }
        if map.screen_flags.contains(ScreenFlags::SCREEN_L3_MAIN) {
            self.layer_target_main |= LayerFlags::Layer3;
        }
        if map.screen_flags.contains(ScreenFlags::SCREEN_SPR_MAIN) {
            self.layer_target_main |= LayerFlags::Sprites;
        }

        if map.screen_flags.contains(ScreenFlags::SCREEN_L1_SUB) {
            self.layer_target_sub |= LayerFlags::Layer1;
        }
        if map.screen_flags.contains(ScreenFlags::SCREEN_L2_SUB) {
            self.layer_target_sub |= LayerFlags::Layer2;
        }
        if map.screen_flags.contains(ScreenFlags::SCREEN_L3_SUB) {
            self.layer_target_sub |= LayerFlags::Layer3;
        }
        if map.screen_flags.contains(ScreenFlags::SCREEN_SPR_SUB) {
            self.layer_target_sub |= LayerFlags::Sprites;
        }

        if map.effect_flags.contains(EffectFlags::EFFECT_L1) {
            self.layer_blend_enable |= LayerFlags::Layer1;
        }
        if map.effect_flags.contains(EffectFlags::EFFECT_L2) {
            self.layer_blend_enable |= LayerFlags::Layer2;
        }
        if map.effect_flags.contains(EffectFlags::EFFECT_L3) {
            self.layer_blend_enable |= LayerFlags::Layer3;
        }
        if map.effect_flags.contains(EffectFlags::EFFECT_SPR) {
            self.layer_blend_enable |= LayerFlags::Sprites;
        }
        if map.effect_flags.contains(EffectFlags::EFFECT_DEFAULT_COL) {
            self.layer_blend_enable |= LayerFlags::Background;
        }

        if map.effect_flags.contains(EffectFlags::EFFECT_SUBTRACT) {
            if map.effect_flags.contains(EffectFlags::EFFECT_HALF_INTENSITY) {
                self.layer_blend_mode = LayerBlendMode::SubHalf;
            } else {
                self.layer_blend_mode = LayerBlendMode::Sub;
            }
        } else {
            if map.effect_flags.contains(EffectFlags::EFFECT_HALF_INTENSITY) {
                self.layer_blend_mode = LayerBlendMode::AddHalf;
            } else {
                self.layer_blend_mode = LayerBlendMode::Add;
            }
        }
    }

    pub fn render(&mut self, _: f64, camera: &Camera, surface: &mut Surface, map: &Map, tileset_l12: &TileSet, tileset_l3: &TileSet, palette: &GamePalette, map_sprites: &Vec<MapSprite>, sprites: &SpriteList) {
        self.screen_sub.fill(self.layer_blend_color);
        self.pixels_main.clear();
        self.pixels_sub.clear();

        let mut render_data = RenderData {
            camera,
            map,
            tileset_l12,
            tileset_l3,
            palette,
            map_sprites,
            layer3_priority: self.layer3_priority,
        };
        render_to_target(surface, &mut self.pixels_main, &mut render_data, sprites, self.layer_enabled & self.layer_target_main);
        render_to_target(&mut self.screen_sub, &mut self.pixels_sub, &mut render_data, sprites, self.layer_enabled & self.layer_target_sub);

        self.blend_surfaces(surface);
    }

    fn blend_surfaces(&mut self, dest_surface: &mut Surface) {
        for (index, src) in self.pixels_main.data.iter().enumerate() {
            let dest = index * 4;

            let input;
            if self.pixels_sub.data[index] == 0 {
                input = &self.layer_blend_color[0..3];
            } else {
                input = &self.screen_sub.data[dest + 0..dest + 3];
            }

            if src & self.layer_blend_enable.bits() > 0 && self.pixels_sub.data[index] > 0 {
                match self.layer_blend_mode {
                    LayerBlendMode::Add => {
                        dest_surface.data[dest + 0] = dest_surface.data[dest + 0].saturating_add(input[0]);
                        dest_surface.data[dest + 1] = dest_surface.data[dest + 1].saturating_add(input[1]);
                        dest_surface.data[dest + 2] = dest_surface.data[dest + 2].saturating_add(input[2]);
                    },
                    LayerBlendMode::AddHalf => {
                        dest_surface.data[dest + 0] = (dest_surface.data[dest + 0] >> 1).saturating_add(input[0] >> 1);
                        dest_surface.data[dest + 1] = (dest_surface.data[dest + 1] >> 1).saturating_add(input[1] >> 1);
                        dest_surface.data[dest + 2] = (dest_surface.data[dest + 2] >> 1).saturating_add(input[2] >> 1);
                    },
                    LayerBlendMode::Sub => {
                        dest_surface.data[dest + 0] = dest_surface.data[dest + 0].saturating_sub(input[0]);
                        dest_surface.data[dest + 1] = dest_surface.data[dest + 1].saturating_sub(input[1]);
                        dest_surface.data[dest + 2] = dest_surface.data[dest + 2].saturating_sub(input[2]);
                    },
                    LayerBlendMode::SubHalf => {
                        dest_surface.data[dest + 0] = (dest_surface.data[dest + 0] >> 1).saturating_sub(input[0] >> 1);
                        dest_surface.data[dest + 1] = (dest_surface.data[dest + 1] >> 1).saturating_sub(input[1] >> 1);
                        dest_surface.data[dest + 2] = (dest_surface.data[dest + 2] >> 1).saturating_sub(input[2] >> 1);
                    },
                }
            }
            else if *src == 0 && self.pixels_sub.data[index] > 0  {
                dest_surface.data[dest..dest + 3].copy_from_slice(input);
            }
        }
    }
}

fn render_layer(target: &mut Surface, pixel_source: &mut Bitmap, source_value: LayerFlags, tileset: &TileSet, layer: &MapLayer, priority: u8, palette: &Palette, camera: &Camera) {
    let x1;
    let y1;
    if matches!(layer.scroll_mode, LayerScrollMode::IgnoreCamera) {
        x1 = layer.lerp_scroll_x.floor();
        y1 = layer.lerp_scroll_y.floor();
    } else if matches!(layer.scroll_mode, LayerScrollMode::Parallax) {
        x1 = ((camera.lerp_x / 2.0) + layer.lerp_scroll_x).floor();
        y1 = ((camera.lerp_y / 2.0) + layer.lerp_scroll_y).floor();
    } else {
        x1 = (camera.lerp_x + layer.lerp_scroll_x).floor();
        y1 = (camera.lerp_y + layer.lerp_scroll_y).floor();
    }

    let chip_x1 = (x1 / 8.0).floor() as i32;
    let chip_y1 = (y1 / 8.0).floor() as i32;
    let chip_x2 = ((x1 + camera.width) / 8.0).ceil() as i32;
    let chip_y2 = ((y1 + camera.height) / 8.0).ceil() as i32;

    let source_bits = source_value.bits();

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

            let chip_offset = (chip_x_wrap + chip_y_wrap * chip_width) as usize;
            let chip = &layer.chips[chip_offset];
            if chip.chip == 0 || chip.chip >= tileset.chip_bitmaps.len() {
                continue;
            }

            // Priority must match what is being rendered.
            if priority == 0 && chip.flags.contains(MapChipFlags::PRIORITY) {
                continue;
            }
            if priority == 1 && !chip.flags.contains(MapChipFlags::PRIORITY) {
                continue;
            }

            let mut render_flags = BitmapBlitFlags::SKIP_0;
            if chip.flags.contains(MapChipFlags::FLIP_X) {
                render_flags |= BitmapBlitFlags::FLIP_X;
            }
            if chip.flags.contains(MapChipFlags::FLIP_Y) {
                render_flags |= BitmapBlitFlags::FLIP_Y;
            }

            let px = chip_x * 8 - x1 as i32;
            let py = chip_y * 8 - y1 as i32;
            blit_bitmap_to_surface_and_source(&tileset.chip_bitmaps[chip.chip], target, pixel_source, 0, 0, 8, 8, px, py, palette, chip.palette, source_bits, render_flags);
        }
    }
}

fn render_to_target(surface: &mut Surface, pixels: &mut Bitmap, render_data: &mut RenderData, sprites: &SpriteList, layers: LayerFlags) {

    // Layer 3, priority 0.
    if layers.contains(LayerFlags::Layer3) && render_data.map.layers[2].chips.len() > 0 {
        render_layer(surface, pixels, LayerFlags::Layer3, &render_data.tileset_l3, &render_data.map.layers[2], 0, &render_data.palette.palette, &render_data.camera);
    }

    // Sprites, priority 0.
    if layers.contains(LayerFlags::Sprites) {
        render_sprites(surface, pixels, &render_data.map_sprites, 0, &render_data.camera, &sprites);
    }

    // Layer 3, priority 1, if layer 3 does not have priority.
    if layers.contains(LayerFlags::Layer3) && render_data.map.layers[2].chips.len() > 0 && !render_data.layer3_priority {
        render_layer(surface, pixels, LayerFlags::Layer3, &render_data.tileset_l3, &render_data.map.layers[2], 1, &render_data.palette.palette, &render_data.camera);
    }

    // Sprites, priority 1.
    if layers.contains(LayerFlags::Sprites) {
        render_sprites(surface, pixels, &render_data.map_sprites, 1, &render_data.camera, &sprites);
    }

    // Layer 2 and layer 1, priority 0.
    if layers.contains(LayerFlags::Layer2) {
        render_layer(surface, pixels, LayerFlags::Layer2, &render_data.tileset_l12, &render_data.map.layers[1], 0, &render_data.palette.palette, &render_data.camera);
    }
    if layers.contains(LayerFlags::Layer1) {
        render_layer(surface, pixels, LayerFlags::Layer1, &render_data.tileset_l12, &render_data.map.layers[0], 0, &render_data.palette.palette, &render_data.camera);
    }

    // Sprites, priority 2.
    if layers.contains(LayerFlags::Sprites) {
        render_sprites(surface, pixels, &render_data.map_sprites, 2, &render_data.camera, &sprites);
    }

    // Layer 2 and layer 1, priority 1.
    if layers.contains(LayerFlags::Layer2) {
        render_layer(surface, pixels, LayerFlags::Layer2, &render_data.tileset_l12, &render_data.map.layers[1], 1, &render_data.palette.palette, &render_data.camera);
    }
    if layers.contains(LayerFlags::Layer1) {
        render_layer(surface, pixels, LayerFlags::Layer1, &render_data.tileset_l12, &render_data.map.layers[0], 1, &render_data.palette.palette, &render_data.camera);
    }

    // Sprites, priority 3.
    if layers.contains(LayerFlags::Sprites) {
        render_sprites(surface, pixels, &render_data.map_sprites, 3, &render_data.camera, &sprites);
    }

    // Layer 3, priority 1, if layer 3 has priority.
    if layers.contains(LayerFlags::Layer3) && render_data.map.layers[2].chips.len() > 0 && render_data.layer3_priority {
        render_layer(surface, pixels, LayerFlags::Layer3, &render_data.tileset_l3, &render_data.map.layers[2], 1, &render_data.palette.palette, &render_data.camera);
    }
}

fn render_sprites(target: &mut Surface, pixel_source: &mut Bitmap, map_sprites: &Vec<MapSprite>, priority: u32, camera: &Camera, sprites: &SpriteList) {
    for map_sprite in map_sprites {
        if !map_sprite.visible {
            continue;
        }
        if map_sprite.priority == priority {
            let x = (map_sprite.x - camera.lerp_x.floor()).floor() as i32;
            let y = (map_sprite.y - camera.lerp_y.floor()).floor() as i32;
            render_sprite(target, pixel_source, LayerFlags::Sprites.bits(), &sprites.get_sprite(map_sprite.sprite_index), map_sprite.frame, x, y, map_sprite.palette_offset);
        }
    }
}
