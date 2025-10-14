use std::collections::HashSet;
use bitflags::bitflags;
use crate::character::{CharacterId, EquipmentSlot, StatusEffect};

pub type ItemId = usize;
pub type ItemModifierId = usize;

bitflags! {
    #[derive(Clone, Default, Copy, Debug, PartialEq)]
    pub struct ItemFlags: u32 {
        const UNKNOWN_01 = 0x0001;
        const USABLE_IN_MENU = 0x0002;
        const CANNOT_SELL = 0x0004;
        const QUEST = 0x0008;
        const NO_NEW_GAME_PLUS = 0x0010;
        const UNKNOWN_20 = 0x0020;
        const UNKNOWN_40 = 0x0040;
        const USABLE_IN_BATTLE = 0x0040;
    }
}

bitflags! {
    #[derive(Clone, Default, Copy, Debug, PartialEq)]
    pub struct ItemAccessoryFlags: u32 {
        const AFFECTS_GOLD = 0x0001;
        const UNKNOWN_0002 = 0x0002;
        const UNKNOWN_0004 = 0x0004;
        const UNKNOWN_0008 = 0x0008;
        const UNKNOWN_0010 = 0x0010;
        const AFFECTS_MP = 0x0020;
        const AFFECTS_COUNTER = 0x0040;
        const UNKNOWN_0080 = 0x0080;
        const UNKNOWN_0100 = 0x0100;
        const UNKNOWN_0200 = 0x0200;
        const UNKNOWN_0400 = 0x0400;
        const UNKNOWN_0800 = 0x0800;
        const AFFECTS_HP = 0x1000;
        const UNKNOWN_2000 = 0x2000;
        const AFFECTS_STATS = 0x4000;
        const AFFECTS_STATUS = 0x8000;
    }
}

bitflags! {
    #[derive(Clone, Default, Copy, Debug, PartialEq)]
    pub struct ItemConsumableFlags: u32 {
        const TARGET_ALL = 0x01;
        const REMOVE_ALL_STATUSES = 0x02;
        const UNKNOWN = 0x04;
        const REVIVES = 0x08;
        const REQUIRE_SAVE_POINT = 0x10;
    }
}

pub enum ModifierCondition {
    EnemyType {
        is_type: usize,
    },
    SightScopeRelicFail,
    BossDeath,
}

pub enum Element {
    Lightning,
    Ice,
    Fire,
    Water,
    Shadow,
}

pub enum ModifierEffect {
    CritFactor {
        factor: f64,
    },
    DamageFactor {
        factor: f64,
    },
    MagicFactor {
        factor: f64,
    },
    MagicDamageFactor {
        factor: f64,
    },
    RandomDamageHealthDigit2,
    IsMagical,
    ReduceHealthAbsoluteChance {
        chance: f64,
        health: u32,
    },
    CharmChance {
        chance: f64,
    },
    RemoveAllStatuses,
    RemoveStatuses {
        statuses: HashSet<StatusEffect>,
    },
    DeadAllyDamageMultiplier {
        factor: u32,
    },
    HalfHealthDamageChance {
        chance: f64,
    },
    LastHealthDigitDamageDivisor {
        divisor: u32,
    },
    DeathChance {
        chance: f64,
        ignore_immunities: bool,
    },
    InflictStatusChance {
        chance: u32,
    },
    InflictRandomStatusChance {
        chance: f64,
    },
    RemoveAllImmunities,
    GetAllImmunities,
    GetImmunities {
        immunities: HashSet<StatusEffect>,
    },
    RemoveImmunities {
        immunities: HashSet<StatusEffect>,
    },
    CritDamageAbsolute {
        damage: u32,
    },
    Affinities {
        affinities: HashSet<Element>,
    },
    GetStatuses {
        statuses: HashSet<StatusEffect>,
    },
    DamageAbsolute {
        damage: u32,
    },
}

pub struct ItemModifier {
    id: ItemModifierId,
    name_str: usize,
    conditions: Vec<ModifierCondition>,
    effects: Vec<ModifierEffect>,
}

pub enum ItemCategory {
    Weapon {
        attack: u32,
        crit_chance: f64,
        modifier: ItemModifierId,
        palette: usize,
        sound1: usize,
        sound2: usize,
    },
    Armor {
        defense: u32,
        protection: HashSet<Element>,
        protection_factor: f64,
        modifier: ItemModifierId,
    },
    Helmet {
        defense: u32,
        protection: HashSet<Element>,
        protection_factor: f64,
        modifier: ItemModifierId,
    },
    Accessory {
        flags: ItemAccessoryFlags,
        counter_chance: u32,
        statuses: HashSet<StatusEffect>,
        immunities: HashSet<StatusEffect>,
    },
    Consumable {
        flags: ItemConsumableFlags,
        status_add: HashSet<StatusEffect>,
        status_remove: HashSet<StatusEffect>,
    },
    Quest,
}

pub struct Item {
    pub id: ItemId,
    pub category: ItemCategory,
    pub slot: EquipmentSlot,
    pub name_str: usize,
    pub description_str: usize,
    pub price: u32,
    pub flags: ItemFlags,
    pub equipped_by: HashSet<CharacterId>,
    pub modifier: ItemModifierId,
}
