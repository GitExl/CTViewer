#![allow(dead_code)]

use std::path::Path;
use filesystem::filesystem::FileSystem;
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
use crate::destination::Destination;
use crate::renderer::Renderer;

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
mod renderer;
mod destination;

const UPDATES_PER_SECOND: f64 = 60.0;
const UPDATE_INTERVAL: f64 = 1.0 / UPDATES_PER_SECOND;



#[derive(Copy, Clone, PartialEq, Debug)]
pub enum GameEvent {
    GotoDestination {
        destination: Destination,
    },
}

/// Load and display Chrono Trigger game data.
#[derive(Parser, Debug)]
struct Args {
    /// Source data path.
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
    let mut renderer = Renderer::new(&sdl, args.scale, args.scale_linear, args.aspect_ratio, !args.no_vsync);

    let mut gamestate: Box<dyn GameStateTrait>;
    if args.scene > -1 {
        gamestate = Box::new(GameStateScene::new(&fs, &l10n, &mut renderer, args.scene as usize, 0, 0));
    } else if args.world > -1 {
        gamestate = Box::new(GameStateWorld::new(&fs, &l10n, &mut renderer, args.world as usize, 768, 512));
    } else {
        println!("No scene or world specified, loading scene 0x1.");
        gamestate = Box::new(GameStateScene::new(&fs, &l10n, &mut renderer, 1, 0, 0));
    }

    if args.dump {
        gamestate.dump();
    }

    let title = format!("Chrono Trigger - {}", gamestate.get_title(&l10n));
    renderer.set_title(title.as_str());

    let mut timer_loop = Timer::new();
    let mut timer_render = Timer::new();
    let mut timer_update = Timer::new();
    let mut timer_stats = Timer::new();
    let mut stat_render_time: f64 = 0.0;
    let mut stat_render_count: usize = 0;
    let mut stat_update_time: f64 = 0.0;
    let mut stat_update_count: usize = 0;

    let mut accumulator = 0.0;

    let mut event_pump = sdl.event_pump().unwrap();
    'running: loop {

        // Process input.
        for event in event_pump.poll_iter() {
            match event {
                Event::MouseMotion { x, y, .. } => {
                    let (x, y) = renderer.window_to_target_coordinates(x, y);
                    gamestate.mouse_motion(x, y);
                },
                Event::Quit {..} => break 'running,
                Event::KeyDown { keycode, .. } => {
                    match keycode {
                        Some(Keycode::Escape) => break 'running,
                        Some(Keycode::Backslash) => {
                            renderer.target.write_to_bmp((&"debug_output/screenshot.bmp").as_ref());
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

            let game_event = gamestate.tick(UPDATE_INTERVAL);
            if game_event.is_some() {
                match game_event.unwrap() {
                    GameEvent::GotoDestination { destination } => {
                        match destination {
                            Destination::Scene { index, x, y, .. } => {
                                gamestate = Box::new(GameStateScene::new(&fs, &l10n, &mut renderer, index, x, y));
                            },
                            Destination::World { index, x, y } => {
                                gamestate = Box::new(GameStateWorld::new(&fs, &l10n, &mut renderer, index, x, y));
                            },
                        };

                        let title = format!("Chrono Trigger - {}", gamestate.get_title(&l10n));
                        renderer.set_title(title.as_str());
                    },
                }
            }

            stat_update_time += timer_update.stop();
            stat_update_count += 1;

            accumulator -= UPDATE_INTERVAL;
        }
        let lerp = accumulator / UPDATE_INTERVAL;

        // Render a frame.
        timer_render.start();

        renderer.clear();
        gamestate.render(lerp, &mut renderer);
        renderer.copy_to_canvas();

        stat_render_time += timer_render.stop();
        stat_render_count += 1;

        renderer.present();

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
