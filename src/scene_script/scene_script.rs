use std::collections::HashMap;
use std::io::Cursor;
use crate::actor::{Actor, ActorFlags, Direction};
use crate::Context;
use crate::map::Map;
use crate::scene::scene_map::SceneMap;
use crate::scene_script::ops::Op;
use crate::scene_script::ops_char_load::CharacterType;
use crate::scene_script::scene_script_decoder::op_decode;
use crate::scene_script::scene_script_memory::SceneScriptMemory;

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
    memory: SceneScriptMemory,
    pub actor_scripts: Vec<SceneActorScript>,
    pub script_states: Vec<ActorScriptState>,
}

impl SceneScript {
    pub fn new(index: usize, data: Vec<u8>, actor_scripts: Vec<SceneActorScript>) -> SceneScript {
        SceneScript {
            index,
            data: Cursor::new(data),
            memory: SceneScriptMemory::new(),
            actor_scripts,
            script_states: Vec::new(),
        }
    }

    pub fn add_initial_state(&mut self, actor_index: usize) -> &ActorScriptState {
        let state = self.actor_scripts[actor_index].get_initial_state();
        self.script_states.push(state);
        self.script_states.get(actor_index).unwrap()
    }

    pub fn run_until_yield(&mut self, ctx: &mut Context, actors: &mut Vec<Actor>, map: &mut Map, scene_map: &mut SceneMap) {
        for (state_index, state) in self.script_states.iter_mut().enumerate() {
            self.data.set_position(state.address);
            'decoder: loop {
                let op = op_decode(&mut self.data);
                state.address = self.data.position();
                if op_execute(ctx, op, state_index, actors, map, scene_map, &mut self.memory) {
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
        println!("  Global: {:02X?}", self.memory.global);
        println!("  Local: {:02X?}", self.memory.local);
        println!("  Temp: {:02X?}", self.memory.temp);
    }
}

fn op_execute(ctx: &mut Context, op: Op, this_actor: usize, actors: &mut Vec<Actor>, _map: &mut Map, scene_map: &mut SceneMap, memory: &mut SceneScriptMemory) -> bool {
    match op {
        Op::NOP => false,
        Op::Yield { forever: _ } => true,
        Op::Return => true,

        Op::Copy8 { source, dest } => {
            dest.put_u8(memory, source.get_u8(memory));
            false
        },
        Op::Copy16 { source, dest } => {
            dest.put_u16(memory, source.get_u16(memory));
            false
        },
        Op::CopyBytes { dest, bytes, length } => {
            dest.put_bytes(memory, bytes, length);
            false
        },

        // todo must_be_in_party
        Op::LoadCharacter { char_type, index, is_static, battle_index, .. } => {
            let real_index = match char_type {
                CharacterType::PC => index,
                CharacterType::PCAsNPC => index,
                CharacterType::NPC => index + 7,
                CharacterType::Enemy => index + 256,
            };

            ctx.sprite_assets.load(&ctx.fs, real_index);

            actors[this_actor].x = 0.0;
            actors[this_actor].y = 0.0;
            actors[this_actor].battle_index = battle_index;

            actors[this_actor].flags |= ActorFlags::RENDERED | ActorFlags::VISIBLE;
            if is_static {
                actors[this_actor].flags |= ActorFlags::BATTLE_STATIC;
            }

            if char_type == CharacterType::PC {
                actors[this_actor].player_index = Some(index);
            }

            let state = &mut ctx.sprites_states.get_state_mut(this_actor);
            state.enabled = true;
            state.sprite_index = real_index;
            ctx.sprites_states.set_animation(&ctx.sprite_assets, this_actor, 0, true);

            false
        },

        Op::ActorCoordinatesSet { actor, x, y } => {
            let actor_index = actor.deref(this_actor);
            let x = x.get_u8(memory) as f64;
            let y = y.get_u8(memory) as f64;

            actors[actor_index].x = x * 16.0 + 8.0;
            actors[actor_index].y = y * 16.0 + 16.0;

            // Set sprite priority from scene map properties.
            let tile_x = (actors[actor_index].x / 16.0) as u32;
            let tile_y = (actors[actor_index].y / 16.0 - 4.0) as u32;
            let index = (tile_y * scene_map.props.width + tile_x) as usize;
            if index < scene_map.props.props.len() {
                if let Some(sprite_priority) = scene_map.props.props[index].sprite_priority {
                    actors[actor_index].sprite_priority_top = sprite_priority;
                    actors[actor_index].sprite_priority_bottom = sprite_priority;
                }
            }

            false
        },

        Op::ActorCoordinatesSetPrecise { actor, x, y } => {
            let actor_index = actor.deref(this_actor);
            let x = x.get_u16(memory) as f64;
            let y = y.get_u16(memory) as f64;

            actors[actor_index].x = x;
            actors[actor_index].y = y + 1.0;

            // Set sprite priority from scene map properties.
            let tile_x = (actors[actor_index].x / 16.0) as u32;
            let tile_y = (actors[actor_index].y / 16.0 - 4.0) as u32;
            let index = (tile_y * scene_map.props.width + tile_x) as usize;
            if index < scene_map.props.props.len() {
                if let Some(sprite_priority) = scene_map.props.props[index].sprite_priority {
                    actors[actor_index].sprite_priority_top = sprite_priority;
                    actors[actor_index].sprite_priority_bottom = sprite_priority;
                }
            }

            false
        },

        Op::ActorSetDirection { actor, direction } => {
            let actor_index = actor.deref(this_actor);
            let direction = Direction::from_index(direction.get_u8(memory) as usize);
            actors[actor_index].direction = direction;
            ctx.sprites_states.set_direction(&ctx.sprite_assets, actor_index, direction);

            false
        },

        Op::ActorSetSpriteFrame { actor, frame } => {
            let actor_index = actor.deref(this_actor);
            let frame_index = frame.get_u8(memory) as usize;
            ctx.sprites_states.set_sprite_frame(actor_index, frame_index);

            false
        },

        // todo mode, unknowns
        Op::ActorSetSpritePriority { actor, top, bottom, .. } => {
            let actor_index = actor.deref(this_actor);
            actors[actor_index].sprite_priority_top = top;
            actors[actor_index].sprite_priority_bottom = bottom;

            false
        },

        // todo loops, wait
        Op::Animate { actor, animation, run, .. } => {
            let actor_index = actor.deref(this_actor);
            let anim_index = animation.get_u8(memory) as usize;
            ctx.sprites_states.set_animation(&ctx.sprite_assets, actor_index, anim_index, run);

            false
        },

        _ => false,
    }
}
