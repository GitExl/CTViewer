#![allow(dead_code)]

use std::path::Path;
use filesystem::filesystem::FileSystem;
use util::timer::Timer;
use crate::filesystem::backend_pc::{FileSystemBackendPc, FileSystemBackendPcMode};
use crate::filesystem::backend_snes::FileSystemBackendSnes;
use crate::gamestate::gamestate::GameStateTrait;
use crate::gamestate::gamestate_scene::GameStateScene;
use crate::gamestate::gamestate_world::GameStateWorld;
use crate::l10n::L10n;
use clap::Parser;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use crate::destination::Destination;
use crate::facing::Facing;
use crate::memory::Memory;
use crate::renderer::Renderer;
use crate::screen_fade::ScreenFade;
use assets::Assets;
use crate::input::{InputAction, InputManager};
use crate::party::party::Party;
use crate::sprites::sprite_state_list::SpriteStateList;
use crate::text_processor::TextProcessor;
use crate::ui_theme::UiTheme;
use crate::util::random::Random;
use crate::util::vec2df64::Vec2Df64;

mod camera;
mod filesystem;
mod game_palette;
mod map_renderer;
mod map;
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
mod ui_theme;
mod text_processor;
mod screen_fade;
mod next_destination;
mod memory;
mod party;
mod world_script;
mod shared_op;
mod music_list;
mod sound_list;
pub mod assets;
mod scroll_state;
mod input;

const UPDATES_PER_SECOND: f64 = 60.0;
const UPDATE_INTERVAL: f64 = 1.0 / UPDATES_PER_SECOND;

#[derive(Copy, Clone, PartialEq)]
pub enum GameMode {
    Pc,
    Snes,
}

#[derive(Copy, Clone)]
pub enum GameEvent {
    GotoDestination {
        destination: Destination,
        fade_in: bool,
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

    /// Display scale factor, integer.
    #[arg(long, default_value_t = -1)]
    scale: i32,

    /// Scale output using linear scaling.
    #[arg(long, default_value_t = false)]
    scale_linear: bool,

    /// Set the display aspect ratio.
    // As used by Mesen in Auto aspect ratio mode.
    #[arg(short, long, default_value_t = 1.306122448979592, value_name = "RATIO")]
    display_aspect_ratio: f64,

    /// Set the pixel aspect ratio.
    #[arg(short, long, default_value_t = (256.0 * (8.0 / 7.0)) / 256.0, value_name = "RATIO")]
    pixel_aspect_ratio: f64,

    /// Disable vertical sync.
    #[arg(long, default_value_t = false)]
    no_vsync: bool,

    /// The user interface theme index, from 0 to 7.
    #[arg(short, long, default_value_t = 0, value_name = "THEME")]
    ui_theme: usize,
}

pub struct Context<'a> {
    fs: FileSystem,
    l10n: L10n<'a>,
    sprite_states: SpriteStateList,
    assets: Assets,
    render: Renderer<'a>,
    random: Random,
    ui_theme: UiTheme,
    memory: Memory,
    party: Party,
    text_processor: TextProcessor,
    screen_fade: ScreenFade,
    mode: GameMode,
    input: InputManager,
    debug_mode: bool,
}

