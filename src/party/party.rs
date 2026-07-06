use std::collections::HashMap;
use crate::party::character::{Character, CharacterEquipment, CharacterId, CharacterStats, EquipmentSlot, StatusEffect};
use crate::party::items::{Item, ItemId};

#[derive(Clone, Copy)]
pub struct PartySlot {
    pub character_id: CharacterId,
    pub disabled: bool,
}

impl PartySlot {
    fn new(character_id: CharacterId, disabled: bool) -> Self {
        Self { character_id, disabled }
    }
}

const PARTY_LEN: usize = 3;

pub struct Party {

    /// All known characters and their state.
    characters: HashMap<CharacterId, Character>,

    /// List and order of characters in the current party.
    party_slots: Vec<PartySlot>,

    /// All known items.
    items: HashMap<ItemId, Item>,

    /// Items and their amounts held in the party inventory.
    inventory: HashMap<ItemId, u32>,

    /// Amount of gold.
    gold: u32,
}

impl Party {
    pub fn new() -> Party {
        let mut characters = HashMap::new();

        characters.insert(0, Character {
            id: 0,
            name: "Crono".into(),
            text_key: "NAME_CRO".into(),
            recruited: true,
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
            recruited: true,
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
            recruited: true,
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
            name: "Robo".into(),
            text_key: "NAME_ROB".into(),
            recruited: true,
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

        characters.insert(4, Character {
            id: 4,
            name: "Frog".into(),
            text_key: "NAME_FRO".into(),
            recruited: true,
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

        characters.insert(5, Character {
            id: 5,
            name: "Ayla".into(),
            text_key: "NAME_AYL".into(),
            recruited: true,
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
            recruited: true,
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
            party_slots: vec![
                PartySlot::new(3, false),
                PartySlot::new(1, false),
                PartySlot::new(2, false),
                PartySlot::new(6, false),
                PartySlot::new(4, false),
                PartySlot::new(5, true),
                PartySlot::new(0, true),
            ],
            inventory: HashMap::new(),
            items: HashMap::new(),
            gold: 0,
        }
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

    pub fn find_character_index(&self, character_id: CharacterId) -> Option<usize> {
        for (index, slot) in self.party_slots.iter().enumerate() {
            if !slot.disabled && slot.character_id == character_id {
                return Some(index);
            }
        }
        None
    }

    pub fn find_active_character_index(&self, character_id: CharacterId) -> Option<usize> {
        for index in 0..PARTY_LEN {
            let slot = &self.party_slots[index];
            if !slot.disabled && slot.character_id == character_id {
                return Some(index);
            }
        }
        None
    }

    pub fn deactivate_character(&mut self, character_id: CharacterId) {
        if let Some(index) = self.find_character_index(character_id) {
            self.party_slots[index].disabled = true;
        }
    }

    pub fn recruit_character_at_index(&mut self, index: usize, character_id: CharacterId) {
        self.characters.get_mut(&character_id).unwrap().recruited = true;
        self.party_slots[index].disabled = false;
    }

    fn insert_slot_at_top_of_reserve(&mut self, slot: PartySlot) {
        let mut current_slot = slot;
        for i in PARTY_LEN..9 {
            let next_slot = self.party_slots[i];
            self.party_slots[i] = current_slot;
            if next_slot.disabled {
                return;
            }
            current_slot = next_slot;
        }
    }

    pub fn add_character_to_party(&mut self, character_id: CharacterId) {
        self.deactivate_character(character_id);

        // Look for empty slot in party.
        for i in 0..PARTY_LEN {
            let slot = self.party_slots[i];
            if slot.disabled && slot.character_id == character_id {
                self.recruit_character_at_index(i, character_id);
                return;
            }
        }

        // No empty slot found: Shift party and push index 2 into reserve
        let old_slot_state_end = self.party_slots[PARTY_LEN - 1];
        self.party_slots[2] = PartySlot::new(character_id, false);
        self.insert_slot_at_top_of_reserve(old_slot_state_end);
    }

    pub fn move_character_from_party_into_reserve(&mut self, character_id: CharacterId) {
        let index = match self.find_active_character_index(character_id) {
            Some(i) => i,
            None => return,
        };

        // Shift active party members up to fill the gap
        for i in index..PARTY_LEN - 1 {
            self.party_slots[i] = self.party_slots[i + 1];
        }

        // Mark the last party slot as inactive and move removed char to reserve
        self.party_slots[PARTY_LEN - 1].disabled = true;
        self.insert_slot_at_top_of_reserve(PartySlot::new(character_id, false));
    }

    pub fn add_character_to_reserve(&mut self, character_id: CharacterId) {
        if self.find_character_index(character_id).is_some() {
            return;
        }

        // Look for first available position in reserve.
        for i in PARTY_LEN..9 {
            let slot = self.party_slots[i];
            if slot.disabled {
                self.recruit_character_at_index(i, character_id);
                return;
            }
        }
    }

    pub fn is_character_recruited(&self, character_id: CharacterId) -> bool {
        if let Some(character) = self.characters.get(&character_id) {
            return character.recruited;
        }
        false
    }

    pub fn gold_give(&mut self, amount: u32) {
        self.gold += amount;
    }

    pub fn gold_take(&mut self, amount: u32) {
        self.gold -= amount;
    }

    pub fn get_characters_iter(&self) -> impl Iterator<Item = &Character> + '_ {
        self.characters.values()
    }

    pub fn get_character(&self, character_id: CharacterId) -> Option<&Character> {
        self.characters.get(&character_id)
    }

    pub fn get_party_slots(&self) -> impl Iterator<Item = &PartySlot> + '_ {
        self.party_slots.iter()
    }

    pub fn get_active_party_slots(&self) -> impl Iterator<Item = &PartySlot> + '_ {
        self.party_slots[0..PARTY_LEN].iter()
    }

}
