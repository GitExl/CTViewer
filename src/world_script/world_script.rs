use crate::{Context, GameMode};
use crate::gamestate::gamestate_world::WorldState;
use crate::world_script::world_script_disassembler::WorldScriptDisassembler;

#[derive(Clone, Copy)]
pub struct WorldActorState {
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

impl WorldActorState {
    pub fn dump(&self) {
        println!("World actor script state");
        println!();
    }

    pub fn default() -> Self {
        Self {
            counter: 0,
            timer: 0,
            current_address: 0,
            memory: [0; 48],
            return_address: 0,
            unknown: 0,
            animation_counter: 0,
            palette_priority: 0,
            action_function: 0,
            animation_address: 0,
        }
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

    pub fn initialize(&mut self, world_state: &mut WorldState) {

        // Root actor starts from 0.
        self.add_actor(world_state, 0x0F63);
    }

    pub fn run(&self, _ctx: &mut Context, _world_state: &mut WorldState) {

    }

    pub fn add_special_actor(&mut self, world_state: &mut WorldState, action_func: u16) -> usize {
        for index in 0..4 {
            let actor = world_state.actors.get_mut(index).unwrap();
            if actor.action_function == 0 {
                actor.action_function = action_func;
                return index;
            }
        }

        panic!("Out of world special actor slots!");
    }

    pub fn add_actor(&mut self, world_state: &mut WorldState, action_func: u16) -> usize {
        for index in 4..world_state.actors.len() {
            let actor = world_state.actors.get_mut(index).unwrap();
            if actor.action_function == 0 {
                actor.action_function = action_func;
                return index;
            }
        }

        panic!("Out of world actor slots!");
    }

    pub fn disassemble(&self) {
        let mut disassembler = WorldScriptDisassembler::new(&self.data, self.mode);
        disassembler.disassemble();
        disassembler.dump();
    }
}
