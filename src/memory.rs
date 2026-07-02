use crate::scene::actor::SceneActorFlags;
use crate::character::CharacterId;
use crate::Context;
use crate::gamestate::gamestate_scene::SceneState;
use crate::gamestate::gamestate_world::WorldState;
use crate::party::CharacterPartyState;
use crate::scene_script::scene_script_decoder::{ActorRef, InputBinding};
use crate::util::vec2df64::Vec2Df64;
use crate::world_script::world_actor::WorldActor;

#[derive(Clone)]
pub struct MemoryRegion {
    data: Vec<u8>,
}

impl MemoryRegion {
    pub fn new(size: usize) -> MemoryRegion {
        MemoryRegion {
            data: vec![0; size],
        }
    }

    pub fn clear(&mut self) {
        self.data.fill(0);
    }

    pub fn put_u8(&mut self, address: usize, value: u8) {
        self.data[address] = value;
    }

    pub fn put_i8(&mut self, address: usize, value: i8) {
        self.data[address] = value as u8;
    }

    pub fn get_u8(&self, address: usize) -> u8 {
        self.data[address]
    }

    pub fn get_i8(&self, address: usize) -> i8 {
        self.data[address] as i8
    }

    pub fn put_u16(&mut self, address: usize, value: u16) {
        self.data[address + 0] = (value        & 0xFF) as u8;
        self.data[address + 1] = ((value >> 8) & 0xFF) as u8;
    }

    pub fn get_u16(&self, address: usize) -> u16 {
        self.data[address + 0] as u16 | (self.data[address + 1] as u16) << 8
    }

    pub fn put_u24(&mut self, address: usize, value: u32) {
        self.data[address + 0] = (value         & 0xFF) as u8;
        self.data[address + 1] = ((value >> 8)  & 0xFF) as u8;
        self.data[address + 2] = ((value >> 16) & 0xFF) as u8;
    }

    pub fn get_u24(&self, address: usize) -> u32 {
        self.data[address + 0] as u32 | (self.data[address + 1] as u32) << 8 | (self.data[address + 2] as u32) << 16
    }

    pub fn put_u32(&mut self, address: usize, value: u32) {
        self.data[address + 0] = (value         & 0xFF) as u8;
        self.data[address + 1] = ((value >> 8)  & 0xFF) as u8;
        self.data[address + 2] = ((value >> 16) & 0xFF) as u8;
        self.data[address + 3] = ((value >> 24) & 0xFF) as u8;
    }

    pub fn get_u32(&self, address: usize) -> u32 {
        self.data[address + 0] as u32 | (self.data[address + 1] as u32) << 8 | (self.data[address + 2] as u32) << 16 | (self.data[address + 3] as u32) << 24
    }
}

