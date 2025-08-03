#![allow(dead_code)]

use std::path::Path;
use filesystem::filesystem::FileSystem;
use software_renderer::surface::Surface;
use util::timer::Timer;
use crate::filesystem::backend_pc::FileSystemBackendPc;
use crate::filesystem::backend_snes::FileSystemBackendSnes;
use crate::filesystem::filesystem::ParseMode;
use crate::gamestate::gamestate::GameStateTrait;
use crate::gamestate::gamestate_scene::GameStateScene;
use crate::gamestate::gamestate_world::GameStateWorld;
use crate::l10n::L10n;
use clap::Parser;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::pixels::{PixelFormat, PixelFormatEnum};
use sdl3::render::ScaleMode;
use sdl3::sys;
use crate::software_renderer::blit::{blit_surface_to_surface, SurfaceBlendOps};
use crate::software_renderer::text::{text_draw_to_surface, TextRenderFlags};

mod actor;
mod camera;
mod filesystem;
mod game_palette;
mod map_renderer;
mod map;
mod palette_anim;
mod software_renderer;
mod sprites;
mod tileset;
mod util;
mod world;
mod scene;
mod l10n;
mod gamestate;

const SCREEN_WIDTH: u32 = 256;
const SCREEN_HEIGHT: u32 = 224;

const UPDATES_PER_SECOND: f64 = 60.0;
const UPDATE_INTERVAL: f64 = 1.0 / UPDATES_PER_SECOND;

/// Load and display Chrono Trigger game data.
#[derive(Parser, Debug)]
struct Args {
    /// Source data path.
    #[arg(short, long)]
    path: String,

    /// Index of the world to load.
    #[arg(short, long, default_value_t = -1)]
    world: isize,

    /// Index of the scene to load.
    #[arg(short, long, default_value_t = -1)]
    scene: isize,

    /// Display scale.
    #[arg(long, default_value_t = -1)]
    scale: i32,

    /// Scale output using linear scaling.
    #[arg(long, default_value_t = false)]
    scale_linear: bool,

    /// Set the output aspect ratio.
    #[arg(short, long, default_value_t = 4.0 / 3.0)]
    aspect_ratio: f64,

    /// Dump information and debug data.
    #[arg(short, long, default_value_t = false)]
    dump: bool,

    /// Disable vertical sync.
    #[arg(long, default_value_t = false)]
    no_vsync: bool,
}

