use std::collections::HashMap;
use regex::Regex;
use crate::character::CharacterId;
use crate::Context;
use crate::party::Party;
use crate::renderer::TextFont;
use crate::util::vec2di32::Vec2Di32;

pub enum TextIcon {
    Blade,
    Bow,
    Gun,
    Arm,
    Sword,
    Fist,
    Scythe,
    Helm,
    Armor,
    Ring,
    Shield,
    Star,
    Left,
    Right,
}

#[derive(Debug, PartialEq)]
pub enum TextPartContents {
    /// A single word.
    Word {
        word: String
    },

    /// Wait for n ticks.
    Delay {
        ticks: usize,
    },

    /// Whitepace.
    Whitespace {
        space: String,
    },

    /// An icon.
    Icon {
        icon: usize,
    },

    /// Advance the dialog.
    Progress,

    /// Hard line break.
    LineBreak,
}

pub struct TextPart {
    size: Vec2Di32,
    pos: Vec2Di32,
    contents: TextPartContents,
}

pub struct TextPage {
    parts: Vec<TextPart>,
    auto: bool,
}

impl TextPage {
    pub fn new() -> TextPage {
        TextPage {
            auto: false,
            parts: Vec::new(),
        }
    }
}

// icons not part of the TTF font, need to handle these separately
// <BLADE> <BOW> <GUN> <ARM> <SWORD> <FIST> <SCYTHE> <HELM> <ARMOR> <RING>
//
// probably no need to implement these, they look like battle texts
// <H> <M> <P>
// <SHIELD> <STAR> <LEFT> <RIGHT>
// <HAND1> <HAND2> <HAND3> <HAND4>
// <H> <M> <P>
// <HP0> <HP1> <HP2> <HP3> <HP4> <HP5> <HP6> <HP7> <HP8>
// <D> <Z> <UP> <A> <L> <R>
// <H> <M> <P>
// <CORNER>
//
// text flow, not found in PC
// <STOP> ?
// <STOP LINE BREAK> ?
// <INSTANT LINE BREAK> ?

// text flow
// <AUTO_PAGE> Automatically go to next page after x time?
// <AUTO_END> ?
// <PAGE> New dialog page
// <BR> Hard line break
// <WAIT>00</WAIT> Wait for 00 ticks

// data
// <NUMBER> number from somewhere, PC
// <NUMBER 8> 8 bit number from somewhere, SNES
// <NUMBER 16> 16 bit number from somewhere, SNES
// <NUMBER 24> 24 bit number from somewhere, SNES
// <RESULT ITEM> item name from result value? SNES?

// Coliseum related
// <STR>
// <NAME_MON>
// <NAME_TEC> tech name, PC

// other
// <SPCH 11> from the SNES text decoder, should repeat last substring so should never appear here...
// <CT> center horizontally
// <UNKNOWN> an unknown character the text decoder didn't understand
// <UNKNOWN_SPEC> an unknown special character the text decoder didn't understand

// name replacements
// <NAME_CRO> Crono name
// <NAME_MAR> Marle name
// <NAME_LUC> Lucca name
// <NAME_FRO> Frog name
// <NAME_ROB> Robo name
// <NAME_AYL> Ayla name
// <NAME_MAG> Magus name
// <NICK_CRO> Crono nickname used by Ayla (what is this again?)
// <NAME_PT1> Party member 1 name
// <NAME_PT2> Party member 2 name
// <NAME_PT3> Party member 3 name
// <NAME_LEENE> always replaced by "Leene", SNES
// <NAME_SIL> name for the Epoch ("Sil Bird")

// used by choices by the PC version. end tags are ignored, we just want to use the part index
// <S10> Some sort of indentation?
// <C1>x</C1> Choice 1
// <C2>x</C2> Choice 2
// <C3>x</C3> Choice 3
// <C4>x</C4> Choice 4

pub struct TextProcessor {
    replacements: HashMap<String, String>,
    regex_name_pt: Regex,
}

impl TextProcessor {
    pub fn new() -> TextProcessor {
        let regex_name_pt = Regex::new(r"^<NAME_PT(\d+)>").unwrap();

        let mut replacements = HashMap::new();
        replacements.insert("<NAME_SIL>".into(), "Epoch".into());
        replacements.insert("<NAME_LEENE>".into(), "Leene".into());

        TextProcessor {
            replacements,
            regex_name_pt,
        }
    }

    pub fn update_party_names(&mut self, party: &Party) {
        for (_, character) in party.characters.iter() {
            self.replacements.insert(character.text_key.clone(), character.name.clone());
        }
    }

    pub fn process_dialog_text(&self, ctx: &Context, text: &str) -> Vec<TextPage> {
        let text = text.replace("\\", "<BR>");
        let mut pages = self.split_text(ctx, &text);
        self.layout_pages(ctx, &mut pages);

        self.dump_pages(&pages);

        pages
    }

