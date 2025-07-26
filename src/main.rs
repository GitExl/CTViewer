#![allow(dead_code)]

use std::path::Path;
use filesystem::filesystem::FileSystem;
use software_renderer::surface::Surface;
use sdl2::{event::Event, keyboard::Keycode, pixels::PixelFormatEnum, rect::Rect};
use util::timer::Timer;
use crate::filesystem::backend_pc::FileSystemBackendPc;
use crate::filesystem::backend_snes::FileSystemBackendSnes;
use crate::filesystem::filesystem::ParseMode;
use crate::gamestate::gamestate::GameStateTrait;
use crate::gamestate::gamestate_scene::GameStateScene;
use crate::gamestate::gamestate_world::GameStateWorld;
use crate::l10n::L10n;
use clap::Parser;

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

    /// Aspect ratio.
    #[arg(short, long, default_value_t = 4.0 / 3.0)]
    aspect_ratio: f64,

    /// Dump information and debug data.
    #[arg(short, long, default_value_t = false)]
    dump: bool,
}

fn main() -> Result<(), String> {
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

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    // Auto-adjust scale to display size.
    let output_scale = if args.scale < 1 {
        let current_mode = video_subsystem.current_display_mode(0)?;
        let scale_w = (current_mode.w as f64 / (SCREEN_HEIGHT as f64 * args.aspect_ratio)).floor();
        let scale_h = (current_mode.h as f64 / SCREEN_HEIGHT as f64).floor();
        scale_w.min(scale_h.max(1.0)) as u32
    } else {
        args.scale as u32
    };

    // Calculate final output size.
    let output_width = (SCREEN_HEIGHT as f64 * args.aspect_ratio).ceil() as u32 * output_scale;
    let output_height = SCREEN_HEIGHT * output_scale;
    println!("Display size is {}x{}", output_width, output_height);

    let window = video_subsystem.window("Chrono Trigger", output_width, output_height)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window
        .into_canvas()
        .accelerated()
        .present_vsync()
        .build()
        .unwrap();

    let size = canvas.window().size();
    let canvas_rect = Some(Rect::new(0, 0, size.0, size.1));

    // Internal SNES rendering target.
    let mut target_surface = Surface::new(SCREEN_WIDTH, SCREEN_HEIGHT);

    // Create a surface to copy the internal output to. This is used as the source for the
    // initial integer scaling.
    let texture_creator = canvas.texture_creator();
    sdl2::hint::set("SDL_RENDER_SCALE_QUALITY", "nearest");
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::ABGR8888, SCREEN_WIDTH, SCREEN_HEIGHT)
        .unwrap();

    // Create a surface to scale the output to. This will be scaled to match the final output size
    // linearly to mask uneven pixel widths.
    sdl2::hint::set("SDL_RENDER_SCALE_QUALITY", "linear");
    let mut scaled_texture = texture_creator
        .create_texture_target(PixelFormatEnum::ABGR8888, SCREEN_WIDTH * output_scale, SCREEN_HEIGHT * output_scale)
        .unwrap();

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

    let mut timer_loop = Timer::new(sdl_context.timer()?);
    let mut timer_render = Timer::new(sdl_context.timer()?);
    let mut timer_update = Timer::new(sdl_context.timer()?);
    let mut timer_stats = Timer::new(sdl_context.timer()?);
    let mut stat_render_time: f64 = 0.0;
    let mut stat_render_count: usize = 0;
    let mut stat_update_time: f64 = 0.0;
    let mut stat_update_count: usize = 0;

    let mut accumulator = 0.0;

    let mut event_pump = sdl_context.event_pump()?;
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

        target_surface.clear();
        gamestate.render(lerp, &mut target_surface);

        // Scale the scene texture up to the integer scale factor. The texture has nearest filtering
        // set so this will keep the pixels crisp.
        texture.with_lock(None, |buffer: &mut [u8], _: usize| {
            buffer.copy_from_slice(&target_surface.data);
        })?;
        // Copy the scaled scene texture to the window texture. This will resize with linear
        // filtering to match the correct aspect ratio without uneven pixel sizes.
        canvas.with_texture_canvas(&mut scaled_texture, |texture_canvas| {
            texture_canvas.copy(&texture, None, None).unwrap();
        }).unwrap();
        canvas.copy(&scaled_texture, None, canvas_rect)?;

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
            stat_render_time = 0.0;
            stat_render_count = 0;
            stat_update_time = 0.0;
            stat_update_count = 0;
        }
    };

    Ok(())
}