fn main() -> Result<(), String> {
    println!("SDL3: {}", sdl3::version::version());
    println!("SDL3 TTF: {}", sdl3::ttf::get_linked_version());

    let args = Args::parse();

    let src = Path::new(&args.path);
    let fs;
    if src.is_dir() {
        let backend = FileSystemBackendPc::new(&src.into());
        fs = FileSystem::new(Box::new(backend), ParseMode::Pc);
    } else {
        let backend = FileSystemBackendSnes::new(&src);
        fs = FileSystem::new(Box::new(backend), ParseMode::Snes);
    }

    let l10n = L10n::new("en", &fs);

    let sdl = sdl3::init().unwrap();
    let video = sdl.video().unwrap();

    // Font setup.
    let ttf_context = sdl3::ttf::init().unwrap();
    let mut font = ttf_context.load_font(&"data/chronotype/ChronoType.ttf", 16.0).unwrap();
    font.set_style(sdl3::ttf::FontStyle::NORMAL);
    
    // Auto-adjust scale to display size.
    let output_scale = if args.scale < 1 {
        let current_mode = video.displays().unwrap()[0].get_mode().unwrap();
        let scale_w = (current_mode.w as f64 / (SCREEN_HEIGHT as f64 * args.aspect_ratio)).floor();
        let scale_h = (current_mode.h as f64 / SCREEN_HEIGHT as f64).floor();
        scale_w.min(scale_h.max(1.0)) as u32
    } else {
        args.scale as u32
    };

    // Calculate final output size.
    let mut output_width = (SCREEN_HEIGHT as f64 * args.aspect_ratio).ceil() as u32 * output_scale;
    output_width += output_width % 4;
    let output_height = SCREEN_HEIGHT * output_scale;
    println!("Display size is {}x{}", output_width, output_height);

    let mut canvas = video.window_and_renderer("Chrono Trigger", output_width, output_height).unwrap();
    unsafe { sys::render::SDL_SetRenderVSync(canvas.raw(), if args.no_vsync { 0 } else { 1 }); }

    // Internal SNES rendering target.
    let mut target_surface = Surface::new(SCREEN_WIDTH, SCREEN_HEIGHT);

    // Create a surface to copy the internal output to. This is used as the source for the
    // initial integer scaling.
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormat::from(PixelFormatEnum::ABGR8888), SCREEN_WIDTH, SCREEN_HEIGHT)
        .unwrap();
    texture.set_scale_mode(if args.scale_linear { ScaleMode::Linear } else { ScaleMode::Nearest });

    // Create a surface to scale the output to. This will be scaled to match the final output size
    // linearly to mask uneven pixel widths.
    let mut scaled_texture = texture_creator
        .create_texture_target(PixelFormat::from(PixelFormatEnum::ABGR8888), SCREEN_WIDTH * output_scale, SCREEN_HEIGHT * output_scale)
        .unwrap();
    scaled_texture.set_scale_mode(ScaleMode::Linear);

    let mut gamestate: Box<dyn GameStateTrait>;
    if args.scene > -1 {
        gamestate = Box::new(GameStateScene::new(&fs, &l10n, &mut target_surface, args.scene as usize));
    } else if args.world > -1 {
        gamestate = Box::new(GameStateWorld::new(&fs, &l10n, &mut target_surface, args.world as usize));
    } else {
        panic!("Must load a scene or a world.");
    }

    if args.dump {
        gamestate.dump();
    }

    let title = format!("Chrono Trigger - {}", gamestate.get_title(&l10n));
    canvas.window_mut().set_title(title.as_str()).unwrap();

    let mut timer_loop = Timer::new();
    let mut timer_render = Timer::new();
    let mut timer_update = Timer::new();
    let mut timer_stats = Timer::new();
    let mut stat_render_time: f64 = 0.0;
    let mut stat_render_count: usize = 0;
    let mut stat_update_time: f64 = 0.0;
    let mut stat_update_count: usize = 0;
    let mut stats_surface = Surface::new(32, 32);

    let mut accumulator = 0.0;

    let mut event_pump = sdl.event_pump().unwrap();
    'running: loop {

        // Process input.
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => break 'running,
                Event::KeyDown { keycode, .. } => {
                    match keycode {
                        Some(Keycode::Escape) => break 'running,
                        Some(Keycode::Backslash) => {
                            target_surface.write_to_bmp((&"debug_output/screenshot.bmp").as_ref());
                            println!("Saved render target to debug_output/screenshot.bmp");
                        },
                        _ => {},
                    }
                },
                _ => {},
            }

            // Pass event on to gamestate.
            gamestate.event(&event);
        }

        // Update state.
        let mut update_time = timer_loop.stop();
        timer_loop.start();
        if update_time > UPDATE_INTERVAL * 3.0 {
            update_time = UPDATE_INTERVAL * 3.0;
        }
        accumulator += update_time;
        while accumulator > UPDATE_INTERVAL {
            timer_update.start();

            gamestate.tick(UPDATE_INTERVAL);

            stat_update_time += timer_update.stop();
            stat_update_count += 1;

            accumulator -= UPDATE_INTERVAL;
        }
        let lerp = accumulator / UPDATE_INTERVAL;

        // Render a frame.
        timer_render.start();

        target_surface.fill([0, 0, 0, 0xFF]);
        gamestate.render(lerp, &mut target_surface);
        blit_surface_to_surface(&stats_surface, &mut target_surface, 0, 0, stats_surface.width as i32, stats_surface.height as i32, 255 - stats_surface.width as i32, 1, SurfaceBlendOps::Blend);

        // Linear scaling can output the scene directly to the window.
        if args.scale_linear {
            texture.with_lock(None, |buffer: &mut [u8], _: usize| {
                buffer.copy_from_slice(&target_surface.data);
            }).unwrap();
            canvas.copy(&texture, None, None).unwrap();

        // Nearest scaling takes care to first scale the scene up to the nearest integer size.
        // Then scales that to the desired aspect ratio linearly.
        } else {
            texture.with_lock(None, |buffer: &mut [u8], _: usize| {
                buffer.copy_from_slice(&target_surface.data);
            }).unwrap();
            canvas.with_texture_canvas(&mut scaled_texture, |texture_canvas| {
                texture_canvas.copy(&texture, None, None).unwrap();
            }).unwrap();
            canvas.copy(&scaled_texture, None, None).unwrap();
        }

        stat_render_time += timer_render.stop();
        stat_render_count += 1;

        canvas.present();

        // Output stats.
        if timer_stats.elapsed() >= 1.0 {
            timer_stats.start();

            println!("r {:.1} ns, u {:.1} ns, {} FPS",
                (stat_render_time / stat_render_count as f64) * 1000000.0,
                (stat_update_time / stat_update_count as f64) * 1000000.0,
                stat_render_count,
            );

            stats_surface = text_draw_to_surface(format!("{} FPS", stat_render_count,).as_str(), &font, [223, 223, 223, 255], TextRenderFlags::SHADOW);

            stat_render_time = 0.0;
            stat_render_count = 0;
            stat_update_time = 0.0;
            stat_update_count = 0;
        }
    };

    Ok(())
}
