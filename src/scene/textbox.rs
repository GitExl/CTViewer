use crate::Context;
use crate::filesystem::filesystem::ParseMode;
use crate::renderer::TextFlags;
use crate::scene::textbox_layout::TextBoxLayout;
use crate::software_renderer::blit::{blit_bitmap_to_surface, blit_surface_to_surface, BitmapBlitFlags, SurfaceBlendOps};
use crate::software_renderer::surface::Surface;
use crate::text_processor::TextPage;

const TEXTBOX_CHIP_WIDTH_SNES: i32 = 32;
const TEXTBOX_CHIP_WIDTH_PC: i32 = 44;
const TEXTBOX_CHIP_HEIGHT: i32 = 10;
const TEXTBOX_LINE_HEIGHT: i32 = 16;
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

pub struct TextBox {
    state: TextBoxState,

    pages: Vec<TextPage>,
    current_page: usize,
    layout: TextBoxLayout,

    position: TextBoxPosition,
    source_actor_index: Option<usize>,
    wait: u32,

    choice_lines: Option<[usize; 2]>,
    current_choice: usize,

    window_surface: Surface,
}

impl TextBox {
    pub fn new(ctx: &mut Context) -> TextBox {
        let chip_width = if ctx.fs.parse_mode == ParseMode::Pc { TEXTBOX_CHIP_WIDTH_PC } else { TEXTBOX_CHIP_WIDTH_SNES };

        // Render a fresh copy of the window UI background.
        let mut window_surface = Surface::new(chip_width as u32 * 8, TEXTBOX_CHIP_HEIGHT as u32 * 8);
        ctx.ui_theme.render_window(&mut window_surface, 0, 0, chip_width, TEXTBOX_CHIP_HEIGHT);

        TextBox {
            state: TextBoxState::Disabled,

            pages: Vec::new(),
            current_page: 0,
            layout: TextBoxLayout::empty(),

            position: TextBoxPosition::Bottom,
            source_actor_index: None,
            wait: 0,

            choice_lines: None,
            current_choice: 0,

            window_surface,
        }
    }

    pub fn tick(&mut self, ctx: &mut Context, delta: f64) {
        if self.state == TextBoxState::Disabled {
            return;
        }

        if self.wait > 0 {
            self.wait -= 1;
            if self.wait == 0 {
                self.advance(ctx);
            }
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
                let item = &mut self.layout.items[*current_item];
                *current_character += delta * TEXTBOX_SHOW_CHARS_PER_SECOND;

                // End of item.
                if *current_character as usize >= item.renderable.get_char_count() {
                    if item.wait > 0 {
                        self.wait = item.wait;
                    }

                    if *current_item < self.layout.items.len() - 1 {
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
                    self.source_actor_index = None;
                    self.state = TextBoxState::Disabled;
                }
            },
            _ => {},
        }
    }

    pub fn is_active(&self) -> bool {
        match self.state {
            TextBoxState::Disabled => false,
            _ => true,
        }
    }

    pub fn get_source_actor_index(&self) -> Option<usize> {
        self.source_actor_index
    }

    pub fn show(&mut self, ctx: &mut Context, text: String, position: TextBoxPosition, actor_index: usize, choice_lines: Option<[usize; 2]>, result_value: u32, result_item: String) {
        self.state = TextBoxState::Showing {
            visibility: 0.0,
            last_visibility: 0.0,
        };
        self.position = position;

        self.pages = ctx.text_processor.process_dialog_text(text.as_str(), result_value, result_item);
        self.current_page = 0;

        self.source_actor_index = Some(actor_index);

        self.current_choice = 0;
        self.choice_lines = choice_lines;

        self.layout_current_page(ctx);
    }

    pub fn choice_previous(&mut self) {
        if self.current_choice == 0 {
            self.current_choice = self.layout.choices.len() - 1;
        } else {
            self.current_choice -= 1;
        }
    }

    pub fn choice_next(&mut self) {
        if self.current_choice == self.layout.choices.len() - 1 {
            self.current_choice = 0;
        } else {
            self.current_choice += 1;
        }
    }

    pub fn progress(&mut self, ctx: &mut Context) {
        if self.state == TextBoxState::Disabled {
            return;
        }

        if matches!(self.state, TextBoxState::Waiting) && self.wait == 0 {
            self.advance(ctx);
        }
    }

    pub fn has_choices(&self) -> bool {
        self.layout.choices.len() > 0
    }

    pub fn get_choice(&self) -> usize {
        self.layout.choices[self.current_choice].line
    }

    fn advance(&mut self, ctx: &mut Context) {

        // Advance to next page.
        if self.current_page < self.pages.len() - 1 {
            self.current_page += 1;
            self.current_choice = 0;
            self.layout_current_page(ctx);

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

    pub fn render(&mut self, ctx: &mut Context, lerp: f64) {
        if self.state == TextBoxState::Disabled {
            return;
        }

        // Determine the part of the window surface to show.
        let window_width = self.window_surface.width as i32;
        let mut window_height = self.window_surface.height as i32;
        let mut src_y = 0;
        let dest_x = ctx.render.target.width as i32 / 2 - window_width / 2;
        let mut dest_y = match self.position {
            TextBoxPosition::Top => 0,
            TextBoxPosition::Bottom => ctx.render.target.height as i32 - window_height,
            TextBoxPosition::Auto => 0,
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
        blit_surface_to_surface(&mut self.window_surface, &mut ctx.render.target, 0, src_y, window_width, window_height, dest_x, dest_y, SurfaceBlendOps::Copy);

        // Draw the text.
        match self.state {

            // While typing, draw all lines up to the current one and set a character limit for the current one.
            TextBoxState::Typing { current_character, current_item } => {
                for (index, item) in self.layout.items.iter_mut().enumerate() {
                    if index > current_item {
                        break;
                    } else if index == current_item {
                        item.renderable.set_char_count_to_show(current_character.ceil() as usize);
                    } else {
                        item.renderable.set_char_count_to_show(item.renderable.get_char_count());
                    }

                    let x = dest_x + 8 + item.pos.x;
                    let y = dest_y + 9 + item.pos.y;
                    ctx.render.render_text(&mut item.renderable, x, y, TextFlags::empty());
                }
            },

            // While waiting, draw all lines.
            TextBoxState::Waiting => {
                for item in self.layout.items.iter_mut() {
                    let x = dest_x + 8 + item.pos.x;
                    let y = dest_y + 9 + item.pos.y;
                    ctx.render.render_text(&mut item.renderable, x, y, TextFlags::empty());
                }

                // Display choice.
                if self.layout.choices.len() > 0 {
                    let choice = &self.layout.choices[self.current_choice];
                    let choice_x = dest_x + 8 + choice.pos.x - 18;
                    let choice_y = dest_y + 9 + choice.pos.y + 1;
                    blit_bitmap_to_surface(&ctx.ui_theme.cursor_bitmap, &mut ctx.render.target, 0, 0, 16, 16, choice_x, choice_y, &ctx.ui_theme.cursor_palette, 0, BitmapBlitFlags::SKIP_0);
                }
            },
            _ => {},
        };
    }

    fn layout_current_page(&mut self, ctx: &Context) {
        let choice_lines = if self.current_page < self.pages.len() - 1 {
            None
        } else {
            self.choice_lines
        };
        self.layout = TextBoxLayout::from_page(ctx, &self.pages[self.current_page], self.window_surface.width as i32 - 16, TEXTBOX_LINE_HEIGHT, choice_lines);
    }
}
