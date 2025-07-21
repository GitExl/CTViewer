use crate::map::MapChip;
use crate::map::MapChipFlags;
use crate::software_renderer::bitmap::Bitmap;
use crate::software_renderer::blit::blit_bitmap_to_surface;
use crate::software_renderer::blit::BitmapBlitFlags;
use crate::software_renderer::palette::Palette;
use crate::software_renderer::surface::Surface;

#[derive(Default, Clone)]
pub struct Tile {
    pub corners: [MapChip; 4],
}

pub struct ChipAnimFrame {
    pub src_chip: usize,
    pub duration: f64,
}

// Chip graphics animation data and state.
// Each animation covers 4 chips in the tileset.
pub struct ChipAnim {
    pub dest_chip: usize,
    pub frames: Vec<ChipAnimFrame>,

    pub frame: usize,
    pub timer: f64,
}

// A map's layers refer to these tiles. They are assembled from 4 "chips", corners of a tile.
// They refer to a chip from the chip graphics data, a palette and have some flags.
pub struct TileSet {
    pub index: usize,
    pub index_assembly: usize,

    pub tiles: Vec<Tile>,
    pub chip_bitmaps: Vec<Bitmap>,
    pub animated_chip_bitmaps: Vec<Bitmap>,
    pub chip_anims: Vec<ChipAnim>,
}

impl TileSet {
    pub fn dump(&self) {
        println!("Tileset {}", self.index);
        println!("  Assembly index: {}", self.index_assembly);
        println!("  Chips: {}, animated chips: {}", self.chip_bitmaps.len(), self.animated_chip_bitmaps.len());
        println!("  Tiles: {}", self.tiles.len());
        println!();

        if self.chip_anims.len() > 0 {
            for (anim_index, anim) in self.chip_anims.iter().enumerate() {
                println!("  Chip anim {} for chip {}", anim_index, anim.dest_chip);
                for (frame_index, frame) in anim.frames.iter().enumerate() {
                    println!("    Frame {}: chip {} for {:.3} seconds", frame_index, frame.src_chip, frame.duration);
                }
                println!();
            }
        }
    }

    pub fn tick(&mut self, delta: f64) {

        // Animate chip graphics.
        for anim in self.chip_anims.iter_mut() {

            let mut current_frame = &anim.frames[anim.frame];
            anim.timer += delta;
            if anim.timer < current_frame.duration {
                continue;
            }
            anim.timer -= current_frame.duration;

            // Advance an animation frame.
            anim.frame += 1;
            if anim.frame >= anim.frames.len() {
                anim.frame = 0;
            }

            // Copy the 4 chip graphics.
            current_frame = &anim.frames[anim.frame];
            for chip_index in 0..4 {
                let dest_chip_index = anim.dest_chip + chip_index;
                if dest_chip_index >= self.chip_bitmaps.len() {
                    continue;
                }

                let src_chip_index = current_frame.src_chip + chip_index;
                if src_chip_index >= self.animated_chip_bitmaps.len() {
                    continue;
                }

                self.chip_bitmaps[dest_chip_index].data.copy_from_slice(&self.animated_chip_bitmaps[src_chip_index].data);
            }
        }
    }

    // Render this tileset's chips to a surface, for debugging purposes.
    pub fn render_chips_to_surface(self: &TileSet, chips: &Vec<Bitmap>) -> Surface {
        const CHIPS_PER_ROW: usize = 16;

        let width = CHIPS_PER_ROW * 8;
        let mut height = (chips.len() as f64 / CHIPS_PER_ROW as f64).ceil() as usize * 8;
        if height == 0 {
            height = 8;
        }

        let mut palette = Palette::new(16);
        for i in 0..16 {
            let v = (i * 15) as u8;
            palette.colors[i] = [v, v, v, 0xFF];
        }

        let mut surface = Surface::new(width as u32, height as u32);
        surface.fill(palette.colors[0]);

        let mut x = 0;
        let mut y = 0;
        for chip in chips.iter() {
            blit_bitmap_to_surface(&chip, &mut surface, 0, 0, 8, 8, x, y, &palette, 0, BitmapBlitFlags::empty());
            x += 8;
            if x >= width as i32 {
                x = 0;
                y += 8;
            }
        }

        surface
    }

    // Render this tileset's tiles to a surface, for debugging purposes.
    pub fn render_tiles_to_surface(self: &TileSet, palette: &Palette) -> Surface {
        const TILES_PER_ROW: usize = 16;

        let width = TILES_PER_ROW * 16;
        let mut height = (self.tiles.len() as f64 / TILES_PER_ROW as f64).ceil() as usize * 16;
        if height == 0 {
            height = 16;
        }

        let mut surface = Surface::new(width as u32, height as u32);
        surface.fill(palette.colors[0]);

        let mut x = 0;
        let mut y = 0;
        for tile in &self.tiles {

            for corner_index in 0..4 {
                let corner = &tile.corners[corner_index];
                if corner.chip >= self.chip_bitmaps.len() {
                    continue;
                }

                let mut render_flags = BitmapBlitFlags::SKIP_0;
                if corner.flags.contains(MapChipFlags::FLIP_X) {
                    render_flags |= BitmapBlitFlags::FLIP_X;
                }
                if corner.flags.contains(MapChipFlags::FLIP_Y) {
                    render_flags |= BitmapBlitFlags::FLIP_Y;
                }

                let (cx, cy) = match corner_index {
                    0 => (x, y),
                    1 => (x + 8, y),
                    2 => (x, y + 8),
                    3 => (x + 8, y + 8),
                    _ => (x, y),
                };
                blit_bitmap_to_surface(&self.chip_bitmaps[corner.chip], &mut surface, 0, 0, 8, 8, cx, cy, palette, corner.palette, render_flags);
            }

            x += 16;
            if x >= width as i32 {
                x = 0;
                y += 16;
            }
        }

        surface
    }
}
