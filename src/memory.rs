use crate::actor::ActorFlags;
use crate::character::CharacterId;
use crate::Context;
use crate::gamestate::gamestate_scene::SceneState;
use crate::party::CharacterPartyState;
use crate::scene_script::scene_script_decoder::{ActorRef, InputBinding};

#[derive(Clone)]
pub struct Memory {
    pub temp: [u8; 0x400],
    pub global: [u8; 0x200],
    pub local: [u8; 0x200],
    pub extended: [u8; 0x200],
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            temp: [0; 0x400],
            global: [0; 0x200],
            local: [0; 0x200],
            extended: [0; 0x200],
        }
    }

    pub fn write_u8(&mut self, address: usize, value: u8) {
        if address >= 0x7E0000 && address < 0x7E0400 {
            self.temp[address - 0x7E0000] = value;
        } else if address >= 0x7F0000 && address < 0x7F0200 {
            self.global[address - 0x7F0000] = value;
        } else if address >= 0x7F0200 && address < 0x7F0400 {
            self.local[address - 0x7F0200] = value;
        } else if address >= 0x9F0200 && address < 0x9F0400 {
            self.extended[address - 0x9F0200] = value;

        } else if address == 0x0001FA {
            println!("Unimplemented: Set battle music track to {}", value);

        } else if address == 0x7E0BD7 {
            println!("Unimplemented: Set main screen enable flags 0x{:02X}", value);
        } else if address == 0x7E0BD8 {
            println!("Unimplemented: Set sub screen enable flags 0x{:02X}", value);
        } else if address == 0x7F1520 {
            println!("Unimplemented: Set main screen enable flags 0x{:02X} / 0x7F1520", value);
        } else if address == 0x7F1521 {
            println!("Unimplemented: Set sub screen enable flags 0x{:02X} / 0x7F1521", value);

        } else if address == 0x110 {
            println!("Unimplemented: Menu {}.", if value == 0 { "enabled" } else { "disabled" });
        } else if address == 0x111 {
            println!("Unimplemented: Pause {}.", if value == 0 { "enabled" } else { "disabled" });

        } else if address == 0x7E29AE {
            println!("Unimplemented: Set currently playing music to {}.", value);

        } else {
            println!("Unhandled scene script u8 memory write of 0x{:02X} to 0x{:06X}.", value, address)
        }
    }

    pub fn read_u8(&self, address: usize, scene_state: &SceneState) -> u8 {
        if address >= 0x7E0000 && address < 0x7E0400 {
            return self.temp[address - 0x7E0000];
        } else if address >= 0x7F0000 && address < 0x7F0200 {
            return self.global[address - 0x7F0000];
        } else if address >= 0x7F0200 && address < 0x7F0400 {
            return self.local[address - 0x7F0200];
        } else if address >= 0x9F0200 && address < 0x9F0400 {
            return self.extended[address - 0x9F0200];

        // Actor type value.
        } else if address >= 0x7E1100 && address < 0x7E1180 {
            let actor = &scene_state.actors[(address - 0x7E1100) / 2];
            let class = actor.class.to_index();
            let dead = if actor.flags.contains(ActorFlags::DEAD) { 0x80 } else { 0 };
            return class & dead;

        // Player character actor index.
        } else if address == 0x7E2980 {
            return *scene_state.player_actors.get(&0).unwrap_or(&0) as u8;
        } else if address == 0x7E2981 {
            return *scene_state.player_actors.get(&1).unwrap_or(&0) as u8;
        } else if address == 0x7E2982 {
            return *scene_state.player_actors.get(&2).unwrap_or(&0) as u8;
        }

        println!("Unhandled scene script u8 memory read at 0x{:06X}.", address);
        0
    }

    pub fn write_u16(&mut self, address: usize, value: u16) {
        if address >= 0x7E0000 && address < 0x7E0400 {
            self.temp[address - 0x7E0000 + 0] = (value >> 8) as u8;
            self.temp[address - 0x7E0000 + 1] = value as u8;
        } else if address >= 0x7F0000 && address < 0x7F0200 {
            self.global[address - 0x7F0000 + 0] = (value >> 8) as u8;
            self.global[address - 0x7F0000 + 1] = value as u8;
        } else if address >= 0x7F0200 && address < 0x7F0400 {
            self.local[address - 0x7F0200 + 0] = (value >> 8) as u8;
            self.local[address - 0x7F0200 + 1] = value as u8;
        } else if address >= 0x9F0200 && address < 0x9F0400 {
            self.extended[address - 0x9F0200 + 0] = (value >> 8) as u8;
            self.extended[address - 0x9F0200 + 1] = value as u8;
        } else {
            println!("Unhandled scene script u16 memory write of 0x{:04X} to 0x{:06X}.", value, address)
        }
    }

    pub fn read_u16(&self, address: usize, _scene_state: &SceneState) -> u16 {
        if address >= 0x7E0000 && address < 0x7E0400 {
            return self.temp[address - 0x7E0000 + 1] as u16 | self.temp[address - 0x7E0000] as u16 >> 8;
        } else if address >= 0x7F0000 && address < 0x7F0200 {
            return self.global[address - 0x7F0000 + 1] as u16 | self.global[address - 0x7F0000] as u16 >> 8;
        } else if address >= 0x7F0200 && address < 0x7F00400 {
            return self.local[address - 0x7F0200 + 1] as u16 | self.local[address - 0x7F0200] as u16 >> 8;
        } else if address >= 0x9F0200 && address < 0x9F00400 {
            return self.extended[address - 0x9F0200 + 1] as u16 | self.extended[address - 0x9F0200] as u16 >> 8;
        }

        println!("Unhandled scene script u16 memory read at 0x{:06X}.", address);
        0
    }

    pub fn write_bytes(&mut self, address: usize, bytes: &[u8]) {
        for index in 0..bytes.len() {
            self.write_u8(address + index, bytes[index]);
        }
    }
}

