use std::collections::HashMap;
use std::io::Cursor;
use std::iter::Map;
use crate::scene_script::ops::Op;
use crate::scene_script::scene_script_decoder::op_decode;

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
        'decoder: loop {
            let op = op_decode(&mut self.data);
            state.address = self.data.position();
            if op_execute(op) {
                break 'decoder;
            }
        }
    }

    pub fn decode(&self) {
        let mut labels: HashMap<u64, String> = HashMap::new();

        for (actor_index, actor_script) in self.actors.iter().enumerate() {
            for (ptr_index, ptr) in actor_script.ptrs.iter().enumerate() {
                if labels.contains_key(ptr) {
                    continue;
                }
                
                if ptr_index == 0 {
                    labels.insert(*ptr, format!("actor_{:02}_init", actor_index));
                } else if ptr_index == 1 {
                    labels.insert(*ptr, format!("actor_{:02}_activate", actor_index));
                } else if ptr_index == 2 {
                    labels.insert(*ptr, format!("actor_{:02}_touch", actor_index));
                } else {
                    labels.insert(*ptr, format!("actor_{:02}_func{:02}", actor_index, ptr_index));
                }
            }
        }

        let mut data = self.data.clone();
        data.set_position(0);
        let data_len = data.get_ref().len() as u64;

        let mut address = 0;
        while address < data_len {
            if labels.contains_key(&address) {
                println!();
                println!("  {}:", labels[&address]);
            }

            let op = op_decode(&mut data);
            println!("    0x{:04X} {:?}", address, op);

            address = data.position();
        }
    }

    pub fn dump(&self) {
        println!("Scene script {}", self.index);
        self.decode();
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
        Op::Yield { forever: _ } => true,
        Op::Return => true,
        _ => {
            println!("Cannot execute unimplemented op {:?}", op);
            true
        },
    }
}
