use crate::Context;
use crate::renderer::{TextFlags, TextFont, TextRenderable};
use crate::software_renderer::blit::{blit_surface_to_surface, SurfaceBlendOps};
use crate::software_renderer::palette::Color;
use crate::software_renderer::surface::Surface;
use crate::software_renderer::text::TextDrawFlags;
use crate::text_processor::{TextPage, TextPart};
use crate::util::vec2di32::Vec2Di32;

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
        current_item: usize,
        current_character: f64,
    },
    Waiting,
    Hiding {
        last_visibility: f64,
        visibility: f64,
    },
}

struct LayoutItem {
    renderable: TextRenderable,
    pos: Vec2Di32,
}

pub struct TextBox {
    state: TextBoxState,

    pages: Vec<TextPage>,
    current_page: usize,

    layout_items: Vec<LayoutItem>,

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
            state: TextBoxState::Disabled,

            pages: Vec::new(),
            current_page: 0,

            layout_items: Vec::new(),

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
                        current_item: 0,
                        current_character: 0.0,
                    };
                }
            },

            // Type out one character per tick.
            TextBoxState::Typing { current_item, current_character } => {
                *current_character += delta * TEXTBOX_SHOW_CHARS_PER_SECOND;
                let renderable = &mut self.layout_items[*current_item].renderable;

                // End of item.
                if *current_character as usize >= renderable.get_char_count() {
                    if *current_item < self.layout_items.len() - 1 {
                        *current_item += 1;
                        *current_character = 0.0;
                    } else {
                        self.state = TextBoxState::Waiting;
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

    pub fn show(&mut self, ctx: &Context, text: String, position: TextBoxPosition, actor_index: usize) {
        self.state = TextBoxState::Showing {
            visibility: 0.0,
            last_visibility: 0.0,
        };
        self.position = position;

        self.pages = ctx.text_processor.process_dialog_text(ctx, text.as_str());
        self.current_page = 0;

        self.layout_items = self.layout_page(ctx, &self.pages[0]);

        self.actor_index = Some(actor_index);
    }

    pub fn progress(&mut self, ctx: &mut Context) {
        if self.state == TextBoxState::Disabled {
            return;
        }

        if matches!(self.state, TextBoxState::Waiting) {

            // Advance to next page.
            if self.current_page < self.pages.len() - 1 {
                self.current_page += 1;

                self.layout_items = self.layout_page(ctx, &self.pages[self.current_page]);

                self.state = TextBoxState::Typing {
                    current_item: 0,
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
            TextBoxState::Typing { current_character, current_item } => {
                for (index, item) in self.layout_items.iter_mut().enumerate() {
                    if index > current_item {
                        break;
                    } else if index == current_item {
                        item.renderable.set_char_count_to_show(current_character.ceil() as usize);
                    } else {
                        item.renderable.set_char_count_to_show(item.renderable.get_char_count());
                    }

                    let x = 8 + item.pos.x;
                    let y = dest_y + 9 + item.pos.y;
                    ctx.render.render_text(&mut item.renderable, x, y, TextFlags::empty());
                }
            },

            // While waiting, draw all lines.
            TextBoxState::Waiting => {
                for item in self.layout_items.iter_mut() {
                    let x = 8 + item.pos.x;
                    let y = dest_y + 9 + item.pos.y;
                    ctx.render.render_text(&mut item.renderable, x, y, TextFlags::empty());
                }
            },
            _ => {},
        };
    }

    fn layout_page(&self, ctx: &Context, page: &TextPage) -> Vec<LayoutItem> {
        const WIDTH: u32 = 238;
        const LINE_HEIGHT: u32 = 16;

        let mut x: u32 = 0;
        let mut y: u32 = 0;
        let mut layout = Vec::new();

        for part in page.parts.iter() {
            let text = match part {
                TextPart::Word { ref word } => word,
                TextPart::Whitespace { ref space } => space,
                TextPart::LineBreak => {
                    x = 0;
                    y += LINE_HEIGHT;
                    continue;
                },
                _ => continue,
            };

            let (width, _height) = ctx.render.measure_text(text, TextFont::Regular);

            // Word wrapping.
            if x + width > WIDTH {
                x = 0;
                y += LINE_HEIGHT;
            }

            // Store layout data.
            layout.push(LayoutItem {
                pos: Vec2Di32::new(x as i32, y as i32),
                renderable: TextRenderable::new(text.clone(), TextFont::Regular, TEXTBOX_TEXT_COLOR, TextDrawFlags::SHADOW, 0),
            });


            // Move position.
            x += width;
        }

        layout
    }
}
