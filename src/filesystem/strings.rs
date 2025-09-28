use crate::filesystem::filesystem::FileSystem;

impl FileSystem {
    pub fn read_world_exit_names(&self, language: &str) -> Vec<String> {
        self.backend.get_world_exit_names(language)
    }

    pub fn read_scene_names(&self, language: &str) -> Vec<String> {
        self.backend.get_scene_names(language)
    }

    pub fn read_world_names(&self, language: &str) -> Vec<String> {
        let names = self.backend.get_world_names(language);

        // Reorder these so they match world indexes.
        [
            names[0].clone(),
            names[1].clone(),
            names[2].clone(),
            names[3].clone(),
            names[4].clone(),
            names[4].clone(),
            names[4].clone(),
            names[5].clone(),
        ].to_vec()
    }

    pub fn read_item_names(&self, language: &str) -> Vec<String> {
        self.backend.get_item_names(language)
    }

    pub fn read_dialogue_table(&self, address: usize, strings: &mut Vec<String>) {
        println!("Loading dialogue table 0x{:06X}", address);

        strings.clear();
        strings.extend(self.backend.get_dialogue_table(address));
    }
}
