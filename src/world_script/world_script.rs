use crate::{Context, GameMode};
use crate::gamestate::gamestate_world::WorldState;
use crate::world_script::world_script_ops::Op;
use crate::world_script::world_script_disassembler::WorldScriptDisassembler;

#[derive(Clone, Copy)]
pub struct WorldActorScriptState {

    /// The current execution address.
    pub current_address: u64,

    /// Current decoded op.
    pub current_op: Option<Op>,
}

impl WorldActorScriptState {
    pub fn dump(&self) {
        println!("World actor script state");
        println!("  Current address 0x{:04X}", self.current_address);
        println!("  Current op {:?}", self.current_op);
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
