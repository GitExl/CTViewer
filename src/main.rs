#![allow(dead_code)]

use std::path::Path;
use filesystem::filesystem::FileSystem;
use util::timer::Timer;
use crate::filesystem::backend_pc::{FileSystemBackendPc, FileSystemBackendPcMode};
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
use crate::sprites::sprite_assets::SpriteAssets;
use crate::sprites::sprite_state_list::SpriteStateList;
use crate::util::random::Random;
use crate::util::vec2df64::Vec2Df64;

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
mod scene_script;
mod facing;

const UPDATES_PER_SECOND: f64 = 60.0;
const UPDATE_INTERVAL: f64 = 1.0 / UPDATES_PER_SECOND;


#[derive(Copy, Clone)]
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

    /// Disable vertical sync.
    #[arg(long, default_value_t = false)]
    no_vsync: bool,
}

pub struct Context<'a> {
    fs: FileSystem,
    l10n: L10n,
    sprites_states: SpriteStateList,
    sprite_assets: SpriteAssets,
    render: Renderer<'a>,
    random: Random,
}

fn main() -> Result<(), String> {
    println!("SDL3: {}", sdl3::version::version());
    println!("SDL3 TTF: {}", sdl3::ttf::get_linked_version());

    let args = Args::parse();
    let fs = create_filesystem(args.path);
    let l10n = L10n::new("en", &fs);
    let sdl = sdl3::init().unwrap();
    let render = Renderer::new(&sdl, args.scale, args.scale_linear, args.aspect_ratio, !args.no_vsync);
    let sprite_assets = SpriteAssets::new(&fs);
    let sprites = SpriteStateList::new();
    let random = Random::new();

    let mut ctx = Context {
        fs,
        l10n,
        sprites_states: sprites,
        sprite_assets,
        render,
        random,
    };

    let mut gamestate: Box<dyn GameStateTrait>;
    if args.scene > -1 {
        gamestate = Box::new(GameStateScene::new(&mut ctx, args.scene as usize, Vec2Df64::new(0.0, 0.0)));
    } else if args.world > -1 {
        gamestate = Box::new(GameStateWorld::new(&mut ctx, args.world as usize, Vec2Df64::new(768.0, 512.0)));
    } else {
        println!("No scene or world specified, loading scene 0x1.");
        gamestate = Box::new(GameStateScene::new(&mut ctx, 1, Vec2Df64::new(0.0, 0.0)));
    }

    let title = format!("Chrono Trigger - {}", gamestate.get_title(&ctx));
    ctx.render.set_title(title.as_str());

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
                    let (x, y) = ctx.render.window_to_target_coordinates(x, y);
                    gamestate.mouse_motion(&ctx, x, y);
                },
                Event::Quit {..} => break 'running,
                Event::KeyDown { keycode, .. } => {
                    match keycode {
                        Some(Keycode::Escape) => break 'running,
                        Some(Keycode::Backspace) => {
                            ctx.sprite_assets.dump();
                            gamestate.dump(&ctx)
                        },
                        Some(Keycode::Backslash) => {
                            ctx.render.target.write_to_bmp((&"debug_output/screenshot.bmp").as_ref());
                            println!("Saved render target to debug_output/screenshot.bmp");
                        },
                        _ => {},
                    }
                },
                _ => {},
            }

            // Pass event on to gamestate.
            gamestate.event(&mut ctx, &event);
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

            let game_event = gamestate.tick(&mut ctx, UPDATE_INTERVAL);
            if game_event.is_some() {
                match game_event.unwrap() {
                    GameEvent::GotoDestination { destination } => {
                        match destination {
                            Destination::Scene { index, pos, .. } => {
                                gamestate = Box::new(GameStateScene::new(&mut ctx, index, pos.as_vec2d_f64()));
                            },
                            Destination::World { index, pos } => {
                                gamestate = Box::new(GameStateWorld::new(&mut ctx, index, pos.as_vec2d_f64()));
                            },
                        };

                        let title = format!("Chrono Trigger - {}", gamestate.get_title(&ctx));
                        ctx.render.set_title(title.as_str());
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

        ctx.render.clear();
        gamestate.render(&mut ctx, lerp);
        ctx.render.copy_to_canvas();

        stat_render_time += timer_render.stop();
        stat_render_count += 1;

        ctx.render.present();

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

fn create_filesystem(path: String) -> FileSystem {
    let src = Path::new(&path);

    // A directory is assumed to be the extracted version of the Steam resources.bin file.
    if src.is_dir() {
        let backend = FileSystemBackendPc::new(&src.into(), FileSystemBackendPcMode::FileSystem);
        return FileSystem::new(Box::new(backend), ParseMode::Pc);

    // Steam version resources.bin.
    } else if src.file_name().unwrap().to_ascii_lowercase() == "resources.bin" {
        let backend = FileSystemBackendPc::new(&src.into(), FileSystemBackendPcMode::ResourcesBin);
        return FileSystem::new(Box::new(backend), ParseMode::Pc);
    }

    // Any other file is assumed to be an SNES ROM image.
    let backend = FileSystemBackendSnes::new(&src);
    FileSystem::new(Box::new(backend), ParseMode::Snes)
}