/// Source values for data operations.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum DataSource {
    // Immediate value.
    Immediate(i32),

    // From memory.
    Memory(usize),

    // The result value of an actor.
    ActorResult(ActorRef),

    // The current character at the party index.
    PartyCharacter(usize),

    // A flag of an actor.
    ActorFlag(ActorRef, ActorFlags),

    // Button state.
    // Since last check?
    CurrentInput(bool),

    // A specific input.
    Input(InputBinding),

    // Number of items of type in inventory.
    ItemCount(usize),

    // Amount of gold in inventory.
    GoldCount,

    // Player character is recruited/active.
    PCIsActiveOrReserve(CharacterId),
    PCIsActive(CharacterId),
}

impl DataSource {
    pub fn for_temp_memory(address: usize) -> DataSource {
        DataSource::Memory(address + 0x7E0000)
    }

    pub fn for_global_memory(address: usize) -> DataSource {
        DataSource::Memory(address + 0x7F0000)
    }

    pub fn for_local_memory(address: usize) -> DataSource {
        DataSource::Memory(address + 0x7F0200)
    }

    pub fn for_upper_memory(address: usize) -> DataSource {
        DataSource::Memory(address + 0x7F0000)
    }

    pub fn for_extended_memory(address: usize) -> DataSource {
        DataSource::Memory(address + 0x9F0000)
    }

    pub fn get_u8(self, ctx: &Context, scene_state: &SceneState, current_actor: usize) -> u8 {
        match self {
            DataSource::Immediate(value) => value as u8,
            DataSource::Memory(address) => ctx.memory.read_u8(address, scene_state),
            DataSource::ActorResult(actor) => scene_state.actors[actor.deref(scene_state, current_actor)].result as u8,
            DataSource::GoldCount => 0,
            DataSource::ItemCount(..) => 0,
            DataSource::ActorFlag(actor, flags) => (scene_state.actors[actor.deref(scene_state, current_actor)].flags.bits() & flags.bits()) as u8,
            DataSource::PartyCharacter(index) => *scene_state.player_actors.get(&index).unwrap_or(&0) as u8,
            DataSource::Input(..) => 0,
            DataSource::CurrentInput(is_current) => is_current as u8,
            DataSource::PCIsActive(pc) => (ctx.party.characters.get(&pc).unwrap().party_state == CharacterPartyState::Active) as u8,
            DataSource::PCIsActiveOrReserve(pc) => (ctx.party.characters.get(&pc).unwrap().party_state != CharacterPartyState::Unavailable) as u8,
        }
    }

    pub fn get_u16(self, ctx: &Context, scene_state: &SceneState, current_actor: usize) -> u16 {
        match self {
            DataSource::Immediate(value) => value as u16,
            DataSource::Memory(address) => ctx.memory.read_u16(address, scene_state),
            DataSource::ActorResult(actor) => scene_state.actors[actor.deref(scene_state, current_actor)].result as u16,
            DataSource::GoldCount => 0,
            DataSource::ItemCount(..) => 0,
            DataSource::ActorFlag(actor, flags) => (scene_state.actors[actor.deref(scene_state, current_actor)].flags.bits() & flags.bits()) as u16,
            DataSource::PartyCharacter(index) => *scene_state.player_actors.get(&index).unwrap_or(&0) as u16,
            DataSource::Input(..) => 0,
            DataSource::CurrentInput(is_current) => is_current as u16,
            DataSource::PCIsActive(pc) => (ctx.party.characters.get(&pc).unwrap().party_state == CharacterPartyState::Active) as u16,
            DataSource::PCIsActiveOrReserve(pc) => (ctx.party.characters.get(&pc).unwrap().party_state != CharacterPartyState::Unavailable) as u16,
        }
    }
}

/// Destination values for data operations.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum DataDest {
    // To memory.
    Memory(usize),
}

impl DataDest {
    pub fn for_temp_memory(address: usize) -> DataDest {
        DataDest::Memory(address + 0x7E0000)
    }

    pub fn for_global_memory(address: usize) -> DataDest {
        DataDest::Memory(address + 0x7F0000)
    }

    pub fn for_local_memory(address: usize) -> DataDest {
        DataDest::Memory(address + 0x7F0200)
    }

    pub fn for_upper_memory(address: usize) -> DataDest {
        DataDest::Memory(address + 0x7F0000)
    }

    pub fn for_extended_memory(address: usize) -> DataDest {
        DataDest::Memory(address + 0x9F0000)
    }

    pub fn put_u8(&self, ctx: &mut Context, _scene_state: &mut SceneState, value: u8) {
        match self {
            DataDest::Memory(address) => ctx.memory.write_u8(*address, value),
        }
    }

    pub fn put_u16(&self, ctx: &mut Context, _scene_state: &mut SceneState, value: u16) {
        match self {
            DataDest::Memory(address) => ctx.memory.write_u16(*address, value),
        }
    }

    pub fn put_bytes(&self, ctx: &mut Context, _scene_state: &mut SceneState, bytes: [u8; 64], length: usize) {
        match self {
            DataDest::Memory(address) => ctx.memory.write_bytes(*address, &bytes[0..length]),
        }
    }
}
