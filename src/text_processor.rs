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

    /// Hard line break.
    LineBreak,

    /// Center this line.
    CenterNextLine,
}

#[derive(Debug, PartialEq)]
enum PartType {
    TagWait,
    TagOpen,
    Text,
}

// Text parts that form a single page.
pub type TextPage = Vec<TextPart>;

pub struct TextProcessor {
    replacements: HashMap<String, String>,

    regex_match_parts: Vec<Regex>,

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

        // Change some PC line break handling to something more sane.
        let text = text.replace("<CT>\\", "<BR><CT>");
        let text = text.replace("<PAGE>\\", "<PAGE>");
        let text = text.replace("\\", "<BR>");

        println!(">>> {}", text);

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

                    // Center this line.
                    } else if match_contents == "CT" {
                        page.push(TextPart::CenterNextLine);

                    // Match a number result.
                    } else if match_contents == "NUMBER" {
                        page.push(TextPart::Text { text: result_value.to_string() });

                    // Match a result item name.
                    } else if match_contents == "NAME_ITM" {
                        page.push(TextPart::Text { text: result_item.clone() });

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
