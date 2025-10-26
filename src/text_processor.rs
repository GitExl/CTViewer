use std::collections::HashMap;
use regex::Regex;
use crate::party::{CharacterPartyState, Party};

#[derive(Debug, PartialEq)]
pub enum TextPart {
    /// A single word.
    Text {
        text: String
    },

    /// Wait for n ticks.
    Delay {
        ticks: u32,
    },

    /// Choice option.
    Choice {
        index: usize,
    },

    /// Hard line break.
    LineBreak,
}

#[derive(Debug, PartialEq)]
enum PartType {
    TagWait,
    TagOpen,
    Text,
}

pub type TextPage = Vec<TextPart>;

// icons not part of the TTF font, need to handle these separately - todo
// <BLADE> <BOW> <GUN> <ARM> <SWORD> <FIST> <SCYTHE> <HELM> <ARMOR> <RING>
//
// probably no need to implement these, they look battle UI related - todo
// <H> <M> <P>
// <SHIELD> <STAR> <LEFT> <RIGHT>
// <HAND1> <HAND2> <HAND3> <HAND4>
// <H> <M> <P>
// <HP0> <HP1> <HP2> <HP3> <HP4> <HP5> <HP6> <HP7> <HP8>
// <D> <Z> <UP> <A> <L> <R>
// <H> <M> <P>
// <CORNER>

// text flow
// <AUTO_PAGE> Automatically go to next page after x time?
// <AUTO_END> End, skipping remaining pages
// <INDENT> 3 space indent
// <BR> Hard line break, indent next line?
// <WAIT>00</WAIT> Wait for 00 ticks, then auto-progress

// data
// <NUMBER> number from textbox choice result. PC
// <NUMBER 8> 8 bit number from textbox choice result. SNES, from 0x7E0200
// <NUMBER 16> 16 bit number from textbox choice result. SNES, from 0x7E0200
// <NUMBER 24> 24 bit number from textbox choice result. SNES, from 0x7E0200
// <NAME_ITM> item name from result value. SNES. from 0x7F0200

// Coliseum related
// <STR> - todo
// <NAME_MON> - todo
// <NAME_TEC> tech name, PC - todo

// other
// <SPCH 11> from the SNES text decoder, should repeat last substring? - todo
// <CT> center horizontally - todo
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
// <NICK_CRO> Crono nickname used by Ayla in the Japanese version
// <NAME_PT1> Party member 1 name
// <NAME_PT2> Party member 2 name
// <NAME_PT3> Party member 3 name
// <NAME_LEENE> always replaced by "Leene", SNES
// <NAME_SIL> name for the Epoch (from "Sil Bird")

// used by choices by the PC version. end tags are ignored, we just want to use the part index
// <S10> Some sort of indentation?
// <C1>x</C1> Choice 1
// <C2>x</C2> Choice 2
// <C3>x</C3> Choice 3
// <C4>x</C4> Choice 4

pub struct TextProcessor {
    replacements: HashMap<String, String>,

    regex_match_parts: Vec<Regex>,

    regex_tag_choice: Regex,
    regex_tag_variable: Regex,
    regex_tag_number_bits: Regex,
}

impl TextProcessor {
    pub fn new() -> TextProcessor {
        TextProcessor {
            replacements: HashMap::new(),

            regex_match_parts: [
                Regex::new(r"^<WAIT>(.+?)</WAIT>").unwrap(),
                Regex::new(r"^<(.+?)>").unwrap(),
                Regex::new(r"^([^<]+)").unwrap(),
            ].to_vec(),

            regex_tag_choice: Regex::new(r"^C(\d{1})$").unwrap(),
            regex_tag_variable: Regex::new(r"<(.+?)>").unwrap(),
            regex_tag_number_bits: Regex::new(r"NUMBER (\d+)").unwrap(),
        }
    }