pub struct Memory {
    pub system: MemoryRegion,
    pub global: MemoryRegion,
    pub local: MemoryRegion,
    pub extended: MemoryRegion,
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            system: MemoryRegion::new(0x10000),
            global: MemoryRegion::new(0x200),
            local: MemoryRegion::new(0x200),
            extended: MemoryRegion::new(0x200),
        }
    }

    pub fn clear_local(&mut self) {
        self.local.clear()
    }

    pub fn put_u8(&mut self, address: usize, value: u8) {
        if address >= 0x7E0000 && address < 0x7F0000 {
            self.system.put_u8(address - 0x7E0000, value);
        } else if address >= 0x7F0000 && address < 0x7F0200 {
            self.global.put_u8(address - 0x7F0000, value);
        } else if address >= 0x7F0200 && address < 0x7F0400 {
            self.local.put_u8(address - 0x7F0200, value);
        } else if address >= 0x9F0200 && address < 0x9F0400 {
            self.extended.put_u8(address - 0x9F0200, value);
        } else {
            println!("Unhandled u8 memory write of 0x{:02X} to 0x{:06X}.", value, address)
        }
    }

    pub fn put_scene_u8(&mut self, address: usize, value: u8, _scene_state: &SceneState) {
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
            self.put_u8(address, value);
        }
    }

    pub fn get_u8(&self, address: usize) -> u8 {
        if address >= 0x7E0000 && address < 0x7F0000 {
            return self.system.get_u8(address - 0x7E0000);
        } else if address >= 0x7F0000 && address < 0x7F0200 {
            return self.global.get_u8(address - 0x7F0000);
        } else if address >= 0x7F0200 && address < 0x7F0400 {
            return self.local.get_u8(address - 0x7F0200);
        } else if address >= 0x9F0200 && address < 0x9F0400 {
            return self.extended.get_u8(address - 0x9F0200);
        }

        println!("Unhandled u8 memory read at 0x{:06X}.", address);
        0
    }

    pub fn get_scene_u8(&self, address: usize, scene_state: &SceneState) -> u8 {

        // Actor type value.
        if address >= 0x7E1100 && address < 0x7E1180 {
            let actor = &scene_state.actors[(address - 0x7E1100) / 2];
            let class = actor.class.to_index();
            let dead = if actor.flags.contains(SceneActorFlags::DEAD) { 0x80 } else { 0 };
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
        self.get_u8(address)
    }

    pub fn put_u16(&mut self, address: usize, value: u16) {
        if address >= 0x7E0000 && address < 0x7F0000 {
            self.system.put_u16(address - 0x7E0000, value);
        } else if address >= 0x7F0000 && address < 0x7F0200 {
            self.global.put_u16(address - 0x7F0000, value);
        } else if address >= 0x7F0200 && address < 0x7F0400 {
            self.local.put_u16(address - 0x7F0200, value);
        } else if address >= 0x9F0200 && address < 0x9F0400 {
            self.extended.put_u16(address - 0x9F0200, value);
        } else {
            println!("Unhandled u16 memory write of 0x{:04X} to 0x{:06X}.", value, address)
        }
    }

    pub fn get_u16(&self, address: usize, _scene_state: &SceneState) -> u16 {
        if address >= 0x7E0000 && address < 0x7F0000 {
            return self.system.get_u16(address - 0x7E0000);
        } else if address >= 0x7F0000 && address < 0x7F0200 {
            return self.global.get_u16(address - 0x7F0000);
        } else if address >= 0x7F0200 && address < 0x7F0400 {
            return self.local.get_u16(address - 0x7F0200);
        } else if address >= 0x9F0200 && address < 0x9F0400 {
            return self.extended.get_u16(address - 0x9F0200);
        }

        println!("Unhandled u16 memory read at 0x{:06X}.", address);
        0
    }

    pub fn put_bytes(&mut self, address: usize, bytes: &[u8]) {
        for index in 0..bytes.len() {
            self.put_u8(address + index, bytes[index]);
        }
    }

    pub fn get_bytes(&mut self, address: usize, count: usize) -> Vec<u8> {
        let mut out = vec![0u8; count];
        for index in 0..count {
            out[index] = self.get_u8(address + index);
        }
        out
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
    ActorFlag(ActorRef, SceneActorFlags),

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
            DataSource::Memory(address) => ctx.memory.get_scene_u8(address, scene_state),

            // Scene
            DataSource::ActorResult(actor) => scene_state.actors[actor.deref(scene_state, current_actor)].result as u8,
            DataSource::GoldCount => 0,
            DataSource::ItemCount(..) => 0,
            DataSource::ActorFlag(actor, flags) => (scene_state.actors[actor.deref(scene_state, current_actor)].flags.bits() & flags.bits()) as u8,
            DataSource::PartyCharacter(index) => *scene_state.player_actors.get(&index).unwrap_or(&0) as u8,
            DataSource::Input(..) => 0,
            DataSource::CurrentInput(is_current) => is_current as u8,
            DataSource::PCIsActive(pc) => (ctx.party.characters.get(&pc).unwrap().party_state == CharacterPartyState::Available) as u8,
            DataSource::PCIsActiveOrReserve(pc) => (ctx.party.characters.get(&pc).unwrap().party_state != CharacterPartyState::Unavailable) as u8,

            _ => panic!("Unhandled get_u8 scene."),
        }
    }

    pub fn get_world_u8(self, ctx: &Context, _world_state: &WorldState, actor: &mut WorldActor) -> u8 {
        match self {
            DataSource::Immediate(value) => value as u8,
            DataSource::Memory(address) => ctx.memory.get_u8(address),

            // World
            DataSource::WorldActor(address) => {
                if address == 0x0F {
                    actor.palette_priority
                } else {
                    actor.memory.get_u8(address)
                }
            },

            _ => panic!("Unhandled get_u8 for world."),
        }
    }

    pub fn get_u16(self, ctx: &Context, scene_state: &SceneState, current_actor: usize) -> u16 {
        match self {
            DataSource::Immediate(value) => value as u16,
            DataSource::Memory(address) => ctx.memory.get_u16(address, scene_state),
            DataSource::ActorResult(actor) => scene_state.actors[actor.deref(scene_state, current_actor)].result as u16,
            DataSource::GoldCount => 9999,
            DataSource::ItemCount(..) => 1,
            DataSource::ActorFlag(actor, flags) => (scene_state.actors[actor.deref(scene_state, current_actor)].flags.bits() & flags.bits()) as u16,
            DataSource::PartyCharacter(index) => *scene_state.player_actors.get(&index).unwrap_or(&0) as u16,
            DataSource::Input(..) => 0,
            DataSource::CurrentInput(is_current) => is_current as u16,
            DataSource::PCIsActive(pc) => (ctx.party.characters.get(&pc).unwrap().party_state == CharacterPartyState::Available) as u16,
            DataSource::PCIsActiveOrReserve(pc) => (ctx.party.characters.get(&pc).unwrap().party_state != CharacterPartyState::Unavailable) as u16,

            DataSource::WorldActor(_actor_address) => 0,
        }
    }

    pub fn as_string(&self) -> String {
        match self {
            DataSource::Immediate(value) => format!("{}", value),
            DataSource::Memory(address) => address_to_variable(*address),
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
            DataDest::Memory(address) => ctx.memory.put_u8(*address, value),
            _ => panic!("Unhandled put_u8"),
        }
    }

    pub fn put_scene_u8(&self, ctx: &mut Context, _scene_state: &mut SceneState, value: u8) {
        match self {
            _ => self.put_u8(ctx, value),
        }
    }

    pub fn put_world_u8(&self, ctx: &mut Context, world_state: &mut WorldState, actor: &mut WorldActor, value: u8) {
        match self {
            DataDest::WorldActor(address) => {
                if *address == 0x0F {
                    actor.palette_priority = value;
                } else if *address == 0x10 {
                    actor.sprite_tile_offset = value as i32;
                } else if *address == 0x14 {
                    // actor pixel x 3rd byte
                    actor.pos.x += 1.0;
                } else {
                    actor.memory.put_u8(*address, value);
                }
            }
            DataDest::Memory(address) => {

                // Layer 1 X and Y.
                if *address == 0x7E00E3 {
                    world_state.camera.set_to(Vec2Df64::new(value as f64 * 8.0, world_state.camera.pos.y));
                } else if *address == 0x7E00E7 {
                    world_state.camera.set_to(Vec2Df64::new(world_state.camera.pos.x, value as f64 * 8.0 + 1.0));

                // Layer 2 X and Y.
                } else if *address == 0x7E00E5 {
                    world_state.camera.set_to(Vec2Df64::new(value as f64 * 8.0, world_state.camera.pos.y));
                } else if *address == 0x7E00E9 {
                    world_state.camera.set_to(Vec2Df64::new(world_state.camera.pos.x, value as f64 * 8.0 + 1.0));
                } else {
                    self.put_u8(ctx, value);
                }
            },
        }
    }

    pub fn put_u16(&self, ctx: &mut Context, _scene_state: &mut SceneState, value: u16) {
        match self {
            DataDest::Memory(address) => ctx.memory.put_u16(*address, value),
            DataDest::WorldActor(_address) => {},
        }
    }

    pub fn put_bytes(&self, ctx: &mut Context, _scene_state: &mut SceneState, bytes: [u8; 64], length: usize) {
        match self {
            DataDest::Memory(address) => ctx.memory.put_bytes(*address, &bytes[0..length]),
            DataDest::WorldActor(_address) => {},
        }
    }

    pub fn as_string(&self) -> String {
        match self {
            DataDest::Memory(address) => address_to_variable(*address),
            DataDest::WorldActor(address) => format!("self.0x{:02X}", address),
        }
    }
}

