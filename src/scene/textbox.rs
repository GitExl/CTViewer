use crate::Context;
use crate::renderer::{TextFlags, TextFont, TextRenderable};
use crate::software_renderer::blit::{blit_surface_to_surface, SurfaceBlendOps};
use crate::software_renderer::palette::Color;
use crate::software_renderer::surface::Surface;
use crate::software_renderer::text::TextDrawFlags;
use crate::text_processor::process_text;

const TEXTBOX_TEXT_COLOR: Color = [231, 231, 231, 255];
const TEXTBOX_CHIP_WIDTH: i32 = 32;
const TEXTBOX_CHIP_HEIGHT: i32 = 10;
const TEXTBOX_ANIMATE_PIXELS_PER_SECOND: f64 = 6.0;
const TEXTBOX_SHOW_CHARS_PER_SECOND: f64 = 60.0;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum TextBoxPosition {
    Top,
    Bottom,
    Auto,
}

#[derive(PartialEq)]
pub enum TextBoxState {
    Disabled,
    Showing {
        last_visibility: f64,
        visibility: f64,
    },
    Typing {
        current_line: usize,
        current_character: f64,
    },
    Waiting,
    Hiding {
        last_visibility: f64,
        visibility: f64,
    },
}

struct TextPageLine {
    text: String,
    renderable: TextRenderable,
}

struct TextPage {
    lines: Vec<TextPageLine>,
}

pub struct TextBox {
    state: TextBoxState,
    pages: Vec<TextPage>,
    current_page: usize,
    position: TextBoxPosition,
    actor_index: Option<usize>,

    window_surface: Surface,
}

impl TextBox {
    pub fn new(ctx: &mut Context) -> TextBox {

        // Render a fresh copy of the window UI background.
        let mut window_surface = Surface::new(TEXTBOX_CHIP_WIDTH as u32 * 8, TEXTBOX_CHIP_HEIGHT as u32 * 8);
        ctx.ui_theme.render_window(&mut window_surface, 0, 0, TEXTBOX_CHIP_WIDTH, TEXTBOX_CHIP_HEIGHT);

        TextBox {
            pages: Vec::new(),
            current_page: 0,
            state: TextBoxState::Disabled,
            position: TextBoxPosition::Bottom,
            actor_index: None,

            window_surface,
        }
    }

    pub fn tick(&mut self, delta: f64) {
        if self.state == TextBoxState::Disabled {
            return;
        }

        match &mut self.state {

            // Expand the window while showing.
            TextBoxState::Showing { visibility, last_visibility } => {
                *last_visibility = *visibility;
                *visibility += delta * TEXTBOX_ANIMATE_PIXELS_PER_SECOND;
                if *visibility >= 1.0 {
                    *visibility = 1.0;
                    self.state = TextBoxState::Typing {
                        current_line: 0,
                        current_character: 0.0,
                    };
                }
            },

            // Type out one character per tick.
            TextBoxState::Typing { current_line: line, current_character: character } => {
                *character += delta * TEXTBOX_SHOW_CHARS_PER_SECOND;

                // End of line.
                if *character as usize >= self.pages[self.current_page].lines[*line].renderable.get_char_count() {
                    *line += 1;

                    // End of page.
                    if *line >= self.pages[self.current_page].lines.len() {
                        self.state = TextBoxState::Waiting;
                    } else {
                        *character = 0.0;
                    }
                }
            },

            // Contract the window while hiding.
            TextBoxState::Hiding { visibility, last_visibility } => {
                *last_visibility = *visibility;
                *visibility -= delta * TEXTBOX_ANIMATE_PIXELS_PER_SECOND;
                if *visibility <= 0.0 {
                    self.actor_index = None;
                    self.state = TextBoxState::Disabled;
                }
            },
            _ => {},
        }
    }

    pub fn is_busy(&self) -> bool {
        match self.state {
            TextBoxState::Disabled => false,
            _ => true,
        }
    }

    pub fn get_actor_index(&self) -> Option<usize> {
        self.actor_index
    }

