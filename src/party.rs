use std::collections::{HashMap, HashSet};
use crate::character::{Character, CharacterEquipment, CharacterId, CharacterStats, EquipmentSlot, StatusEffect};
use crate::items::{Item, ItemId};

pub struct Party {

    /// All known characters and their state.
    pub characters: HashMap<CharacterId, Character>,

    /// Items and their amounts held in the party inventory.
    pub inventory: HashMap<ItemId, u32>,

    /// All known items.
    pub items: HashMap<ItemId, Item>,

    /// Amount of gold.
    pub gold: u32,

    /// Characters in the active party.
    pub active: HashSet<CharacterId>,

    /// Characters not in the active party.
    pub reserve: HashSet<CharacterId>,
}

impl Party {
    pub fn new() -> Party {
        let mut characters = HashMap::new();

        characters.insert(0, Character {
            id: 0,
            name: "Crono".parse().unwrap(),
            level: 1,
            xp: 0,
            status: StatusEffect::None,
            hp: 100,
            mp: 100,
            stats: CharacterStats {
                evade: 1,
                hit_chance: 1,
                magic: 1,
                magic_defense: 1,
                power: 1,
                stamina: 1,
                speed: 1,
            },
            equipment: CharacterEquipment {
                armor: None,
                helmet: None,
                weapon: None,
                accessory: None,
            }
        });

        characters.insert(1, Character {
            id: 1,
            name: "Marle".parse().unwrap(),
            level: 1,
            xp: 0,
            status: StatusEffect::None,
            hp: 100,
            mp: 100,
            stats: CharacterStats {
                evade: 1,
                hit_chance: 1,
                magic: 1,
                magic_defense: 1,
                power: 1,
                stamina: 1,
                speed: 1,
            },
            equipment: CharacterEquipment {
                armor: None,
                helmet: None,
                weapon: None,
                accessory: None,
            }
        });

        characters.insert(2, Character {
            id: 2,
            name: "Lucca".parse().unwrap(),
            level: 1,
            xp: 0,
            status: StatusEffect::None,
            hp: 100,
            mp: 100,
            stats: CharacterStats {
                evade: 1,
                hit_chance: 1,
                magic: 1,
                magic_defense: 1,
                power: 1,
                stamina: 1,
                speed: 1,
            },
            equipment: CharacterEquipment {
                armor: None,
                helmet: None,
                weapon: None,
                accessory: None,
            }
        });

        characters.insert(3, Character {
            id: 3,
            name: "Frog".parse().unwrap(),
            level: 1,
            xp: 0,
            status: StatusEffect::None,
            hp: 100,
            mp: 100,
            stats: CharacterStats {
                evade: 1,
                hit_chance: 1,
                magic: 1,
                magic_defense: 1,
                power: 1,
                stamina: 1,
                speed: 1,
            },
            equipment: CharacterEquipment {
                armor: None,
                helmet: None,
                weapon: None,
                accessory: None,
            }
        });

        characters.insert(4, Character {
            id: 4,
            name: "Robo".parse().unwrap(),
            level: 1,
            xp: 0,
            status: StatusEffect::None,
            hp: 100,
            mp: 100,
            stats: CharacterStats {
                evade: 1,
                hit_chance: 1,
                magic: 1,
                magic_defense: 1,
                power: 1,
                stamina: 1,
                speed: 1,
            },
            equipment: CharacterEquipment {
                armor: None,
                helmet: None,
                weapon: None,
                accessory: None,
            }
        });

        characters.insert(5, Character {
            id: 5,
            name: "Ayla".parse().unwrap(),
            level: 1,
            xp: 0,
            status: StatusEffect::None,
            hp: 100,
            mp: 100,
            stats: CharacterStats {
                evade: 1,
                hit_chance: 1,
                magic: 1,
                magic_defense: 1,
                power: 1,
                stamina: 1,
                speed: 1,
            },
            equipment: CharacterEquipment {
                armor: None,
                helmet: None,
                weapon: None,
                accessory: None,
            }
        });

        characters.insert(6, Character {
            id: 6,
            name: "Magus".parse().unwrap(),
            level: 1,
            xp: 0,
            status: StatusEffect::None,
            hp: 100,
            mp: 100,
            stats: CharacterStats {
                evade: 1,
                hit_chance: 1,
                magic: 1,
                magic_defense: 1,
                power: 1,
                stamina: 1,
                speed: 1,
            },
            equipment: CharacterEquipment {
                armor: None,
                helmet: None,
                weapon: None,
                accessory: None,
            }
        });

        Party {
            characters,

            inventory: HashMap::new(),
            items: HashMap::new(),
            gold: 0,

            active: HashSet::new(),
            reserve: HashSet::new(),
        }
    }

    pub fn character_add_to_reserve(&mut self, character_id: CharacterId) {
        if self.active.contains(&character_id) {
            return;
        }
        self.reserve.insert(character_id);
    }

    pub fn character_remove_from_active(&mut self, character_id: CharacterId) {
        self.active.remove(&character_id);
    }

    pub fn character_add_to_active(&mut self, character_id: CharacterId) {
        self.active.insert(character_id);
        self.reserve.remove(&character_id);
    }

    pub fn character_move_to_reserve(&mut self, character_id: CharacterId) {
        self.reserve.insert(character_id);
        self.active.remove(&character_id);
    }

    pub fn character_equip(&mut self, character_id: CharacterId, slot: EquipmentSlot, item_id: Option<ItemId>) {
        let character = self.characters.get_mut(&character_id).unwrap();
        match slot {
            EquipmentSlot::Weapon => character.equipment.weapon = item_id,
            EquipmentSlot::Helmet => character.equipment.helmet = item_id,
            EquipmentSlot::Armor => character.equipment.armor = item_id,
            EquipmentSlot::Accessory => character.equipment.accessory = item_id,
        };
    }

    pub fn is_character_recruited(&self, character: CharacterId) -> bool {
        self.active.contains(&character) || self.reserve.contains(&character)
    }

    pub fn is_character_active(&self, character: CharacterId) -> bool {
        self.active.contains(&character)
    }

    pub fn gold_give(&mut self, amount: u32) {
        self.gold += amount;
    }

    pub fn gold_take(&mut self, amount: u32) {
        self.gold -= amount;
    }
}
