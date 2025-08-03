use sdl3::pixels::{PixelFormat, PixelFormatEnum};
use sdl3::render::{Texture, TextureCreator, WindowCanvas};
use sdl3::{sys, Sdl};
use sdl3::sys::everything::SDL_ScaleMode;
use sdl3::ttf::Font;
use sdl3::video::WindowContext;
use crate::software_renderer::surface::Surface;

const SCREEN_WIDTH: u32 = 256;
const SCREEN_HEIGHT: u32 = 224;

pub struct Renderer<'a> {
    pub scale: i32,
    scale_linear: bool,
    aspect_ratio: f64,
    vsync: bool,

    texture_creator: TextureCreator<WindowContext>,
    texture: Texture,
    scaled_texture: Texture,
    pub font: Font<'a>,

    pub target: Surface,
    pub canvas: WindowCanvas,
}

impl<'a> Renderer<'a> {
    pub fn new(sdl: &Sdl, scale: i32, scale_linear: bool, aspect_ratio: f64, vsync: bool) -> Renderer {
        let video = sdl.video().unwrap();

        // Font setup.
        let ttf_context = sdl3::ttf::init().unwrap();
        let mut font = ttf_context.load_font(&"data/chronotype/ChronoType.ttf", 16.0).unwrap();
        font.set_style(sdl3::ttf::FontStyle::NORMAL);

        // Auto-adjust scale to display size.
        let output_scale = if scale < 1 {
            let current_mode = video.displays().unwrap()[0].get_mode().unwrap();
            let scale_w = (current_mode.w as f64 / (SCREEN_HEIGHT as f64 * aspect_ratio)).floor();
            let scale_h = (current_mode.h as f64 / SCREEN_HEIGHT as f64).floor();
            scale_w.min(scale_h.max(1.0)) as u32
        } else {
            scale as u32
        };

        // Calculate scaled output size.
        let mut output_width = (SCREEN_HEIGHT as f64 * aspect_ratio).ceil() as u32 * output_scale;
        output_width += output_width % 4;
        let output_height = SCREEN_HEIGHT * output_scale;
        println!("Display size is {}x{}", output_width, output_height);

        let canvas = video.window_and_renderer("Chrono Trigger", output_width, output_height).unwrap();
        unsafe { sys::render::SDL_SetRenderVSync(canvas.raw(), if vsync { 1 } else { 0 }); }

        // Internal SNES rendering target.
        let target = Surface::new(SCREEN_WIDTH, SCREEN_HEIGHT);

        let texture_creator = canvas.texture_creator();

        // Create a surface to copy the internal output to. This is used as the source for the
        // initial integer scaling.
        let texture = texture_creator
            .create_texture_streaming(PixelFormat::from(PixelFormatEnum::ABGR8888), SCREEN_WIDTH, SCREEN_HEIGHT)
            .unwrap();
        unsafe { sys::render::SDL_SetTextureScaleMode(texture.raw(), if scale_linear { SDL_ScaleMode::LINEAR } else { SDL_ScaleMode::NEAREST }); }

        // Create a surface to scale the output to. This will be scaled to match the final output size
        // linearly to mask uneven pixel widths.
        let scaled_texture = texture_creator
            .create_texture_target(PixelFormat::from(PixelFormatEnum::ABGR8888), SCREEN_WIDTH * output_scale, SCREEN_HEIGHT * output_scale)
            .unwrap();
        unsafe { sys::render::SDL_SetTextureScaleMode(scaled_texture.raw(), SDL_ScaleMode::LINEAR); }

        Renderer {
            scale,
            scale_linear,
            aspect_ratio,
            vsync,

            texture_creator,
            texture,
            scaled_texture,
            font,

            target,
            canvas,
        }
    }

    pub fn set_title(&mut self, title: &str) {
        self.canvas.window_mut().set_title(title).unwrap();
    }

    pub fn copy_to_canvas(&mut self) {

        // Linear scaling can output the scene directly to the window.
        if self.scale_linear {
            self.texture.with_lock(None, |buffer: &mut [u8], _: usize| {
                buffer.copy_from_slice(&self.target.data);
            }).unwrap();
            self.canvas.copy(&self.texture, None, None).unwrap();

        // Nearest scaling takes care to first scale the scene up to the nearest integer size.
        // Then scales that to the desired aspect ratio linearly.
        } else {
            self.texture.with_lock(None, |buffer: &mut [u8], _: usize| {
                buffer.copy_from_slice(&self.target.data);
            }).unwrap();
            self.canvas.with_texture_canvas(&mut self.scaled_texture, |texture_canvas| {
                texture_canvas.copy(&self.texture, None, None).unwrap();
            }).unwrap();
            self.canvas.copy(&self.scaled_texture, None, None).unwrap();
        }
    }

    pub fn present(&mut self) {
        self.canvas.present();
    }

    pub fn clear(&mut self) {
        self.target.fill([0, 0, 0, 0xFF]);
    }
}