    fn layout_pages(&self, ctx: &Context, pages: &mut Vec<TextPage>) {
        const WIDTH: u32 = 238;
        const LINE_HEIGHT: u32 = 16;

        for page in pages.iter_mut() {
            let mut x: u32 = 0;
            let mut y: u32 = 0;

            for part in page.parts.iter_mut() {
                let text = match part.contents {
                    TextPartContents::Word { ref word } => word,
                    TextPartContents::Whitespace { ref space } => space,
                    TextPartContents::LineBreak => {
                        x = 0;
                        y += LINE_HEIGHT;
                        continue;
                    },
                    _ => continue,
                };

                let (width, height) = ctx.render.measure_text(text, TextFont::Regular);

                // Word wrapping.
                if x + width > WIDTH {
                    x = 0;
                    y += LINE_HEIGHT;
                }

                // Store layout data.
                part.size.x = width as i32;
                part.size.y = height as i32;
                part.pos.x = x as i32;
                part.pos.y = y as i32;

                // Move position.
                x += width;
            }
        }
    }

    fn split_text(&self, ctx: &Context, text: &str) -> Vec<TextPage> {
        let mut index = 0;
        let mut pages: Vec<TextPage> = Vec::new();
        let mut page = TextPage::new();
        loop {
            let result = match_part(&text, index);
            if result.is_none() {
                break;
            }

            let (match_type, match_contents) = result.unwrap();
            index += match_contents.len();

            let contents = match match_type {
                MatchType::Word => Some(TextPartContents::Word { word: String::from(match_contents) }),
                MatchType::Whitespace => Some(TextPartContents::Whitespace { space: String::from(match_contents) }),

                MatchType::TagOpen => {
                    if match_contents == "<BR>" {
                        Some(TextPartContents::LineBreak)
                    } else if match_contents == "<END>" {
                        Some(TextPartContents::Progress)

                    // Page end. What does AUTO_END mean?
                    } else if match_contents == "<PAGE>" || match_contents == "<AUTO_END>" {
                        pages.push(page);
                        page = TextPage::new();
                        None

                    // Page end, advance to next page.
                    } else if match_contents == "<AUTO_PAGE>" {
                        page.auto = true;
                        page = TextPage::new();
                        None

                    // Match a choice option.
                    // todo track what part contains the start of the option
                    } else if match_contents == "<C1>" || match_contents == "<C2>" || match_contents == "<C3>" || match_contents == "<C4>" {
                        None

                    // Match a party member name.
                    // <NAME_PT*>
                    } else if let Some(captures) = self.regex_name_pt.captures(match_contents) {
                        let character_id: CharacterId = captures[0].parse().unwrap();
                        if let Some(character) = ctx.party.characters.get(&character_id) {
                            Some(TextPartContents::Word { word: character.name.clone() })
                        } else {
                            None
                        }

                    // From replacements map.
                    } else if let Some(replacement) = self.replacements.get(match_contents) {
                        Some(TextPartContents::Word { word: replacement.to_string() })

                    } else {
                        Some(TextPartContents::Word { word: String::from(match_contents) })
                    }
                },

                MatchType::TagClose => {
                    None
                },
            };

            if let Some(contents) = contents {
                page.parts.push(TextPart {
                    contents,
                    size: Vec2Di32::default(),
                    pos: Vec2Di32::default(),
                });
            }
        }

        if page.parts.len() > 0 {
            pages.push(page);
        }

        pages
    }

    fn dump_pages(&self, pages: &Vec<TextPage>) {
        for page in pages.iter() {
            println!("-------------------------------------");
            for part in page.parts.iter() {
                match part.contents {
                    TextPartContents::Word { ref word } => print!("{}", word),
                    TextPartContents::LineBreak => println!(),
                    TextPartContents::Whitespace { ref space } => print!("{}", space),
                    _ => {},
                }
            }
            println!();
        }
        println!("-------------------------------------");
    }
}



#[derive(Debug, PartialEq)]
enum MatchType {
    TagOpen,
    TagClose,
    Word,
    Whitespace,
}

fn match_part(text: &str, index: usize) -> Option<(MatchType, &str)> {
    let parts_regex = [
        Regex::new(r"^<(.+?)>").unwrap(),
        Regex::new(r"^</(\S+)>").unwrap(),
        Regex::new(r"^([^[:space:]<]+)").unwrap(),
        Regex::new(r"^(\s+)").unwrap(),
    ];

    for (regex_index, regex) in parts_regex.iter().enumerate() {
        let matches = regex.find_at(&text[index..text.len()], 0);
        if let Some(matches) = matches {
            let contents = matches.as_str();
            let part = match regex_index {
                0 => MatchType::TagOpen,
                1 => MatchType::TagClose,
                2 => MatchType::Word,
                3 => MatchType::Whitespace,
                _ => panic!("Unknown match type."),
            };
            return Some((part, contents));
        }
    }

    None
}

fn tag_has_close(text: &str) -> bool {
    match text {
        "<WAIT>" => true,
        _ => false,
    }
}
