use crate::{Context, GameMode};
use crate::gamestate::gamestate_world::WorldState;
use crate::world_script::world_script_disassembler::WorldScriptDisassembler;

pub struct WorldActorScriptState {
    pub action_function: u16,
    pub unknown: u8,
    pub timer: u8,
    pub return_address: u64,
    pub current_address: u64,
    pub counter: u8,
    pub animation_address: u64,
    pub animation_counter: u8,
    pub palette_priority: u8,
    pub memory: [u8; 48],
}

impl WorldActorScriptState {
    pub fn dump(&self) {
        println!("World actor script state");
        println!();
    }
}

pub struct WorldScript {
    index: usize,
    mode: GameMode,
    data: Vec<u8>,
}

impl WorldScript {
    pub fn new(index: usize, data: Vec<u8>, mode: GameMode) -> WorldScript {
        WorldScript {
            index,
            mode,
            data,
        }
    }

    pub fn get_data(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn run(&self, _ctx: &mut Context, _world_state: &mut WorldState) {

    }

    pub fn disassemble(&self) {
        let mut disassembler = WorldScriptDisassembler::new(&self.data, self.mode);
        disassembler.disassemble();
        disassembler.dump();
    }
}