fn main() -> Result<(), String> {
    println!("SDL3: {}", sdl3::version::version());
    println!("SDL3 TTF: {}", sdl3::ttf::get_linked_version());

    let args = Args::parse();

    let fs = create_filesystem(args.path);
    let l10n = L10n::new("it", &fs);
    let sdl = sdl3::init().unwrap();
    let render = Renderer::new(&sdl, args.scale, args.scale_linear, args.pixel_aspect_ratio, args.display_aspect_ratio, !args.no_vsync);
    let assets = Assets::new(&fs);
    let sprite_states = SpriteStateList::new();
    let random = Random::new();
    let ui_theme = fs.read_ui_theme(args.ui_theme);
    let screen_fade = ScreenFade::new(0.0);
    let mode = fs.mode;

    let mut input = InputManager::new();

    // Default bindings.
    input.bind(InputAction::Exit, Keycode::Escape);

    input.bind(InputAction::TogglePause, Keycode::P);
    input.bind(InputAction::OpenMap, Keycode::Tab);
    input.bind(InputAction::OpenSettingsMenu, Keycode::R);
    input.bind(InputAction::OpenPartyMenu, Keycode::C);

    input.bind(InputAction::MenuPrevious, Keycode::Q);
    input.bind(InputAction::MenuNext, Keycode::E);
    input.bind(InputAction::MenuDown, Keycode::S);
    input.bind(InputAction::MenuLeft, Keycode::A);
    input.bind(InputAction::MenuRight, Keycode::D);

    input.bind(InputAction::MoveUp, Keycode::W);
    input.bind(InputAction::MoveDown, Keycode::S);
    input.bind(InputAction::MoveLeft, Keycode::A);
    input.bind(InputAction::MoveRight, Keycode::D);
    input.bind(InputAction::Activate, Keycode::F);
    input.bind(InputAction::Run, Keycode::LShift);

    input.bind(InputAction::DialogueChoicePrevious, Keycode::W);
    input.bind(InputAction::DialogueChoiceNext, Keycode::S);
    input.bind(InputAction::DialogueChoiceConfirm, Keycode::F);

    input.bind(InputAction::ToggleDebug, Keycode::Backspace);
    input.bind(InputAction::DebugCameraUp, Keycode::W);
    input.bind(InputAction::DebugCameraDown, Keycode::S);
    input.bind(InputAction::DebugCameraLeft, Keycode::A);
    input.bind(InputAction::DebugCameraRight, Keycode::D);
    input.bind(InputAction::DebugToggleLayer1, Keycode::_1);
    input.bind(InputAction::DebugToggleLayer2, Keycode::_2);
    input.bind(InputAction::DebugToggleLayer3, Keycode::_3);
    input.bind(InputAction::DebugToggleSprites, Keycode::_4);
    input.bind(InputAction::DebugTogglePalette, Keycode::_5);
    input.bind(InputAction::DebugOverlaysDisable, Keycode::Z);
    input.bind(InputAction::DebugOverlays1, Keycode::X);
    input.bind(InputAction::DebugOverlays2, Keycode::C);
    input.bind(InputAction::DebugOverlays3, Keycode::V);
    input.bind(InputAction::DebugOverlays4, Keycode::B);
    input.bind(InputAction::DebugOverlays5, Keycode::N);
    input.bind(InputAction::DebugOverlays6, Keycode::M);
    input.bind(InputAction::DebugOverlays7, Keycode::Comma);
    input.bind(InputAction::DebugOverlays8, Keycode::Period);
    input.bind(InputAction::DebugOverlays9, Keycode::Backslash);
    input.bind(InputAction::DebugDump, Keycode::Slash);
    input.bind(InputAction::DebugActorStep, Keycode::Space);

    let mut memory = Memory::new();
    memory.put_u8(0x7F0061, 1);     // Initialized at 0xC28D8A.

    // Run intro.
    memory.put_u8(0x7F0057, 4);


    let mut text_processor = TextProcessor::new();
    let party = Party::new();
    text_processor.update_party_names(&party);

    let mut ctx = Context {
        fs,
        l10n,
        sprite_states,
        assets,
        render,
        random,
        ui_theme,
        memory,
        party,
        text_processor,
        screen_fade,
        input,
        mode,
        debug_mode: false,
    };


    let mut gamestate: Box<dyn GameStateTrait>;
    if args.scene > -1 {
        gamestate = Box::new(GameStateScene::new(&mut ctx, args.scene as usize, Vec2Df64::new(128.0, 112.0), Facing::Down, true));
    } else if args.world > -1 {
        gamestate = Box::new(GameStateWorld::new(&mut ctx, args.world as usize, Vec2Df64::new(384.0, 296.0), true));
    } else {
        println!("No scene or world specified, loading world 0.");
        gamestate = Box::new(GameStateWorld::new(&mut ctx, 0, Vec2Df64::new(504.0, 448.0), true));
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

        // Process events.
        for event in event_pump.poll_iter() {
            match event {
                Event::MouseMotion { x, y, .. } => {
                    let (x, y) = ctx.render.window_to_target_coordinates(x, y);
                    gamestate.mouse_motion(&ctx, x, y);
                },
                Event::Quit {..} => break 'running,
                Event::KeyUp { keycode, .. } => {
                    if let Some(keycode) = keycode {
                        ctx.input.key_up(keycode);
                    }
                },
                Event::KeyDown { keycode, .. } => {
                    if let Some(keycode) = keycode {
                        ctx.input.key_down(keycode);
                    }
                }
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

            if ctx.input.was_pressed(InputAction::DebugDump) {
                ctx.assets.dump();
                ctx.ui_theme.dump();
                gamestate.dump(&ctx);
                ctx.render.target.write_to_bmp((&"debug_output/screenshot.bmp").as_ref());
            }

            if ctx.input.was_pressed(InputAction::Exit) {
                break 'running;
            }
            if ctx.input.was_pressed(InputAction::ToggleDebug) {
                ctx.debug_mode = !ctx.debug_mode;
                println!("Debug mode: {}.", ctx.debug_mode);
                gamestate.set_debug_mode(ctx.debug_mode);
                update_window_title(&mut ctx, &gamestate);
            }

            ctx.screen_fade.tick(UPDATE_INTERVAL);

            let game_event = gamestate.tick(&mut ctx, UPDATE_INTERVAL);
            if game_event.is_some() {
                match game_event.unwrap() {
                    GameEvent::GotoDestination { destination, fade_in } => {
                        println!("Heading to {}...", destination.as_string());

                        // Store previous location.
                        ctx.memory.put_u16(0x7E0105, destination.get_index() as u16);

                        match destination {
                            Destination::Scene { index, pos, facing, data } => {
                                ctx.memory.put_u8(0x7E0104, data);
                                gamestate = Box::new(GameStateScene::new(&mut ctx, index, pos.as_vec2d_f64(), facing, fade_in));
                            },
                            Destination::World { index, pos, data } => {
                                ctx.memory.put_u8(0x7E0104, data);
                                gamestate = Box::new(GameStateWorld::new(&mut ctx, index, pos.as_vec2d_f64(), fade_in));
                            },
                        };
                        update_window_title(&mut ctx, &gamestate);
                    },
                }
            }

            ctx.input.clear();

            stat_update_time += timer_update.stop();
            stat_update_count += 1;

            accumulator -= UPDATE_INTERVAL;
        }
        let lerp = accumulator / UPDATE_INTERVAL;

        // Render a frame.
        timer_render.start();

        ctx.render.clear();
        gamestate.render(&mut ctx, lerp);
        ctx.screen_fade.render(&mut ctx.render, lerp);
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

fn update_window_title(ctx: &mut Context, gamestate: &Box<dyn GameStateTrait>) {
    let title = if ctx.debug_mode {
        format!("Chrono Trigger - DEBUG - {}", gamestate.get_title(&ctx))
    } else {
        format!("Chrono Trigger - {}", gamestate.get_title(&ctx))
    };
    ctx.render.set_title(title.as_str());
}

fn create_filesystem(path: String) -> FileSystem {
    let src = Path::new(&path);

    // A directory is assumed to be the extracted version of the Steam resources.bin file.
    if src.is_dir() {
        let backend = FileSystemBackendPc::new(&src.into(), FileSystemBackendPcMode::FileSystem);
        return FileSystem::new(Box::new(backend), GameMode::Pc);

    // Steam version resources.bin.
    } else if src.file_name().unwrap().to_ascii_lowercase() == "resources.bin" {
        let backend = FileSystemBackendPc::new(&src.into(), FileSystemBackendPcMode::ResourcesBin);
        return FileSystem::new(Box::new(backend), GameMode::Pc);
    }

    // Any other file is assumed to be an SNES ROM image.
    let backend = FileSystemBackendSnes::new(&src);
    FileSystem::new(Box::new(backend), GameMode::Snes)
}
