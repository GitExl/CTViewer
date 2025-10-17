use regex::Regex;
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

pub enum TextPartContents {
    /// A single word.
    Word {
        word: String
    },

    /// Wait for n ticks.
    Delay {
        ticks: usize,
    },

    /// A single space.
    Space,

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
    contents: TextPartContents,
}

pub struct TextPage {
    parts: Vec<TextPart>,
    auto: bool,
    choice_lines: [usize; 4],
}

// icons not part of the TTF font, need to handle these separately
// <BLADE> <BOW> <GUN> <ARM> <SWORD> <FIST> <SCYTHE> <HELM> <ARMOR> <RING>
//
// probably no need to implement these?
// <H> <M> <P>
// <SHIELD> <STAR> <LEFT> <RIGHT>
// <HAND1> <HAND2> <HAND3> <HAND4>
// <H> <M> <P>
// <HP0> <HP1> <HP2> <HP3> <HP4> <HP5> <HP6> <HP7> <HP8>
// <D> <Z> <UP> <A> <L> <R>
// <H> <M> <P>
// <CORNER>
//
// text flow
// <STOP> ?
// <STOP LINE BREAK> ?
// <INSTANT LINE BREAK> ?
// <AUTO_PAGE> Automatically go to next page after x time?
// <AUTO_END> ?
// <PAGE> New dialog page
// <BR> Hard line break
// <WAIT>00</WAIT> Wait for 00 ticks

// data
// <NUMBER 8> 8 bit number from somewhere
// <NUMBER 16> 16 bit number from somewhere
// <NUMBER 24> 24 bit number from somewhere
// <RESULT ITEM> item name from result value?
// <NAME_TEC>00</NAME_TEC> tech name 00

// other
// <SPCH 11> from the SNES text decoder, should repeat last substring so should never appear here...
// <CT> center horizontally?
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
// <NICK_CRO> Crono nickname used by Ayla
// <NAME_PT1> Party member 1 name
// <NAME_PT2> Party member 2 name
// <NAME_PT3> Party member 3 name
// <NAME_LEENE> from SNES version, always replaced by "Leene"
// <NAME_ITM> item name, from where?
// <NAME_SIL> name for the Epoch ("Sil Bird")

// used by choices by the PC version, end tags can probably be ignored if we want to just use line
// numbers?
// <S10> Some sort of indentation?
// <C1>x</C1> Choice 1
// <C2>x</C2> Choice 2
// <C3>x</C3> Choice 3
// <C4>x</C4> Choice 4

pub fn process_text(text: &str) -> Vec<TextPart> {
    let text = text.replace("\\", "<BR>");
    println!("{}", text);

    let mut index = 0;
    loop {
        let result = match_part(&text, index);
        if result.is_none() {
            break;
        }

        let (match_type, match_contents) = result.unwrap();
        index += match_contents.len();

        if match_type == MatchType::TagOpen && tag_has_close(match_contents) {
            // loop {
            //     let result = match_part(&text, index);
            // }
        }
    }

    Vec::new()
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
            // println!("{:?}", matches);
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
        "<NAME_TEC>" => true,
        _ => false,
    }
}
