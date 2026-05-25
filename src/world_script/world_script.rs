use std::io::Cursor;
use crate::{Context, GameMode};
use crate::gamestate::gamestate_world::WorldState;
use crate::world_script::ops::Op;
use crate::world_script::world_script_decoder::op_decode;

pub struct WorldActorScript {
    ptrs: [u64; 16],
}

impl WorldActorScript {
    pub fn new(ptrs: [u64; 16]) -> WorldActorScript {
        WorldActorScript {
            ptrs,
        }
    }

    pub fn get_initial_state(&self) -> WorldActorScriptState {
        WorldActorScriptState {
            delay: 4,
            delay_counter: 4,
            pause_counter: 0,

            current_address: self.ptrs[0],

            current_op: None,
        }
    }
}

#[derive(Clone, Copy)]
pub struct WorldActorScriptState {

    /// The delay is how many ticks need to pass before this script state is processed again.
    /// The counter tracks how many such ticks are left in the current cycle.
    pub delay: u32,
    pub delay_counter: u32,

    /// Counter for pausing.
    pub pause_counter: u32,

    /// The current execution address.
    pub current_address: u64,

    /// Current decoded op.
    pub current_op: Option<Op>,
}

impl WorldActorScriptState {
    pub fn dump(&self) {
        println!("World actor script state");
        println!("  Delay {} / {}", self.delay_counter, self.delay);
        println!("  Pause {}", self.pause_counter);
        println!("  Current address 0x{:04X}", self.current_address);
        println!("  Current op {:?}", self.current_op);
        println!();
    }
}

pub struct WorldScript {
    index: usize,
    mode: GameMode,
    data: Vec<u8>,
    actor_scripts: Vec<WorldActorScript>,
}

impl WorldScript {
    pub fn new(index: usize, data: Vec<u8>, mode: GameMode) -> WorldScript {
        WorldScript {
            index,
            mode,
            data,
            actor_scripts: Vec::new(),
        }
    }

    pub fn get_actor_scripts(&self) -> &Vec<WorldActorScript> {
        &self.actor_scripts
    }

    pub fn get_data(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn run(&self, _ctx: &mut Context, _world_state: &mut WorldState) {

    }

    pub fn decode(&self) {
        let mut data = Cursor::new(self.data.clone());
        data.set_position(0);
        let data_len = data.get_ref().len() as u64;

        let mut address = 0;
        while address < data_len {
            let op = op_decode(&mut data, self.mode);
            match op {
                Some(op) => println!("    0x{:04X} {:?}", address, op),
                None => println!("    0x{:04X} ???", address),
            };

            address = data.position();
        }
    }
}