    pub fn update_party_names(&mut self, party: &Party) {

        // Add names for active party members.
        for (index, character) in party.characters.iter() {
            self.replacements.insert(character.text_key.clone(), character.name.clone());
            if character.party_state == CharacterPartyState::Active {
                self.replacements.insert(format!("NAME_PT{}", index + 1), character.name.clone());
            }
        }

        // Crono nickname.
        self.replacements.insert("NICK_CRO".into(), party.characters[&0].name.clone());

        // Epoch.
        self.replacements.insert("NAME_SIL".into(), "Epoch".into());

        // Unused Queen Leene name.
        self.replacements.insert("NAME_LEENE".into(), "Leene".into());
    }

    pub fn process_dialog_text(&self, text: &str, result_value: u32, result_item: String) -> Vec<TextPage> {
        println!(">>> {}", text);

        // Change PC line breaks to SNES line breaks.
        let text = text.replace("<PAGE>\\", "<PAGE>");
        let text = text.replace("\\", "<BR>");

        // Replace variables.
        let text = self.replace_variables(text);

        // Split text on parts.
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
                PartType::Text => {
                    page.push(TextPart::Text { text: match_contents });
                },
                PartType::TagOpen => {

                    // Hard line break.
                    if match_contents == "BR" {
                        page.push(TextPart::LineBreak);

                    // Indentation.
                    } else if match_contents == "INDENT" {
                        page.push(TextPart::Text { text: "   ".into() });

                    // New page, auto-progress is actually done by <WAIT>.
                    } else if match_contents == "PAGE" || match_contents == "AUTO_PAGE" || match_contents == "AUTO_END" {
                        pages.push(page);
                        page = TextPage::new();

                    // Whitespace?
                    } else if match_contents == "S10" {
                        page.push(TextPart::Text { text: "   ".into() });

                    // Match a number result.
                    } else if match_contents == "NUMBER" {
                        page.push(TextPart::Text { text: result_value.to_string() });

                    // Match a result item name.
                    } else if match_contents == "NAME_ITM" {
                        page.push(TextPart::Text { text: result_item.clone() });

                    // Match a choice option.
                    // <C1>Choice</C1>
                    } else if let Some(captures) = self.regex_tag_choice.captures(&match_contents) {
                        let index: usize = captures[1].parse::<usize>().unwrap() - 1;
                        page.push(TextPart::Choice { index });

                    // Match a sized number result.
                    } else if let Some(captures) = self.regex_tag_number_bits.captures(&match_contents) {
                        let bits: usize = captures[1].parse::<usize>().unwrap();
                        let value = match bits {
                            8 => result_value & 0xFF,
                            16 => result_value & 0xFFFF,
                            24 => result_value & 0xFFFFFF,
                            _ => panic!("Number result bits is {} but must be 8, 16 or 24.", bits),
                        };
                        page.push(TextPart::Text { text: value.to_string() });
                    }
                },

                // Wait a number of ticks.
                PartType::TagWait => {
                    let ticks: u32 = u32::from_str_radix(&match_contents, 16).unwrap();
                    page.push(TextPart::Delay { ticks: (ticks + 1) * 16 });
                },
            };
        }

        if page.len() > 0 {
            pages.push(page);
        }

        self.dump_pages(&pages);

        pages
    }

    fn replace_variables(&self, text: String) -> String {
        let mut new_text = String::with_capacity(text.len());
        let mut last_match = 0;
        for capture in self.regex_tag_variable.captures_iter(&text) {
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

    fn match_part(&self, text: &str, index: usize) -> Option<(PartType, String, usize)> {
        for (regex_index, regex) in self.regex_match_parts.iter().enumerate() {
            let captures = regex.captures_at(&text[index..text.len()], 0);
            if let Some(captures) = captures {
                let len = captures[0].len();
                let contents = captures[1].to_string();
                let part = match regex_index {
                    0 => PartType::TagWait,
                    1 => PartType::TagOpen,
                    2 => PartType::Text,
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
            for part in page.iter() {
                match part {
                    TextPart::Text { ref text } => print!("{}", text),
                    TextPart::LineBreak => println!(),
                    _ => {},
                }
            }
            println!();
        }
        println!("-------------------------------------");
    }
}
