use std::collections::HashMap;
use regex::Regex;
use crate::party::{CharacterPartyState, Party};

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
        ticks: u32,
    },

    /// Whitepace.
    Whitespace {
        space: String,
    },

    /// An icon.
    Icon {
        icon: usize,
    },

    /// Choice option.
    Choice {
        index: usize,
    },

    /// Advance the dialog.
    Progress,

    /// Hard line break.
    LineBreak,
}

#[derive(Debug, PartialEq)]
enum MatchType {
    TagOpen = 0,
    Word = 1,
    Whitespace = 2,
    TagWait = 3,
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
// <BR> Hard line break, indent next line?
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
    regex_choice: Regex,
    regex_variable: Regex,
    regex_match_parts: Vec<Regex>,
}

impl TextProcessor {
    pub fn new() -> TextProcessor {
        let mut replacements = HashMap::new();
        replacements.insert("NAME_SIL".into(), "Epoch".into());
        replacements.insert("NAME_LEENE".into(), "Leene".into());

        TextProcessor {
            replacements,
            regex_choice: Regex::new(r"^C(\d{1})$").unwrap(),
            regex_variable: Regex::new(r"<(.+?)>").unwrap(),
            regex_match_parts: [
                Regex::new(r"^<WAIT>(.+?)</WAIT>").unwrap(),
                Regex::new(r"^<(.+?)>").unwrap(),
                Regex::new(r"^([^[:space:]<]+)").unwrap(),
                Regex::new(r"^(\s+)").unwrap(),
            ].to_vec(),
        }
    }

    pub fn update_party_names(&mut self, party: &Party) {
        for (index, character) in party.characters.iter() {
            self.replacements.insert(character.text_key.clone(), character.name.clone());
            if character.party_state == CharacterPartyState::Active {
                self.replacements.insert(format!("NAME_PT{}", index + 1), character.name.clone());
            }
        }
    }

    pub fn process_dialog_text(&self, text: &str) -> Vec<TextPage> {
        println!("{}", text);

        // Change PC line breaks to SNES line breaks.
        let text = text.replace("\\", "<BR>");
        let text = self.replace_variables(text);

        // Split text into parts separated by whitespace.
        let mut index = 0;
        let mut pages: Vec<TextPage> = Vec::new();
        let mut page = TextPage::new();
        loop {
            let result = self.match_part(&text, index);
            if result.is_none() {
                break;
            }

            let (match_type, match_contents, match_len) = result.unwrap();
            index += match_len;

            match match_type {
                MatchType::Word => {
                    page.parts.push(TextPart::Word { word: String::from(match_contents) });
                },
                MatchType::Whitespace => {
                    page.parts.push(TextPart::Whitespace { space: String::from(match_contents) });
                },
                MatchType::TagOpen => {

                    // Hard line break.
                    if match_contents == "BR" {
                        page.parts.push(TextPart::LineBreak);

                    // Auto-progress.
                    } else if match_contents == "END" {
                        page.parts.push(TextPart::Progress);

                    // Page end. What does AUTO_END mean?
                    } else if match_contents == "PAGE" || match_contents == "AUTO_END" {
                        pages.push(page);
                        page = TextPage::new();

                    // Page end, advance to next page.
                    } else if match_contents == "AUTO_PAGE" {
                        page.auto = true;
                        page = TextPage::new();

                    // Whitespace?
                    } else if match_contents == "S10" {
                        page.parts.push(TextPart::Whitespace { space: "   ".into() });

                    // Match a choice option.
                    } else if let Some(captures) = self.regex_choice.captures(&match_contents) {
                        let index: usize = captures[1].parse().unwrap();
                        page.parts.push(TextPart::Choice { index });

                    }
                },

                // Wait for n ticks.
                MatchType::TagWait => {
                    let ticks: u32 = u32::from_str_radix(&match_contents, 16).unwrap();
                    page.parts.push(TextPart::Delay { ticks });
                },
            };
        }

        if page.parts.len() > 0 {
            pages.push(page);
        }

        self.dump_pages(&pages);

        pages
    }

    fn replace_variables(&self, text: String) -> String {
        let mut new_text = String::with_capacity(text.len());
        let mut last_match = 0;
        for capture in self.regex_variable.captures_iter(&text) {
            let variable = capture.get(1).unwrap().as_str();
            if !self.replacements.contains_key(variable) {
                continue;
            }

            let m = capture.get(0).unwrap();
            let replacement = self.replacements.get(variable).unwrap();

            // Add the text before this match, and the replacement.
            new_text.push_str(&text[last_match..m.start()]);
            new_text.push_str(&replacement);
            last_match = m.end();
        }

        // Add the last text after the last match.
        new_text.push_str(&text[last_match..]);

        new_text
    }

    fn match_part(&self, text: &str, index: usize) -> Option<(MatchType, String, usize)> {
        for (regex_index, regex) in self.regex_match_parts.iter().enumerate() {
            let captures = regex.captures_at(&text[index..text.len()], 0);
            if let Some(captures) = captures {
                let len = captures[0].len();
                let contents = captures[1].to_string();
                let part = match regex_index {
                    0 => MatchType::TagWait,
                    1 => MatchType::TagOpen,
                    2 => MatchType::Word,
                    3 => MatchType::Whitespace,
                    _ => panic!("Unknown match type."),
                };
                return Some((part, contents, len));
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
