use crate::Context;
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

pub struct TextBoxChoice {
    pub pos: Vec2Di32,
    pub line: usize,
}

pub struct TextBoxLayout {
    pub items: Vec<TextBoxLayoutItem>,
    pub choices: Vec<TextBoxChoice>,
}

impl TextBoxLayout {
    pub fn empty() -> TextBoxLayout {
        TextBoxLayout {
            items: Vec::new(),
            choices: Vec::new(),
        }
    }

    pub fn from_page(ctx: &Context, page: &TextPage, line_height: i32, choice_lines: Option<[usize; 2]>) -> TextBoxLayout {
        let mut x: i32 = 0;
        let mut y: i32 = 0;
        let mut line = 0;
        let mut layout = TextBoxLayout::empty();

        add_choice_line_choice(choice_lines, line, &mut layout, x, y);

        for part in page.parts.iter() {
            let text = match part {
                TextPart::Word { ref word } => { word },
                TextPart::Whitespace { ref space } => space,
                TextPart::LineBreak => {
                    x = 0;
                    y += line_height;
                    line += 1;
                    add_choice_line_choice(choice_lines, line, &mut layout, x, y);
                    continue;
                },
                TextPart::Delay { ticks } => {
                    layout.items.last_mut().unwrap().wait = *ticks;
                    continue;
                },
                TextPart::Choice { index } => {
                    layout.choices.push(TextBoxChoice {
                        pos: Vec2Di32::new(x, y),
                        line: *index,
                    });
                    continue;
                },
            };

            let (width, _height) = ctx.render.measure_text(text, TextFont::Regular);

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

fn add_choice_line_choice(choice_lines: Option<[usize; 2]>, line: usize, layout: &mut TextBoxLayout, x: i32, y: i32) {
    if let Some(choice_lines) = choice_lines {
        if line >= choice_lines[0] && line <= choice_lines[1] {
            layout.choices.push(TextBoxChoice {
                pos: Vec2Di32::new(x + 24, y),
                line,
            });
        }
    }
}
