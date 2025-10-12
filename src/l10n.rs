use std::collections::HashMap;
use crate::filesystem::filesystem::FileSystem;

#[repr(usize)]
pub enum IndexedType {
    WorldExit,
    Scene,
    World,
    Item,
}

static INDEXED_ITER: [IndexedType; 4] = [
    IndexedType::WorldExit,
    IndexedType::Scene,
    IndexedType::World,
    IndexedType::Item,
];

pub struct L10n<'a> {
    language: &'a str,
    indexed_strings: Vec<Vec<String>>,
    strings: HashMap<String, String>,
}

impl L10n<'_> {
    pub fn new<'a>(language: &'a str, fs: &FileSystem) -> L10n<'a> {
        let mut indexed_strings = Vec::new();
        for index_type in &INDEXED_ITER {
            indexed_strings.push(match index_type {
                IndexedType::WorldExit => fs.read_world_exit_names(language),
                IndexedType::Scene => fs.read_scene_names(language),
                IndexedType::World => fs.read_world_names(language),
                IndexedType::Item => fs.read_item_names(language),
            });
        }

        let strings: HashMap<String, String> = HashMap::new();

        L10n {
            language,
            indexed_strings,
            strings,
        }
    }

    pub fn get_language(&self) -> &str {
        self.language
    }

    pub fn get_indexed(&self, indexed_type: IndexedType, index: usize) -> String {
        let source = &self.indexed_strings[indexed_type as usize];
        if index >= source.len() {
            return format!("IDX_{}", index);
        }

        source[index].clone()
    }

    pub fn get_keyed(&self, key: &String) -> String {
        self.strings.get(key).unwrap_or_else(|| key).clone()
    }
}
