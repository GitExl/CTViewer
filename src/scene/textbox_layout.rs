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
    pub size: Vec2Di32,
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

    pub fn from_page(ctx: &Context, page: &TextPage, page_width: i32, line_height: i32, choice_lines: Option<[usize; 2]>) -> TextBoxLayout {
        let mut x: i32 = 0;
        let mut y: i32 = 0;
        let mut line = 0;
        let mut center_start = None;
        let mut layout = TextBoxLayout::empty();

        // Add a choice for the first line if there is any.
        add_choice_line_choice(choice_lines, line, &mut layout, x, y);

        for part in page.iter() {
            match part {
                TextPart::Text { ref text } => {
                    // Measure text size.
                    let (width, height) = ctx.render.measure_text(text, TextFont::Regular);

                    // Store layout data for the text.
                    layout.items.push(TextBoxLayoutItem {
                        pos: Vec2Di32::new(x, y),
                        size: Vec2Di32::new(width as i32, height as i32),
                        renderable: TextRenderable::new(text.clone(), TextFont::Regular, TEXTBOX_TEXT_COLOR, TextDrawFlags::SHADOW, 0),
                        wait: 0,
                    });

                    // Move position.
                    x += width as i32;
                },
                TextPart::LineBreak => {

                    // If this line needed centering, do it after the line is complete.
                    if let Some(center_start) = center_start {
                        let center_end = layout.items.len();
                        center_line(&mut layout.items, center_start, center_end, page_width);
                    }
                    center_start = None;

                    x = 0;
                    y += line_height;

                    line += 1;
                    add_choice_line_choice(choice_lines, line, &mut layout, x, y);
                },
                TextPart::Delay { ticks } => {
                    layout.items.last_mut().unwrap().wait = *ticks;
                },
                TextPart::CenterNextLine => {
                    center_start = Some(layout.items.len());
                }
            };
        }

        // Center the last line if needed.
        if let Some(center_start) = center_start {
            let center_end = layout.items.len();
            center_line(&mut layout.items, center_start, center_end, page_width);
        }

        layout
    }
}

fn center_line(items: &mut Vec<TextBoxLayoutItem>, start: usize, end: usize, textbox_width: i32) {

    // Measure the line length.
    let mut line_width = 0;
    for i in start..end {
        line_width += items[i].size.x;
    }
println!("{}", line_width);
    // Shift the line items back to center them.
    let offset = (textbox_width / 2) - line_width / 2;
    for i in start..end {
        items[i].pos.x += offset;
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