fn address_to_variable(address: usize) -> String {
    match address {
        0x7E0000 .. 0x7E0080 => format!("Temp{:02X}", address - 0x7E0000),

        0x7E00E3 => String::from("L1ScrollXH"),
        0x7E00E4 => String::from("L1ScrollXL"),
        0x7E00E5 => String::from("L1ScrollYH"),
        0x7E00E6 => String::from("L1ScrollYL"),

        0x7E00E7 => String::from("L2ScrollXH"),
        0x7E00E8 => String::from("L2ScrollXL"),
        0x7E00E9 => String::from("L2ScrollYH"),
        0x7E00EA => String::from("L2ScrollYL"),

        0x7E00F0 => format!("Unknown{:06X}", address),
        0x7E00F1 => format!("Unknown{:06X}", address),

        0x7E0104 => String::from("DestinationFacing"),

        0x7E0294 => String::from("EpochStateFlags"),
        0x7E029E => String::from("DactylStateFlags"),
        0x7E027C => String::from("WorldMasterAction"),
        0x7E029F => String::from("EpochMapX"),
        0x7E02A0 => String::from("EpochMapY"),

        0x7E0920 => format!("WorldUnknown{:06X}", address),
        0x7E0921 => format!("WorldUnknown{:06X}", address),

        0x7E1B48 => format!("WorldUnknown{:04X}", address),
        0x7E1B9B => String::from("WorldMusic"),

        0x7E1BA3 => String::from("PlayerCharacter1"),
        0x7E1BA4 => String::from("PlayerCharacter2"),
        0x7E1BA5 => String::from("PlayerCharacter3"),

        0x7E1BA7 .. 0x7E1BBB => format!("WorldStoryFlags{:02X}", address - 0x7E1BA7),

        0x7EA214 => format!("WorldUnknown{:06X}", address),
        0x7EA315 => format!("WorldUnknown{:06X}", address),
        0x7EA418 => format!("WorldUnknown{:06X}", address),
        0x7EA519 => format!("WorldUnknown{:06X}", address),

        _ => format!("0x{:06X}", address),
    }
}

