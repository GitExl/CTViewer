use crate::actor::ActorFlags;
use crate::character::CharacterId;
use crate::Context;
use crate::gamestate::gamestate_scene::SceneState;
use crate::gamestate::gamestate_world::WorldState;
use crate::party::CharacterPartyState;
use crate::scene_script::scene_script_decoder::{ActorRef, InputBinding};

#[derive(Clone)]
pub struct Memory {
    pub system: [u8; 0x10000],
    pub global: [u8; 0x200],
    pub local: [u8; 0x200],
    pub extended: [u8; 0x200],
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            system: [0; 0x10000],
            global: [0; 0x200],
            local: [0; 0x200],
            extended: [0; 0x200],
        }
    }

    pub fn clear_local(&mut self) {
        self.local = [0; 0x200];
    }

    pub fn write_u8(&mut self, address: usize, value: u8) {
        if address >= 0x7E0000 && address < 0x7F0000 {
            self.system[address - 0x7E0000] = value;
        } else if address >= 0x7F0000 && address < 0x7F0200 {
            self.global[address - 0x7F0000] = value;
        } else if address >= 0x7F0200 && address < 0x7F0400 {
            self.local[address - 0x7F0200] = value;
        } else if address >= 0x9F0200 && address < 0x9F0400 {
            self.extended[address - 0x9F0200] = value;
        } else {
            println!("Unhandled u8 memory write of 0x{:02X} to 0x{:06X}.", value, address)
        }
    }

    pub fn write_scene_u8(&mut self, address: usize, value: u8, _scene_state: &SceneState) {
        if address == 0x0001FA {
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

        // Fall-through to memory.
        } else {
            self.write_u8(address, value);
        }
    }

    pub fn read_u8(&self, address: usize) -> u8 {
        if address >= 0x7E0000 && address < 0x7F0000 {
            return self.system[address - 0x7E0000];
        } else if address >= 0x7F0000 && address < 0x7F0200 {
            return self.global[address - 0x7F0000];
        } else if address >= 0x7F0200 && address < 0x7F0400 {
            return self.local[address - 0x7F0200];
        } else if address >= 0x9F0200 && address < 0x9F0400 {
            return self.extended[address - 0x9F0200];
        }

        println!("Unhandled u8 memory read at 0x{:06X}.", address);
        0
    }

    pub fn read_scene_u8(&self, address: usize, scene_state: &SceneState) -> u8 {

        // Actor type value.
        if address >= 0x7E1100 && address < 0x7E1180 {
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

        // Fall-through to memory.
        self.read_u8(address)
    }

    pub fn read_world_u8(&self, address: usize, _world_state: &WorldState) -> u8 {

        // Fall-through to memory.
        self.read_u8(address)
    }

    pub fn write_u16(&mut self, address: usize, value: u16) {
        if address >= 0x7E0000 && address < 0x7F0000 {
            self.system[address - 0x7E0000 + 0] = (value >> 8) as u8;
            self.system[address - 0x7E0000 + 1] = value as u8;
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
            println!("Unhandled u16 memory write of 0x{:04X} to 0x{:06X}.", value, address)
        }
    }

    pub fn read_u16(&self, address: usize, _scene_state: &SceneState) -> u16 {
        if address >= 0x7E0000 && address < 0x7F0000 {
            return self.system[address - 0x7E0000 + 1] as u16 | self.system[address - 0x7E0000] as u16 >> 8;
        } else if address >= 0x7F0000 && address < 0x7F0200 {
            return self.global[address - 0x7F0000 + 1] as u16 | self.global[address - 0x7F0000] as u16 >> 8;
        } else if address >= 0x7F0200 && address < 0x7F0400 {
            return self.local[address - 0x7F0200 + 1] as u16 | self.local[address - 0x7F0200] as u16 >> 8;
        } else if address >= 0x9F0200 && address < 0x9F0400 {
            return self.extended[address - 0x9F0200 + 1] as u16 | self.extended[address - 0x9F0200] as u16 >> 8;
        }

        println!("Unhandled u16 memory read at 0x{:06X}.", address);
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

    // World actor memory.
    WorldActor(usize),
}

impl DataSource {
    pub fn for_system_memory(address: usize) -> DataSource {
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

    pub fn for_world_actor_memory(address: usize) -> DataSource {
        DataSource::WorldActor(address)
    }

    pub fn get_scene_u8(self, ctx: &Context, scene_state: &SceneState, current_actor: usize) -> u8 {
        match self {
            DataSource::Immediate(value) => value as u8,
            DataSource::Memory(address) => ctx.memory.read_scene_u8(address, scene_state),

            // Scene
            DataSource::ActorResult(actor) => scene_state.actors[actor.deref(scene_state, current_actor)].result as u8,
            DataSource::GoldCount => 0,
            DataSource::ItemCount(..) => 0,
            DataSource::ActorFlag(actor, flags) => (scene_state.actors[actor.deref(scene_state, current_actor)].flags.bits() & flags.bits()) as u8,
            DataSource::PartyCharacter(index) => *scene_state.player_actors.get(&index).unwrap_or(&0) as u8,
            DataSource::Input(..) => 0,
            DataSource::CurrentInput(is_current) => is_current as u8,
            DataSource::PCIsActive(pc) => (ctx.party.characters.get(&pc).unwrap().party_state == CharacterPartyState::Active) as u8,
            DataSource::PCIsActiveOrReserve(pc) => (ctx.party.characters.get(&pc).unwrap().party_state != CharacterPartyState::Unavailable) as u8,

            _ => panic!("Unhandled get_u8 scene."),
        }
    }

    pub fn get_world_u8(self, ctx: &Context, world_state: &WorldState, _current_actor: usize) -> u8 {
        match self {
            DataSource::Immediate(value) => value as u8,
            DataSource::Memory(address) => ctx.memory.read_world_u8(address, world_state),

            // World
            DataSource::WorldActor(_actor_address) => 0,

            _ => panic!("Unhandled get_u8 for world."),
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

            DataSource::WorldActor(_actor_address) => 0,
        }
    }

    pub fn as_string(&self) -> String {
        match self {
            DataSource::Immediate(value) => format!("{}", value),
            DataSource::Memory(address) => format!("0x{:06X}", address),
            DataSource::ActorResult(actor) => format!("ActorResult({:?})", actor),
            DataSource::GoldCount => String::from("Gold"),
            DataSource::ItemCount(item) => format!("ItemCount({})", item),
            DataSource::ActorFlag(actor, flags) => format!("ActorFlag({:?}, 0x{:02X})", actor, flags),
            DataSource::PartyCharacter(index) => format!("PC({})", index),
            DataSource::Input(binding) => format!("Input({:?})", binding),
            DataSource::CurrentInput(is_current) => format!("CurrentInput({})", is_current),
            DataSource::PCIsActive(pc) => format!("PCIsActive({})", pc),
            DataSource::PCIsActiveOrReserve(pc) => format!("PCIsActive({})", pc),
            DataSource::WorldActor(address) => format!("self.0x{:02X}", address),
        }
    }
}

/// Destination values for data operations.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum DataDest {
    Memory(usize),
    WorldActor(usize),
}

impl DataDest {
    pub fn for_system_memory(address: usize) -> DataDest {
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

    pub fn for_world_actor_memory(address: usize) -> DataDest {
        DataDest::WorldActor(address)
    }

    pub fn put_u8(&self, ctx: &mut Context, value: u8) {
        match self {
            DataDest::Memory(address) => ctx.memory.write_u8(*address, value),
            _ => panic!("Unhandled put_u8"),
        }
    }

    pub fn put_scene_u8(&self, ctx: &mut Context, _scene_state: &mut SceneState, value: u8) {
        match self {
            _ => self.put_u8(ctx, value),
        }
    }

    pub fn put_world_u8(&self, ctx: &mut Context, _world_state: &mut WorldState, value: u8) {
        match self {
            _ => self.put_u8(ctx, value),
        }
    }

    pub fn put_u16(&self, ctx: &mut Context, _scene_state: &mut SceneState, value: u16) {
        match self {
            DataDest::Memory(address) => ctx.memory.write_u16(*address, value),
            DataDest::WorldActor(_address) => {},
        }
    }

    pub fn put_bytes(&self, ctx: &mut Context, _scene_state: &mut SceneState, bytes: [u8; 64], length: usize) {
        match self {
            DataDest::Memory(address) => ctx.memory.write_bytes(*address, &bytes[0..length]),
            DataDest::WorldActor(_address) => {},
        }
    }

    pub fn as_string(&self) -> String {
        match self {
            DataDest::Memory(address) => format!("0x{:06X}", address),
            DataDest::WorldActor(address) => format!("self.0x{:02X}", address),
        }
    }
}