    pub fn show(&mut self, text: String, position: TextBoxPosition, actor_index: usize) {

        process_text(text.as_str());

        // Split the text into pages, and the pages into lines.
        self.pages.clear();
        for text_page in text.split("<PAGE>") {
            let text_lines: Vec<String> = text_page.split("<BR>").map(str::to_string).collect();

            let mut text_page_lines = Vec::<TextPageLine>::new();
            for text in text_lines {

                // Prepare the line's text for rendering.
                let renderable = TextRenderable::new(
                    text.clone(),
                    TextFont::Regular,
                    TEXTBOX_TEXT_COLOR,
                    TextDrawFlags::SHADOW,
                    0,
                );

                text_page_lines.push(TextPageLine {
                    renderable,
                    text,
                });
            }

            self.pages.push(TextPage {
                lines: text_page_lines,
            });
        }

        self.state = TextBoxState::Showing {
            visibility: 0.0,
            last_visibility: 0.0,
        };
        self.position = position;
        self.current_page = 0;
        self.actor_index = Some(actor_index);
    }

    pub fn progress(&mut self) {
        if self.state == TextBoxState::Disabled {
            return;
        }

        if matches!(self.state, TextBoxState::Waiting) {

            // Advance to next page.
            if self.current_page < self.pages.len() - 1 {
                self.current_page += 1;
                self.state = TextBoxState::Typing {
                    current_line: 0,
                    current_character: 0.0,
                };

            // Reached end of pages, hide.
            } else {
                self.state = TextBoxState::Hiding {
                    visibility: 1.0,
                    last_visibility: 1.0,
                };
            }
        }
    }

    pub fn render(&mut self, ctx: &mut Context, lerp: f64) {
        if self.state == TextBoxState::Disabled {
            return;
        }

        // Determine the part of the window surface to show.
        let window_width = self.window_surface.width as i32;
        let mut window_height = self.window_surface.height as i32;
        let mut src_y = 0;
        let mut dest_y = match self.position {
            TextBoxPosition::Top => 0,
            TextBoxPosition::Bottom => ctx.render.target.height as i32 - window_height,
            TextBoxPosition::Auto => 0,    // todo, what does this mean exactly?
        };
        match self.state {
            TextBoxState::Showing { visibility, last_visibility } |
            TextBoxState::Hiding { visibility, last_visibility } => {
                let visibility_lerp = last_visibility + (visibility - last_visibility) * lerp;
                let offset = (window_height as f64 * visibility_lerp * 0.5) as i32;
                dest_y += window_height / 2 - offset;
                src_y += window_height / 2 - offset;
                window_height = (window_height as f64 * visibility_lerp) as i32;
            },
            _ => {}
        }

        // Draw the window UI surface.
        blit_surface_to_surface(&mut self.window_surface, &mut ctx.render.target, 0, src_y, window_width, window_height, 0, dest_y, SurfaceBlendOps::Copy);

        // Draw the text.
        match self.state {

            // While typing, draw all lines up to the current one and set a character limit for the current one.
            TextBoxState::Typing { current_line: line, current_character: character } => {
                let current_page = &mut self.pages[self.current_page];
                for (line_index, page_line) in current_page.lines.iter_mut().enumerate() {
                    let x = if line_index == 0 { 8 } else { 20 };
                    let y = dest_y + 9 + line_index as i32 * 16;

                    // Limit character count for the current line.
                    let char_count = character as usize;
                    if line_index == line {
                        page_line.renderable.set_char_count_to_show(char_count);
                    }

                    ctx.render.render_text(&mut page_line.renderable, x, y, TextFlags::AlignHStart | TextFlags::AlignVStart);

                    // Skip future lines.
                    if line_index >= line {
                        break;
                    }
                }
            },

            // While waiting, draw all lines.
            TextBoxState::Waiting => {
                let current_page = &mut self.pages[self.current_page];
                for (line_index, page_line) in current_page.lines.iter_mut().enumerate() {
                    let x = if line_index == 0 { 8 } else { 20 };
                    let y = dest_y + 9 + line_index as i32 * 16;
                    ctx.render.render_text(&mut page_line.renderable, x, y, TextFlags::AlignHStart | TextFlags::AlignVStart);
                }
            },
            _ => {},
        };
    }
}
