use std::collections::HashMap;
use std::io::Cursor;
use crate::actor::{Actor, ActorFlags};
use crate::map_renderer::MapSprite;
use crate::scene_script::ops::Op;
use crate::scene_script::ops_char_load::CharacterType;
use crate::scene_script::scene_script_decoder::op_decode;
use crate::sprites::sprite_manager::{SpriteManager, SpriteState};

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

pub struct SceneScript {
    index: usize,
    data: Cursor<Vec<u8>>,
    pub actor_scripts: Vec<SceneActorScript>,
    pub script_states: Vec<ActorScriptState>,
}

impl SceneScript {
    pub fn new(index: usize, data: Vec<u8>, actor_scripts: Vec<SceneActorScript>) -> SceneScript {
        SceneScript {
            index,
            data: Cursor::new(data),
            actor_scripts,
            script_states: Vec::new(),
        }
    }

    pub fn create_state(&mut self, index: usize) -> usize {
        let state = self.actor_scripts[index].get_initial_state();
        self.script_states.push(state);
        self.script_states.len() - 1
    }

    pub fn run_until_yield(&mut self, actors: &mut Vec<Actor>, sprites: &mut SpriteManager, map_sprites: &mut Vec<MapSprite>) {
        for (state_index, state) in self.script_states.iter_mut().enumerate() {
            self.data.set_position(state.address);
            'decoder: loop {
                let op = op_decode(&mut self.data);
                state.address = self.data.position();
                if op_execute(op, state_index, sprites, map_sprites, actors) {
                    break 'decoder;
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

fn op_execute(op: Op, actor_index: usize, sprites: &mut SpriteManager, map_sprites: &mut Vec<MapSprite>, actors: &mut Vec<Actor>) -> bool {
    match op {
        Op::NOP => false,
        Op::Yield { forever: _ } => true,
        Op::Return => true,

        Op::LoadCharacter { char_type, index, .. } => {
            sprites.load(index);

            map_sprites.push(MapSprite::new());

            let mut state = SpriteState::new(map_sprites.len() - 1);
            state.enabled = true;
            state.priority = 3;
            state.sprite_index = index;
            actors[actor_index].sprite_state = Some(state);
            actors[actor_index].x = 0.0;
            actors[actor_index].y = 0.0;
            actors[actor_index].flags |= ActorFlags::RENDERED;

            match char_type {
                CharacterType::PC => {}
                CharacterType::PCAsNPC => {}
                CharacterType::NPC => {}
                CharacterType::Enemy => {}
            }

            false
        },

        Op::ActorCoordinatesSet { actor, x, y, precise } => {
            let actor_index = actor.deref(actor_index);
            let x = x.deref() as f64;
            let y = y.deref() as f64;

            if precise {
                actors[actor_index].x = x;
                actors[actor_index].y = y;
            } else {
                actors[actor_index].x = x * 16.0;
                actors[actor_index].y = y * 16.0;
            }

            false
        },

        _ => false,
    }
}
