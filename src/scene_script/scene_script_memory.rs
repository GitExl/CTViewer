use crate::actor::ActorFlags;
use crate::scene_script::scene_script_decoder::{ActorRef, InputBinding};

pub struct SceneScriptMemory {
    pub temp: [u8; 0x200],
    pub global: [u8; 0x200],
    pub local: [u8; 0x200],
}

impl SceneScriptMemory {
    pub fn new() -> SceneScriptMemory {
        SceneScriptMemory {
            temp: [0; 0x200],
            global: [0; 0x200],
            local: [0; 0x200],
        }
    }

    pub fn write_u8(&mut self, address: usize, value: u8) {
        if address >= 0x7E0000 && address < 0x7E0200 {
            self.temp[address - 0x7E0000] = value;
        } else if address >= 0x7F0000 && address < 0x7F0200 {
            self.global[address - 0x7F0000] = value;
        } else if address >= 0x7F0200 && address < 0x7F0400 {
            self.local[address - 0x7F0200] = value;
        } else {
            println!("Unhandled scene script u8 memory write of 0x{:02X} to 0x{:06X}.", value, address)
        }
    }

    pub fn read_u8(&self, address: usize) -> u8 {
        if address >= 0x7E0000 && address < 0x7E0200 {
            return self.temp[address - 0x7E0000];
        } else if address >= 0x7F0000 && address < 0x7F0200 {
            return self.global[address - 0x7F0000];
        } else if address >= 0x7F0200 && address < 0x7F0400 {
            return self.local[address - 0x7F0200];
        }

        println!("Unhandled scene script u8 memory read at 0x{:06X}.", address);
        0
    }

    pub fn write_u16(&mut self, address: usize, value: u16) {
        if address >= 0x7E0000 && address < 0x7E0200 {
            self.temp[address - 0x7E0000 + 0] = (value >> 8) as u8;
            self.temp[address - 0x7E0000 + 1] = value as u8;
        } else if address >= 0x7F0000 && address < 0x7F0200 {
            self.global[address - 0x7F0000 + 0] = (value >> 8) as u8;
            self.global[address - 0x7F0000 + 1] = value as u8;
        } else if address >= 0x7F0200 && address < 0x7F0400 {
            self.local[address - 0x7F0200 + 0] = (value >> 8) as u8;
            self.local[address - 0x7F0200 + 1] = value as u8;
        } else {
            println!("Unhandled scene script u16 memory write of 0x{:04X} to 0x{:06X}.", value, address)
        }
    }

    pub fn read_u16(&self, address: usize) -> u16 {
        if address >= 0x7E0000 && address < 0x7E0200 {
            return self.temp[address - 0x7E0000 + 1] as u16 | self.temp[address - 0x7E0000] as u16 >> 8;
        } else if address >= 0x7F0000 && address < 0x7F0200 {
            return self.global[address - 0x7F0000 + 1] as u16 | self.global[address - 0x7F0000] as u16 >> 8;
        } else if address >= 0x7F0200 && address < 0x7F00400 {
            return self.local[address - 0x7F0200 + 1] as u16 | self.local[address - 0x7F0200] as u16 >> 8;
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
    Immediate(u32),

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
    PCIsRecruited,
    PCIsActive,
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

    pub fn get_u8(self, memory: &SceneScriptMemory) -> u8 {
        match self {
            DataSource::Immediate(value) => value as u8,
            DataSource::Memory(address) => memory.read_u8(address),
            DataSource::ActorResult(..) => 0,
            DataSource::GoldCount => 0,
            DataSource::ItemCount(..) => 0,
            DataSource::ActorFlag(..) => 0,
            DataSource::PartyCharacter(..) => 0,
            DataSource::Input(..) => 0,
            DataSource::CurrentInput(is_current) => is_current as u8,
            DataSource::PCIsActive => 0,
            DataSource::PCIsRecruited => 0,
        }
    }

    pub fn get_u16(self, memory: &SceneScriptMemory) -> u16 {
        match self {
            DataSource::Immediate(value) => value as u16,
            DataSource::Memory(address) => memory.read_u16(address),
            DataSource::ActorResult(..) => 0,
            DataSource::GoldCount => 0,
            DataSource::ItemCount(..) => 0,
            DataSource::ActorFlag(..) => 0,
            DataSource::PartyCharacter(..) => 0,
            DataSource::Input(..) => 0,
            DataSource::CurrentInput(is_current) => is_current as u16,
            DataSource::PCIsActive => 0,
            DataSource::PCIsRecruited => 0,
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

    pub fn put_u8(&self, memory: &mut SceneScriptMemory, value: u8) {
        match self {
            DataDest::Memory(address) => memory.write_u8(*address, value),
        }
    }

    pub fn put_u16(&self, memory: &mut SceneScriptMemory, value: u16) {
        match self {
            DataDest::Memory(address) => memory.write_u16(*address, value),
        }
    }

    pub fn put_bytes(&self, memory: &mut SceneScriptMemory, bytes: [u8; 32], length: usize) {
        match self {
            DataDest::Memory(address) => memory.write_bytes(*address, &bytes[0..length]),
        }
    }
}
