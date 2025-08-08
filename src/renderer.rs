use bitflags::bitflags;
use sdl3::pixels::{PixelFormat, PixelFormatEnum};
use sdl3::render::{Texture, TextureCreator, WindowCanvas};
use sdl3::{sys, Sdl};
use sdl3::sys::everything::SDL_ScaleMode;
use sdl3::ttf::Font;
use sdl3::video::WindowContext;
use crate::software_renderer::blit::{blit_surface_to_surface, SurfaceBlendOps};
use crate::software_renderer::clip::Rect;
use crate::software_renderer::draw::draw_box;
use crate::software_renderer::palette::Color;
use crate::software_renderer::surface::Surface;
use crate::software_renderer::text::{text_draw_to_surface, text_draw_to_surface_wrapped, TextDrawFlags};

const SCREEN_WIDTH: u32 = 256;
const SCREEN_HEIGHT: u32 = 224;

pub struct Renderer<'a> {
    pub scale: i32,
    scale_factor_x: f32,
    scale_factor_y: f32,

    aspect_ratio: f64,
    scale_linear: bool,
    vsync: bool,

    texture_creator: TextureCreator<WindowContext>,
    texture: Texture,
    scaled_texture: Texture,
    pub font: Font<'a>,

    pub target: Surface,
    pub canvas: WindowCanvas,
}

pub struct BoxRenderable {
    rect: Rect,
    color: Color,
    blend_op: SurfaceBlendOps,
}

impl BoxRenderable {
    pub fn new(x1: i32, y1: i32, x2: i32, y2: i32, color: Color, blend_op: SurfaceBlendOps) -> BoxRenderable {
        BoxRenderable {
            rect: Rect::new(x1, y1, x2, y2),
            color,
            blend_op,
        }
    }
}

pub struct TextRenderable {
    text: String,
    color: Color,
    flags: TextDrawFlags,
    wrap_width: i32,
    surface: Option<Surface>,
}

impl TextRenderable {
    pub fn new(text: String, color: Color, flags: TextDrawFlags, wrap_width: i32) -> TextRenderable {
        TextRenderable {
            text,
            color,
            flags,
            wrap_width,
            surface: None
        }
    }
}

bitflags! {
    #[derive(Clone, Default, Copy)]
    pub struct TextFlags: u32 {
        const AlignHStart = 0x01;
        const AlignHCenter = 0x02;
        const AlignHEnd = 0x04;
        const AlignVStart = 0x08;
        const AlignVCenter = 0x10;
        const AlignVEnd = 0x20;

        const ClampToTarget = 0x40;
    }
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
            scale_factor_x: target.width as f32 / output_width as f32,
            scale_factor_y: target.height as f32 / output_height as f32,

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

    pub fn render_text(&mut self, renderable: &mut TextRenderable, x: i32, y: i32, flags: TextFlags) {
        if renderable.surface.is_none() {
            if renderable.wrap_width > 0 {
                renderable.surface = Some(text_draw_to_surface_wrapped(&renderable.text.as_str(), &self.font, renderable.color, renderable.flags, renderable.wrap_width));
            } else {
                renderable.surface = Some(text_draw_to_surface(&renderable.text.as_str(), &self.font, renderable.color, renderable.flags));
            }
        }
        let surface = renderable.surface.as_mut().unwrap();

        let width = surface.width as i32;
        let mut dest_x = x;
        if flags.contains(TextFlags::AlignHCenter) {
            dest_x = x - width / 2;
        } else if flags.contains(TextFlags::AlignHEnd) {
            dest_x = x - width;
        }

        let height = surface.height as i32;
        let mut dest_y = y;
        if flags.contains(TextFlags::AlignVCenter) {
            dest_y = y - height / 2;
        } else if flags.contains(TextFlags::AlignVEnd) {
            dest_y = y - height;
        }

        if flags.contains(TextFlags::ClampToTarget) {
            dest_x = dest_x.max(0);
            dest_x = dest_x.min(self.target.width as i32 - surface.width as i32);
            dest_y = dest_y.max(0);
            dest_y = dest_y.min(self.target.height as i32 - surface.height as i32);
        }

        blit_surface_to_surface(&surface, &mut self.target, 0, 0, surface.width as i32, surface.height as i32, dest_x, dest_y, SurfaceBlendOps::Blend);
    }

    pub fn render_box(&mut self, rect: Rect, color: Color, blend_op: SurfaceBlendOps) {
        draw_box(&mut self.target, rect, color, blend_op);
    }

    pub fn window_to_target_coordinates(&self, x: f32, y: f32) -> (i32, i32) {
        (
            (x * self.scale_factor_x).floor() as i32,
            (y * self.scale_factor_y).floor() as i32,
        )
    }
}
