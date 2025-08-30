use std::collections::HashMap;
use std::path::Path;
use crate::filesystem::filesystem::FileSystem;
use crate::software_renderer::bitmap::Bitmap;
use crate::software_renderer::blit::blit_bitmap_to_surface;
use crate::software_renderer::blit::BitmapBlitFlags;
use crate::software_renderer::palette::Palette;
use crate::software_renderer::surface::Surface;
use super::sprite_anim::{SpriteAnimFrame, SpriteAnimSet};
use super::sprite_assembly::SpriteAssembly;

// Keys for world sprite data.
pub const WORLD_SPRITE_INDEX: usize = 0xFFFFFF;
pub const WORLD_ANIM_SET_INDEX: usize = 0xFFFFFF;
pub const WORLD_ASSEMBLY_SET_INDEX: usize = 0xFFFFFF;

pub struct Sprite {
    pub index: usize,
    pub tiles: Bitmap,
    pub assembly: SpriteAssembly,
    pub palette: Palette,
    pub anim_set_index: usize,
}

#[derive(Clone)]
pub struct SpriteState {
    pub map_sprite_index: usize,

    pub sprite_index: usize,
    pub sprite_frame: usize,
    pub palette_offset: usize,
    pub direction: usize,
    pub priority: u32,
    pub enabled: bool,

    pub anim_index: usize,
    pub anim_frame: usize,
    pub anim_timer: f64,
    pub animating: bool,
}

impl SpriteState {
    pub fn new() -> SpriteState {
        SpriteState {
            map_sprite_index: 0,

            sprite_index: 0,
            sprite_frame: 0,
            palette_offset: 0,
            direction: 0,
            priority: 0,
            enabled: false,

            anim_index: 0,
            anim_frame: 0,
            anim_timer: 0.0,
            animating: false,
        }
    }
}

pub struct SpriteList<'a> {
    fs: &'a FileSystem,
    sprites: HashMap<usize, Sprite>,
    anim_sets: HashMap<usize, SpriteAnimSet>,
    sprite_states: Vec<SpriteState>,
}

