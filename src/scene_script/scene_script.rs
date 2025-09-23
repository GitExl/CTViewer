use std::collections::HashMap;
use std::io::Cursor;
use bitflags::bitflags;
use crate::actor::{Actor, ActorFlags};
use crate::Context;
use crate::map::Map;
use crate::scene::scene_map::SceneMap;
use crate::scene_script::ops::Op;
use crate::scene_script::scene_script_decoder::op_decode;
use crate::scene_script::scene_script_exec::op_execute;
use crate::scene_script::scene_script_memory::SceneScriptMemory;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct OpResult: u32 {
        const YIELD = 0x0001;
        const COMPLETE = 0x0002;
        const JUMPED = 0x0004;
    }
}

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
            delay: 4,
            delay_counter: 4,
            pause_counter: 0,

            current_address: self.ptrs[0],
            function_ptrs: self.ptrs,
            priority_return_ptrs: [0; 8],
            current_priority: 7,

            call_waiting: false,
            current_op: None,
            op_result: OpResult::empty(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct ActorScriptState {

    /// The delay is how many ticks need to pass before this script state is processed again.
    /// The counter tracks how many such ticks are left in the current cycle.
    pub delay: u32,
    pub delay_counter: u32,

    /// Counter for pausing.
    pub pause_counter: u32,

    /// The current execution address.
    pub current_address: u64,

    /// Pointers to each script function.
    pub function_ptrs: [u64; 16],

    /// Return addresses for each priority level call.
    pub priority_return_ptrs: [u64; 8],

    /// The active priority level.
    pub current_priority: usize,

    /// True if waiting for another call to complete.
    pub call_waiting: bool,

    /// Current decoded op.
    pub current_op: Option<Op>,

    /// Result from the last op execution.
    pub op_result: OpResult,
}

impl ActorScriptState {
    pub fn dump(&self) {
        println!("Actor script state");
        println!("  Delay {} / {}", self.delay_counter, self.delay);
        println!("  Pause {}", self.pause_counter);
        println!("  Current address 0x{:04X}", self.current_address);
        println!("  Return addresses: {:04X?}", self.priority_return_ptrs);
        println!("  Current priority: {}", self.current_priority);
        println!("  Current op {:?}", self.current_op);
        println!("  Result: {:?}", self.op_result);
        println!("  Call waiting: {:?}", self.call_waiting);
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
    dialogue_strings: Vec<String>,
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
            dialogue_strings: Vec::new(),
        }
    }

    pub fn add_initial_state(&mut self, actor_index: usize) -> &ActorScriptState {
        let state = self.actor_scripts[actor_index].get_initial_state();
        self.script_states.push(state);
        self.script_states.get(actor_index).unwrap()
    }

    pub fn run_until_return(&mut self, ctx: &mut Context, actors: &mut Vec<Actor>, map: &mut Map, scene_map: &mut SceneMap) {
        for state_index in 0..self.script_states.len() {
            if actors[state_index].flags.contains(ActorFlags::SCRIPT_DISABLED) {
                continue;
            }

            let mut state_dup = self.script_states[state_index].clone();
            loop {

                // Decode op at current position.
            self.data.set_position(state_dup.current_address);
                state_dup.current_op = op_decode(&mut self.data, self.mode);

                // Execute op and handle result.
                state_dup.op_result = op_execute(ctx, state_index, &mut state_dup, &mut self.script_states, actors, map, scene_map, &mut self.memory, &mut self.dialogue_strings);
                if state_dup.op_result.contains(OpResult::JUMPED) {
                    self.data.set_position(state_dup.current_address);
                } else if state_dup.op_result.contains(OpResult::COMPLETE) {
                    state_dup.current_address = self.data.position();
                }

                if let Some(op) = state_dup.current_op {
                    if op == Op::Return {
                        state_dup.op_result |= OpResult::COMPLETE;
                        break;
                    }
                }
            }
            self.script_states[state_index] = state_dup;
        }
    }

    pub fn run(&mut self, ctx: &mut Context, actors: &mut Vec<Actor>, map: &mut Map, scene_map: &mut SceneMap) {
        for state_index in 0..self.script_states.len() {
            if actors[state_index].flags.contains(ActorFlags::SCRIPT_DISABLED) {
                continue;
            }

            let mut state_dup = self.script_states[state_index].clone();

            // Countdown until next time this actor's script needs to be processed.
            if state_dup.delay_counter > 1 {
                state_dup.delay_counter -= 1;
            } else {
                state_dup.delay_counter = state_dup.delay;

                // Execute up to 5 instructions, unless one yields.
                for _ in 0..5 {

                    // Decode op at current position.
                    self.data.set_position(state_dup.current_address);
                    state_dup.current_op = op_decode(&mut self.data, self.mode);

                    // After reaching the end of data, reset the object to the init function.
                    if state_dup.current_op.is_none() {
                        state_dup.priority_return_ptrs = [0; 8];
                        state_dup.current_priority = 7;
                        state_dup.current_address = state_dup.function_ptrs[0];
                    }

                    // Execute op and handle result.
                    state_dup.op_result = op_execute(ctx, state_index, &mut state_dup, &mut self.script_states, actors, map, scene_map, &mut self.memory, &mut self.dialogue_strings);
                    if state_dup.op_result.contains(OpResult::JUMPED) {
                        self.data.set_position(state_dup.current_address);
                    } else if state_dup.op_result.contains(OpResult::COMPLETE) {
                        state_dup.current_address = self.data.position();
                    }
                    if state_dup.op_result.contains(OpResult::YIELD) {
                        break;
                    }
                }
            }

            self.script_states[state_index] = state_dup;
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
