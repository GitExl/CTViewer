use std::io::Cursor;
use crate::scene::scene_script_decoder::{op_decode, Op};

pub struct SceneActorScript {
    ptrs: [u64; 16],
}

impl SceneActorScript {
    pub fn new(ptrs: [u64; 16]) -> SceneActorScript {
        SceneActorScript {
            ptrs,
        }
    }

    pub fn get_initial_state(&self) -> ActorScriptState {
        ActorScriptState {
            ptrs: self.ptrs,
            address: self.ptrs[0],
            ops_per_tick: 1,
            priority_address: [0; 8],
            stored_address: 0,
        }
    }
}

pub struct SceneScript {
    index: usize,
    data: Cursor<Vec<u8>>,
    pub actors: Vec<SceneActorScript>,
}

impl SceneScript {
    pub fn new(index: usize, data: Vec<u8>, actors: Vec<SceneActorScript>) -> SceneScript {
        SceneScript {
            index,
            data: Cursor::new(data),
            actors,
        }
    }

    pub fn run_until_yield(&mut self, state: &mut ActorScriptState) {
        self.data.set_position(state.address);

        println!("Run until yield from 0x{:X}", state.address);

        'decoder: loop {
            let op = op_decode(&mut self.data);
            println!("  0x{:04X} {:?}", state.address, op);
            state.address = self.data.position();
            if op_execute(op) {
                break 'decoder;
            }
        }
    }

    pub fn dump(&self) {
        println!("Scene script {}", self.index);
        for (index, _) in self.actors.iter().enumerate() {
            println!("  Actor script {}", index);
        }
        println!();
    }
}

pub struct ActorScriptState {
    pub ops_per_tick: u32,
    pub address: u64,
    pub stored_address: usize,
    pub ptrs: [u64; 16],
    pub priority_address: [usize; 8],
}

impl ActorScriptState {
    pub fn new() -> ActorScriptState {
        ActorScriptState {
            ops_per_tick: 0,
            address: 0,
            stored_address: 0,
            ptrs: [0; 16],
            priority_address: [0; 8],
        }
    }
}

fn op_execute(op: Op) -> bool {
    match op {
        Op::NOP => false,
        Op::Yield => true,
        _ => {
            println!("Cannot execute unimplemented op {:?}", op);
            true
        },
    }
}