impl SpriteList<'_> {
    pub fn new(fs: &'_ FileSystem) -> SpriteList<'_> {
        let manager = SpriteList {
            fs,
            sprites: HashMap::new(),
            anim_sets: fs.read_sprite_animations(),
            sprite_states: Vec::new(),
        };

        manager
    }

    pub fn add_sprite_state(&mut self) -> &mut SpriteState {
        self.sprite_states.push(SpriteState::new());
        let index = self.sprite_states.len() - 1;
        self.sprite_states.get_mut(index).unwrap()
    }

    pub fn get_sprite(&self, sprite_index: usize) -> &Sprite {
        self.sprites.get(&sprite_index).unwrap()
    }

    pub fn get_state(&self, actor_index: usize) -> &SpriteState {
        self.sprite_states.get(actor_index).unwrap()
    }

    pub fn get_state_mut(&mut self, actor_index: usize) -> &mut SpriteState {
        self.sprite_states.get_mut(actor_index).unwrap()
    }

    pub fn set_animation(&mut self, actor_index: usize, anim_index: usize) {
        let state = &self.sprite_states[actor_index];
        let frame = self.get_frame_for_animation(state.sprite_index, anim_index, 0);
        let sprite_frame = frame.sprite_frames[state.direction];

        let state = &mut self.sprite_states[actor_index];
        state.anim_index = anim_index;
        state.anim_frame = 0;
        state.animating = true;
        state.sprite_frame = sprite_frame;
        state.anim_timer = 0.0;
    }

    pub fn set_direction(&mut self, actor_index: usize, direction: usize) {
        let state = &self.sprite_states[actor_index];
        let frame = self.get_frame_for_animation(state.sprite_index, state.anim_index, state.anim_frame);
        let sprite_frame = frame.sprite_frames[direction];

        let state = &mut self.sprite_states[actor_index];
        state.direction = direction;
        state.sprite_frame = sprite_frame;
    }

    pub fn set_frame(&mut self, actor_index: usize, frame_index: usize) {
        let state = &mut self.sprite_states[actor_index];
        state.anim_frame = 0;
        state.animating = false;
        state.sprite_frame = frame_index;
        state.anim_timer = 0.0;
    }

    fn get_frame_for_animation(&self, sprite_index: usize, anim_index: usize, frame_index: usize) -> &SpriteAnimFrame {
        let sprite = self.sprites.get(&sprite_index).unwrap();
        let anim_set = self.anim_sets.get(&sprite.anim_set_index).unwrap();

        let anim = if anim_set.anims.len() <= anim_index {
            println!("Warning: sprite {} does not have animation {}. Using animation 0.", sprite_index, anim_index);
            &anim_set.anims[0]
        } else {
            &anim_set.anims[anim_index]
        };

        if anim.frames.len() == 0 {
            println!("Warning: sprite {} animation {} does not have frame {}. Using frame 0.", sprite_index, anim_index, frame_index);
            &anim.frames[0]
        } else {
            &anim.frames[frame_index]
        }
    }

    // Updates sprite state.
    pub fn tick_state(&mut self, delta: f64, actor_index: usize) {
        let state = self.sprite_states.get_mut(actor_index).unwrap();
        if !state.animating {
            return;
        }

        // Get the current visible animation frame through the sprite's animation set.
        let sprite = self.sprites.get(&state.sprite_index).unwrap();
        let anim_set = self.anim_sets.get(&sprite.anim_set_index).unwrap();
        let anim = &anim_set.anims[state.anim_index];
        let frame = &anim.frames[state.anim_frame];

        // 0-duration frames show indefinitely.
        if frame.duration == 0.0 {
            return;
        }

        // Advance animation time.
        state.anim_timer += delta;
        if state.anim_timer < frame.duration {
            return;
        }

        // Advance to the next frame.
        state.anim_timer -= frame.duration;
        state.anim_frame += 1;
        if state.anim_frame >= anim.frames.len() {
            state.anim_frame = 0;
        }
        state.sprite_frame = anim.frames[state.anim_frame].sprite_frames[state.direction];
    }

    // Load a sprite for future use.
    pub fn load_sprite(&mut self, sprite_index: usize) {
        if self.sprites.contains_key(&sprite_index) {
            return;
        }

        let info = self.fs.read_sprite_header(sprite_index);
        let assembly = self.fs.read_sprite_assembly(info.assembly_index, &info);
        let palette = self.fs.read_sprite_palette(info.palette_index).unwrap();
        let tiles = self.fs.read_sprite_tiles(info.bitmap_index, assembly.chip_max);

        let sprite = Sprite {
            index: sprite_index,
            tiles,
            assembly,
            palette,
            anim_set_index: info.anim_index,
        };
        self.sprites.insert(sprite_index, sprite);
    }

    // Loads the generic world sprite used by world maps.
    pub fn load_world_sprite(&mut self, world_index: usize, sprite_graphics: [usize; 4], palette: &Palette) {
        if self.sprites.contains_key(&WORLD_SPRITE_INDEX) {
            return;
        }

        // Read animation, assembly and graphics data.
        let (assembly, anim_set) = self.fs.read_world_sprites();
        self.anim_sets.insert(WORLD_ANIM_SET_INDEX, anim_set);
        let tiles = self.fs.read_world_sprite_tiles_all(world_index, sprite_graphics);

        self.sprites.insert(WORLD_SPRITE_INDEX, Sprite {
            index: WORLD_SPRITE_INDEX,
            tiles,
            assembly,
            palette: palette.clone(),
            anim_set_index: WORLD_ANIM_SET_INDEX,
        });
    }

    // Replace part of the world sprite tile graphics with new data.
    pub fn replace_world_sprite_tiles(&mut self, world_index: usize, tiles_index: usize, offset: usize) {
        let sprite = self.sprites.get_mut(&WORLD_SPRITE_INDEX).unwrap();
        self.fs.read_world_sprite_tiles(world_index, tiles_index, offset, &mut sprite.tiles.data);
    }

    // Read player character sprites.
    pub fn load_world_player_sprites(&mut self, characters: [usize; 3]) {
        let sprite = self.sprites.get_mut(&WORLD_SPRITE_INDEX).unwrap();
        let src_pixels = self.fs.read_world_player_sprite_tiles();
        let dest_pixels = &mut sprite.tiles.data;

        // Copy player world sprites from external bitmap data into sprite bitmap data.
        // Only 3 characters these are loaded at a time.
        for (index, src_index) in characters.iter().enumerate() {

            // Copy regular sprites to 0x4000 in tile memory + the start of the player character.
            // Each row is 0x80 bytes, 16 rows is 0x800 bytes.
            let src_offset = src_index * 0x800;
            let dest_offset = 0x4000 + (index * 0x800);
            dest_pixels[dest_offset..dest_offset + 0x800].copy_from_slice(&src_pixels[src_offset..src_offset + 0x800]);

            // Copy the loose idle sprite.
            for row in 0..16 {
                let src_offset = 0x3800 + src_index * 0x10 + (row * 0x80);
                let dest_offset = 0x5800 + src_index * 0x10 + (row * 0x80);
                dest_pixels[dest_offset..dest_offset + 0x10].copy_from_slice(&src_pixels[src_offset..src_offset + 0x10]);
            }
        }
    }

    // Read epoch sprites.
    pub fn load_world_epoch_sprites(&mut self, mode: usize) {
        let sprite = self.sprites.get_mut(&WORLD_SPRITE_INDEX).unwrap();
        let src_pixels = self.fs.read_world_epoch_sprite_tiles();
        let dest_pixels = &mut sprite.tiles.data;

        // Copy walking sprites to 8192 + start of player character.
        let src_offset = mode * 0x800;
        let dest_offset = 0x6000 + (mode * 0x1000);
        dest_pixels[dest_offset..dest_offset + 0x1000].copy_from_slice(&src_pixels[src_offset..src_offset + 0x1000]);
    }

    // Dump world sprite tiles to disk.
    pub fn dump_world_sprite_graphics(&mut self) {
        let sprite = self.sprites.get_mut(&WORLD_SPRITE_INDEX).unwrap();
        let mut surface = Surface::new(128, 256);
        blit_bitmap_to_surface(&sprite.tiles, &mut surface, 0, 0, 128, 256, 0, 0, &sprite.palette, 0, BitmapBlitFlags::default());
        surface.write_to_bmp(Path::new("debug_output/world_sprite_graphics.bmp"));
    }
}
