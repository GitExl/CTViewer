use std::collections::HashMap;
use crate::Context;
use crate::filesystem::filesystem::ParseMode;
use crate::renderer::{TextFont, TextRenderable};
use crate::software_renderer::palette::Color;
use crate::software_renderer::text::TextDrawFlags;
use crate::text_processor::{TextPage, TextPart};
use crate::util::vec2di32::Vec2Di32;

const TEXTBOX_TEXT_COLOR: Color = [231, 231, 231, 255];
const TEXTBOX_CHOICE_INDENT: i32 = 12;

pub struct TextBoxLayoutItem {
    pub renderable: TextRenderable,
    pub pos: Vec2Di32,
    pub wait: u32,
}

pub struct TextBoxLayout {
    pub items: Vec<TextBoxLayoutItem>,
    pub choices: HashMap<usize, Vec2Di32>,
}

impl TextBoxLayout {
    pub fn empty() -> TextBoxLayout {
        TextBoxLayout {
            items: Vec::new(),
            choices: HashMap::new(),
        }
    }

    pub fn from_page(ctx: &Context, page: &TextPage, line_height: i32, wrap_width: i32, _choice_lines: Option<[usize; 2]>) -> TextBoxLayout {
        let mut x: i32 = 0;
        let mut y: i32 = 0;
        let mut layout = TextBoxLayout::empty();

        // Disable wrapping in PC mode, its text should not need it.
        let wrap_width = if ctx.fs.parse_mode == ParseMode::Pc {
            0
        } else {
            wrap_width
        };

        for part in page.parts.iter() {
            let mut wrap = false;
            let text = match part {
                TextPart::Word { ref word } => {
                    wrap = true;
                    word
                },
                TextPart::Whitespace { ref space } => space,
                TextPart::LineBreak => {
                    x = 0;
                    y += line_height;
                    continue;
                },
                TextPart::Delay { ticks } => {
                    layout.items.last_mut().unwrap().wait = *ticks;
                    continue;
                },
                TextPart::Choice { index } => {
                    layout.choices.insert(*index, Vec2Di32::new(x, y));
                    continue;
                },
                _ => continue,
            };

            let (width, _height) = ctx.render.measure_text(text, TextFont::Regular);

            // Word wrapping.
            if wrap_width > 0 && wrap && x + width as i32 > wrap_width {
                x = 0;
                y += line_height;
            }

            // Store layout data.
            layout.items.push(TextBoxLayoutItem {
                pos: Vec2Di32::new(x, y),
                renderable: TextRenderable::new(text.clone(), TextFont::Regular, TEXTBOX_TEXT_COLOR, TextDrawFlags::SHADOW, 0),
                wait: 0,
            });

            // Move position.
            x += width as i32;
        }

        layout
    }
}
