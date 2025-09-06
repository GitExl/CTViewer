use std::collections::HashMap;
use std::io::Cursor;
use crate::actor::{Actor, ActorFlags};
use crate::Context;
use crate::map::Map;
use crate::scene::scene_map::SceneMap;
use crate::scene_script::ops::Op;
use crate::scene_script::scene_script_decoder::op_decode;
use crate::scene_script::scene_script_exec::op_execute;
use crate::scene_script::scene_script_memory::SceneScriptMemory;

/// Yield, Completed
pub type OpResult = (bool, bool);

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
            delay: 4,
            delay_counter: 0,
            priority_ptrs: [0; 8],
            current_priority: 0,
            current_op: None,
            op_yielded: false,
            op_completed: false,
        }
    }
}

pub struct ActorScriptState {

    /// Delay is how many ticks need to pass before this script state is processed again.
    /// The delay counter tracks how many such ticks are left.
    pub delay: u32,
    pub delay_counter: u32,

    /// The current address of execution.
    pub address: u64,

    /// Pointers to each script function.
    pub ptrs: [u64; 16],

    /// Pointers to script function at prioritry levels.
    pub priority_ptrs: [usize; 8],
    pub current_priority: usize,

    /// Current decoded op.
    pub current_op: Option<Op>,

    /// True if the current op yielded processing.
    pub op_yielded: bool,

    /// True if the current op completed processing and we should advance to the next op.
    pub op_completed: bool,
}

impl ActorScriptState {
    pub fn dump(&self) {
        println!("Actor script state");
        println!("  Delay {} / {}", self.delay_counter, self.delay);
        println!("  Current address 0x{:04X}", self.address);
        println!("  Priorities: {:04X?}", self.priority_ptrs);
        println!("  Current priority: {}", self.current_priority);
        println!("  Current op {:?}", self.current_op);
        println!("    Yielded: {}", self.op_yielded);
        println!("    Completed {}", self.op_completed);
        println!();
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum SceneScriptMode {
    Pc,
    Snes,
}

pub struct SceneScript {
    index: usize,
    mode: SceneScriptMode,
    data: Cursor<Vec<u8>>,
    memory: SceneScriptMemory,
    pub actor_scripts: Vec<SceneActorScript>,
    pub script_states: Vec<ActorScriptState>,
}

impl SceneScript {
    pub fn new(index: usize, data: Vec<u8>, actor_scripts: Vec<SceneActorScript>, mode: SceneScriptMode) -> SceneScript {
        let mut memory = SceneScriptMemory::new();

        // Cats!
        memory.write_u8(0x7F0053, 0xFF);
        memory.write_u8(0x7F005F, 0xFF);

        // Storyline.
        memory.write_u8(0x7F0000, 0x00);

        SceneScript {
            index,
            mode,
            data: Cursor::new(data),
            memory,
            actor_scripts,
            script_states: Vec::new(),
        }
    }

    pub fn add_initial_state(&mut self, actor_index: usize) -> &ActorScriptState {
        let state = self.actor_scripts[actor_index].get_initial_state();
        self.script_states.push(state);
        self.script_states.get(actor_index).unwrap()
    }

    pub fn run_until_return(&mut self, ctx: &mut Context, actors: &mut Vec<Actor>, map: &mut Map, scene_map: &mut SceneMap) {
        for (state_index, state) in self.script_states.iter_mut().enumerate() {
            if actors[state_index].flags.contains(ActorFlags::SCRIPT_DISABLED) {
                continue;
            }

            loop {
                self.data.set_position(state.address);

                if state.current_op.is_none() || state.op_completed {
                    state.current_op = op_decode(&mut self.data, self.mode);
                    state.address = self.data.position();
                }

                (state.op_yielded, state.op_completed) = op_execute(ctx, state, state_index, actors, map, scene_map, &mut self.memory);
                self.data.set_position(state.address);

                if let Some(op) = state.current_op {
                    if op == Op::Return {
                        state.op_completed = true;
                        break;
                    }
                }
            }
        }
    }

    pub fn run(&mut self, ctx: &mut Context, actors: &mut Vec<Actor>, map: &mut Map, scene_map: &mut SceneMap) {
        for (state_index, state) in self.script_states.iter_mut().enumerate() {
            if actors[state_index].flags.contains(ActorFlags::SCRIPT_DISABLED) {
                continue;
            }

            // Countdown until next time this actor's script needs to be processed.
            if state.delay_counter > 0 {
                state.delay_counter -= 1;
                continue;
            }
            state.delay_counter = state.delay;

            // Execute up to 5 instructions, unless one yields.
            for _ in 0..5 {
                self.data.set_position(state.address);

                // Advance to the next op.
                if state.current_op.is_none() || state.op_completed {
                    state.current_op = op_decode(&mut self.data, self.mode);
                    state.address = self.data.position();
                }

                (state.op_yielded, state.op_completed) = op_execute(ctx, state, state_index, actors, map, scene_map, &mut self.memory);
                self.data.set_position(state.address);

                if state.op_yielded {
                    break;
                }
            }
        }
    }

    pub fn decode(&self) {
        let mut labels: HashMap<u64, String> = HashMap::new();

        for (actor_index, actor_script) in self.actor_scripts.iter().enumerate() {
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

            let op = op_decode(&mut data, self.mode);
            match op {
                Some(op) => println!("    0x{:04X} {:?}", address, op),
                None => println!("    0x{:04X} ???", address),
            };

            address = data.position();
        }
    }

    pub fn dump(&self) {
        println!("Scene script {}", self.index);
        self.decode();
        println!();
        for state in self.script_states.iter() {
            state.dump();
        }
        println!("  Global: {:02X?}", self.memory.global);
        println!("  Local: {:02X?}", self.memory.local);
        println!("  Temp: {:02X?}", self.memory.temp);
    }
}
