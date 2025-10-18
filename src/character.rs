use crate::items::ItemId;
use crate::party::CharacterPartyState;

pub type CharacterId = usize;
pub type StatsModifierId = usize;

pub enum StatusEffect {
    None,
    Berserk,
    Haste,
    Lifeline,
    Safe,
    Shield,
    AttackUp,
    MaxAttackUp,
    Seraph,
    Chaos,
    Darkness,
    Lock,
    Poison,
    Sleep,
    Slow,
    Stop,
}

pub enum Stat {
    Power,
    Speed,
    HitChance,
    Evade,
    Magic,
    Stamina,
    MagicDefense,
}

pub struct StatsModifierStat {
    stat: Stat,
    modifier: i32,
}

pub struct StatsModifier {
    id: StatsModifierId,
    modifiers: Vec<StatsModifierStat>,
}

pub enum EquipmentSlot {
    Weapon,
    Armor,
    Helmet,
    Accessory,
}

pub struct CharacterEquipment {
    pub weapon: Option<ItemId>,
    pub armor: Option<ItemId>,
    pub helmet: Option<ItemId>,
    pub accessory: Option<ItemId>,
}

pub struct CharacterStats {
    pub power: u32,
    pub speed: u32,
    pub hit_chance: u32,
    pub evade: u32,
    pub magic: u32,
    pub stamina: u32,
    pub magic_defense: u32,
}

pub struct Character {
    pub id: CharacterId,
    pub name: String,
    pub text_key: String,
    pub party_state: CharacterPartyState,
    pub hp: u32,
    pub mp: u32,
    pub xp: u32,
    pub level: u32,
    pub stats: CharacterStats,
    pub equipment: CharacterEquipment,
    pub status: StatusEffect,
}
