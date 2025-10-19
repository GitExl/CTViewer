use std::collections::HashMap;
use crate::character::{Character, CharacterEquipment, CharacterId, CharacterStats, EquipmentSlot, StatusEffect};
use crate::items::{Item, ItemId};

#[derive(PartialEq, Debug)]
pub enum CharacterPartyState {
    Active,
    InReserve,
    Unavailable,
}

pub struct Party {

    /// All known characters and their state.
    pub characters: HashMap<CharacterId, Character>,

    /// All known items.
    pub items: HashMap<ItemId, Item>,

    /// Items and their amounts held in the party inventory.
    pub inventory: HashMap<ItemId, u32>,

    /// Amount of gold.
    pub gold: u32,
}

impl Party {
    pub fn new() -> Party {
        let mut characters = HashMap::new();

        characters.insert(0, Character {
            id: 0,
            name: "Crono".into(),
            text_key: "NAME_CRO".into(),
            party_state: CharacterPartyState::Active,
            level: 1,
            xp: 0,
            status: StatusEffect::None,
            hp: 70,
            mp: 8,
            stats: CharacterStats {
                evade: 1,
                hit_chance: 8,
                magic: 5,
                magic_defense: 2,
                power: 5,
                stamina: 8,
                speed: 12,
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
            name: "Marle".into(),
            text_key: "NAME_MAR".into(),
            party_state: CharacterPartyState::Active,
            level: 1,
            xp: 0,
            status: StatusEffect::None,
            hp: 65,
            mp: 12,
            stats: CharacterStats {
                evade: 6,
                hit_chance: 8,
                magic: 8,
                magic_defense: 8,
                power: 5,
                stamina: 6,
                speed: 8,
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
            name: "Lucca".into(),
            text_key: "NAME_LUC".into(),
            party_state: CharacterPartyState::Active,
            level: 1,
            xp: 0,
            status: StatusEffect::None,
            hp: 62,
            mp: 12,
            stats: CharacterStats {
                evade: 7,
                hit_chance: 8,
                magic: 8,
                magic_defense: 7,
                power: 2,
                stamina: 6,
                speed: 6,
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
            name: "Frog".into(),
            text_key: "NAME_FRO".into(),
            party_state: CharacterPartyState::Unavailable,
            level: 1,
            xp: 0,
            status: StatusEffect::None,
            hp: 80,
            mp: 9,
            stats: CharacterStats {
                evade: 8,
                hit_chance: 8,
                magic: 6,
                magic_defense: 3,
                power: 4,
                stamina: 8,
                speed: 11,
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
            name: "Robo".into(),
            text_key: "NAME_ROB".into(),
            party_state: CharacterPartyState::Unavailable,
            level: 1,
            xp: 0,
            status: StatusEffect::None,
            hp: 130,
            mp: 6,
            stats: CharacterStats {
                evade: 7,
                hit_chance: 7,
                magic: 3,
                magic_defense: 1,
                power: 7,
                stamina: 10,
                speed: 6,
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
            name: "Ayla".into(),
            text_key: "NAME_AYL".into(),
            party_state: CharacterPartyState::Unavailable,
            level: 1,
            xp: 0,
            status: StatusEffect::None,
            hp: 80,
            mp: 4,
            stats: CharacterStats {
                evade: 12,
                hit_chance: 10,
                magic: 3,
                magic_defense: 1,
                power: 10,
                stamina: 9,
                speed: 13,
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
            name: "Magus".into(),
            text_key: "NAME_MAG".into(),
            party_state: CharacterPartyState::Unavailable,
            level: 1,
            xp: 0,
            status: StatusEffect::None,
            hp: 110,
            mp: 13,
            stats: CharacterStats {
                evade: 10,
                hit_chance: 12,
                magic: 10,
                magic_defense: 9,
                power: 8,
                stamina: 7,
                speed: 12,
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
        }
    }

    pub fn character_add_to_reserve(&mut self, character_id: CharacterId) {
        let character = self.characters.get_mut(&character_id).unwrap();
        if character.party_state == CharacterPartyState::Active {
            return;
        }
        character.party_state = CharacterPartyState::InReserve;
        println!("Added {} to reserve.", character.name);
    }

    pub fn character_remove_from_active(&mut self, character_id: CharacterId) {
        let character = self.characters.get_mut(&character_id).unwrap();
        // todo really removes from active and to out of party
        character.party_state = CharacterPartyState::InReserve;
        println!("Moved {} to reserve.", character.name);
    }

    pub fn character_add_to_active(&mut self, character_id: CharacterId) {
        let character = self.characters.get_mut(&character_id).unwrap();
        character.party_state = CharacterPartyState::Active;
        println!("Moved {} to active.", character.name);
    }

    pub fn character_move_to_reserve(&mut self, character_id: CharacterId) {
        let character = self.characters.get_mut(&character_id).unwrap();
        character.party_state = CharacterPartyState::InReserve;
        println!("Moved {} to reserve.", character.name);
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

    pub fn is_character_recruited(&self, character_id: CharacterId) -> bool {
        if !self.characters.contains_key(&character_id) {
            return false;
        }
        let character = self.characters.get(&character_id).unwrap();
        character.party_state != CharacterPartyState::Unavailable
    }

    pub fn is_character_active(&self, character_id: CharacterId) -> bool {
        if !self.characters.contains_key(&character_id) {
            return false;
        }
        let character = self.characters.get(&character_id).unwrap();
        character.party_state == CharacterPartyState::Active
    }

    pub fn gold_give(&mut self, amount: u32) {
        self.gold += amount;
    }

    pub fn gold_take(&mut self, amount: u32) {
        self.gold -= amount;
    }
}
