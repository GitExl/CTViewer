use std::collections::HashMap;
use regex::Regex;
use crate::character::CharacterId;
use crate::Context;
use crate::party::Party;

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
pub enum TextPart {
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

#[derive(Debug, PartialEq)]
enum MatchType {
    TagOpen,
    TagClose,
    Word,
    Whitespace,
}

pub struct TextPage {
    pub parts: Vec<TextPart>,
    pub auto: bool,
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
    regex_match_parts: Vec<Regex>,
}

impl TextProcessor {
    pub fn new() -> TextProcessor {
        let mut replacements = HashMap::new();
        replacements.insert("<NAME_SIL>".into(), "Epoch".into());
        replacements.insert("<NAME_LEENE>".into(), "Leene".into());

        TextProcessor {
            replacements,
            regex_name_pt: Regex::new(r"^<NAME_PT(\d+)>").unwrap(),
            regex_match_parts: [
                Regex::new(r"^<(.+?)>").unwrap(),
                Regex::new(r"^</(\S+)>").unwrap(),
                Regex::new(r"^([^[:space:]<]+)").unwrap(),
                Regex::new(r"^(\s+)").unwrap(),
            ].to_vec(),
        }
    }

    pub fn update_party_names(&mut self, party: &Party) {
        for (_, character) in party.characters.iter() {
            self.replacements.insert(character.text_key.clone(), character.name.clone());
        }
    }

    pub fn process_dialog_text(&self, ctx: &Context, text: &str) -> Vec<TextPage> {
        println!("{}", text);
        let text = text.replace("\\", "<BR>");
        let pages = self.split_text(ctx, &text);

        self.dump_pages(&pages);

        pages
    }

    fn split_text(&self, ctx: &Context, text: &str) -> Vec<TextPage> {
        let mut index = 0;
        let mut pages: Vec<TextPage> = Vec::new();
        let mut page = TextPage::new();
        loop {
            let result = self.match_part(&text, index);
            if result.is_none() {
                break;
            }

            let (match_type, match_contents) = result.unwrap();
            index += match_contents.len();

            let contents = match match_type {
                MatchType::Word => Some(TextPart::Word { word: String::from(match_contents) }),
                MatchType::Whitespace => Some(TextPart::Whitespace { space: String::from(match_contents) }),

                MatchType::TagOpen => {
                    if match_contents == "<BR>" {
                        Some(TextPart::LineBreak)
                    } else if match_contents == "<END>" {
                        Some(TextPart::Progress)

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
                    } else if let Some(captures) = self.regex_name_pt.captures(&match_contents) {
                        let character_id: CharacterId = captures[0].parse().unwrap();
                        if let Some(character) = ctx.party.characters.get(&character_id) {
                            Some(TextPart::Word { word: character.name.clone() })
                        } else {
                            None
                        }

                    // From replacements map.
                    } else if let Some(replacement) = self.replacements.get(&match_contents) {
                        Some(TextPart::Word { word: replacement.to_string() })

                    } else {
                        Some(TextPart::Word { word: String::from(match_contents) })
                    }
                },

                MatchType::TagClose => {
                    None
                },
            };

            if let Some(contents) = contents {
                page.parts.push(contents);
            }
        }

        if page.parts.len() > 0 {
            pages.push(page);
        }

        pages
    }

    fn match_part(&self, text: &str, index: usize) -> Option<(MatchType, String)> {
        for (regex_index, regex) in self.regex_match_parts.iter().enumerate() {
            let matches = regex.find_at(&text[index..text.len()], 0);
            if let Some(matches) = matches {
                let contents = matches.as_str().to_string();
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

    fn dump_pages(&self, pages: &Vec<TextPage>) {
        for page in pages.iter() {
            println!("-------------------------------------");
            for part in page.parts.iter() {
                match part {
                    TextPart::Word { ref word } => print!("{}", word),
                    TextPart::LineBreak => println!(),
                    TextPart::Whitespace { ref space } => print!("{}", space),
                    _ => {},
                }
            }
            println!();
        }
        println!("-------------------------------------");
    }
}


fn tag_has_close(text: &str) -> bool {
    match text {
        "<WAIT>" => true,
        _ => false,
    }
}
