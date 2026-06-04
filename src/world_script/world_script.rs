use std::io::Cursor;
use crate::{Context, GameMode};
use crate::gamestate::gamestate_world::WorldState;
use crate::memory::MemoryRegion;
use crate::shared_op::{BitMathOp, ByteMathOp, CompareOp};
use crate::world_script::exec::tile_copy::exec_tile_copy;
use crate::world_script::world_script_decoder::op_decode;
use crate::world_script::world_script_disassembler::WorldScriptDisassembler;
use crate::world_script::world_script_ops::Op;

const ACTION_FUNC_RUN_SCRIPT: u64 = 0x0F63;

const ACTION_FUNC_FADE_IN: u64 = 0x2105;
const ACTION_FUNC_FADE_OUT: u64 = 0x20A2;

const ACTION_FUNC_PALETTE_LOAD: u64 = 0x1DD4;
const ACTION_FUNC_PALETTE_LOAD_MODES: u64 = 0x1E38;

const ACTION_FUNC_SCROLL_LAYERS_WORLD0: u64 = 0x75C3;
const ACTION_FUNC_SCROLL_LAYERS_WORLD1: u64 = 0x75FD;
const ACTION_FUNC_SCROLL_LAYERS_WORLD2: u64 = 0x7702;
const ACTION_FUNC_SCROLL_LAYERS_WORLD4: u64 = 0x77F2;
const ACTION_FUNC_SCROLL_LAYERS_WORLD5: u64 = 0x7849;

enum OpResult {
    Yield,
    Continue,
    ContinueFrom {
        address: u64
    },
}

#[derive(Clone)]
pub struct WorldActorState {
    pub action_function: u64,
    pub cycles: u16,
    pub return_address: u64,
    pub current_address: u64,
    pub counter: u8,
    pub animation_address: u64,
    pub animation_counter: u8,
    pub palette_priority: u8,
    pub memory: MemoryRegion,
    pub x: f64,
    pub y: f64,
    pub vector_x: f64,
    pub vector_y: f64,
}

impl WorldActorState {
    pub fn dump(&self) {
        println!("World actor script state");
        println!();
    }

    pub fn new() -> Self {
        Self {
            memory: MemoryRegion::new(64),

            action_function: 0,
            cycles: 0,
            counter: 0,
            current_address: 0,
            return_address: 0,
            animation_counter: 0,
            animation_address: 0,
            palette_priority: 0,

            x: 0.0,
            y: 0.0,
            vector_x: 0.0,
            vector_y: 0.0,
        }
    }
}

pub struct WorldScript {
    index: usize,
    mode: GameMode,
    data: Cursor<Vec<u8>>,
}

impl WorldScript {
    pub fn new(index: usize, data: Vec<u8>, mode: GameMode) -> WorldScript {
        WorldScript {
            index,
            mode,
            data: Cursor::new(data),
        }
    }

    pub fn initialize(&mut self, world_state: &mut WorldState) {

        // Root actor starts from 0.
        self.add_actor(world_state, ACTION_FUNC_RUN_SCRIPT);
    }

    pub fn run(&mut self, ctx: &mut Context, world_state: &mut WorldState) {
        for actor_index in 0..world_state.actors.len() {
            let state = &world_state.actors[actor_index];
            if state.action_function == 0 {
                continue;
            }
            let cycles = state.cycles.wrapping_add(1);

            if state.action_function == ACTION_FUNC_RUN_SCRIPT {
                self.run_script(ctx, actor_index, world_state);

            } else if state.action_function == ACTION_FUNC_PALETTE_LOAD {
                self.run_palette_load(actor_index, world_state);
            } else if state.action_function == ACTION_FUNC_PALETTE_LOAD_MODES {
                self.run_palette_load_mode(actor_index, world_state);

            } else if state.action_function == ACTION_FUNC_SCROLL_LAYERS_WORLD0 {
                self.run_layer_animation(actor_index, world_state, 0);
            } else if state.action_function == ACTION_FUNC_SCROLL_LAYERS_WORLD1 {
                self.run_layer_animation(actor_index, world_state, 1);
            } else if state.action_function == ACTION_FUNC_SCROLL_LAYERS_WORLD2 {
                self.run_layer_animation(actor_index, world_state, 2);
            } else if state.action_function == ACTION_FUNC_SCROLL_LAYERS_WORLD4 {
                self.run_layer_animation(actor_index, world_state, 4);
            } else if state.action_function == ACTION_FUNC_SCROLL_LAYERS_WORLD5 {
                self.run_layer_animation(actor_index, world_state, 5);

            // } else {
            //     println!("Unknown action function 0x{:04X}", state.action_function);
            }

            world_state.actors.get_mut(actor_index).unwrap().cycles = cycles;
        }
        // println!("-------------");
    }

    fn run_script(&mut self, ctx: &mut Context, actor_index: usize, world_state: &mut WorldState) {
        let mut state = world_state.actors[actor_index].clone();

        loop {
            self.data.set_position(state.current_address);

            let result;
            if let Some(op) = op_decode(&mut self.data, self.mode) {
                // println!("{:02} {:04X} {:?}", actor_index, state.current_address, op);
                result = match op {
                    Op::InitMemory => {
                        OpResult::Continue
                    }
                    Op::Copy8 { lhs, rhs } => {
                        let value = rhs.get_world_u8(ctx, world_state, &mut state);
                        lhs.put_world_u8(ctx, world_state, &mut state, value);
                        OpResult::Continue
                    }
                    Op::GoSub { address } => {
                        state.return_address = self.data.position();
                        OpResult::ContinueFrom { address }
                    }
                    Op::Return => {
                        OpResult::ContinueFrom { address: state.return_address }
                    },
                    Op::GoTo { address } => {
                        OpResult::ContinueFrom { address }
                    }
                    Op::DecrementAndJumpIfNonZero { src, dest, offset } => {
                        let mut value = src.get_world_u8(ctx, world_state, &mut state);
                        value = value.wrapping_sub(1);
                        dest.put_world_u8(ctx, world_state, &mut state, value);
                        if value != 0 {
                            OpResult::ContinueFrom { address: (state.current_address as i64 + offset) as u64 }
                        } else {
                            OpResult::Continue
                        }
                    }
                    Op::JumpConditional { lhs, cmp, rhs, offset } => {
                        let lhs_value = lhs.get_world_u8(ctx, world_state, &mut state);
                        let rhs_value = rhs.get_world_u8(ctx, world_state, &mut state);
                        let result = match cmp {
                            CompareOp::Eq => lhs_value == rhs_value,
                            CompareOp::NotEq => lhs_value != rhs_value,
                            CompareOp::Gt => lhs_value > rhs_value,
                            CompareOp::GtEq => lhs_value >= rhs_value,
                            CompareOp::Lt => lhs_value < rhs_value,
                            CompareOp::LtEq => lhs_value <= rhs_value,
                            CompareOp::And => (lhs_value & rhs_value) > 0,
                            CompareOp::Or => (lhs_value | rhs_value) > 0,
                        };
                        if result {
                            OpResult::ContinueFrom {
                                address: (state.current_address as i64 + offset) as u64,
                            }
                        } else {
                            OpResult::Continue
                        }
                    }
                    Op::AddActor { address, .. } => {
                        let index = self.add_actor(world_state, ACTION_FUNC_RUN_SCRIPT);
                        let new_state = world_state.actors.get_mut(index).unwrap();
                        new_state.current_address = address;
                        OpResult::Continue
                    }
                    Op::AddActorSpecial { address, .. } => {
                        let index = self.add_special_actor(world_state, ACTION_FUNC_RUN_SCRIPT);
                        let new_state = world_state.actors.get_mut(index).unwrap();
                        new_state.current_address = address;
                        OpResult::Continue
                    }
                    Op::Link { address } => {
                        self.add_actor(world_state, address);
                        OpResult::Continue
                    }
                    Op::LinkSpecial { address } => {
                        self.add_special_actor(world_state, address);
                        OpResult::Continue
                    }
                    Op::FadeIn { mode } => {
                        self.add_special_actor(world_state, ACTION_FUNC_FADE_IN);
                        OpResult::Continue
                    }
                    Op::FadeOut { mode } => {
                        self.add_special_actor(world_state, ACTION_FUNC_FADE_OUT);
                        OpResult::Continue
                    }
                    Op::Wait { steps } => {
                        if state.counter != 0 {
                            state.counter -= 1;
                        } else {
                            state.counter = steps;
                        }
                        if state.counter != 0 {
                            OpResult::Yield
                        } else {
                            OpResult::Continue
                        }
                    }
                    Op::End => {
                        state.action_function = 0;
                        OpResult::Yield
                    }
                    Op::CopyTiles { source_layer, source_x, source_y, dest_layer, dest_x, dest_y, width, height } => {
                        exec_tile_copy(&mut world_state.map, &mut world_state.world_map, &world_state.tileset_l12, source_layer, source_x, source_y, dest_layer, dest_x, dest_y, width, height);
                        OpResult::Continue
                    }
                    Op::SetTile { layer, x, y, tile_index } => {
                        let layer = &mut world_state.map.layers[layer];
                        let index = x + y * layer.tile_width as usize;
                        layer.tiles[index] = tile_index;
                        layer.assemble_chips(&world_state.tileset_l12, x as u32, y as u32, 1, 1);
                        OpResult::Continue
                    }
                    Op::BitMath { dest, lhs, op, rhs } => {
                        let lhs_value = lhs.get_world_u8(ctx, world_state, &mut state);
                        let rhs_value = rhs.get_world_u8(ctx, world_state, &mut state);
                        let result = match op {
                            BitMathOp::And => lhs_value & rhs_value,
                            BitMathOp::Or => lhs_value | rhs_value,
                            BitMathOp::Xor => lhs_value ^ rhs_value,
                            BitMathOp::ShiftLeft => lhs_value << rhs_value,
                            BitMathOp::ShiftRight => lhs_value >> rhs_value,
                        };
                        dest.put_world_u8(ctx, world_state, &mut state, result);

                        OpResult::Continue
                    }
                    Op::ByteMath { dest, lhs, op ,rhs } => {
                        let lhs_value = lhs.get_world_u8(ctx, world_state, &mut state);
                        let rhs_value = rhs.get_world_u8(ctx, world_state, &mut state);

                        let result = match op {
                            ByteMathOp::Add => lhs_value.overflowing_add(rhs_value).0,
                            ByteMathOp::Subtract => lhs_value.overflowing_sub(rhs_value).0,
                        };
                        dest.put_world_u8(ctx, world_state, &mut state, result);

                        OpResult::Continue
                    }
                    Op::SetPosition { x, y } => {
                        state.x = x as f64;
                        state.y = y as f64;
                        OpResult::Continue
                    }
                    Op::SetPriority { priority } => {
                        state.palette_priority |= priority & 0x4F;
                        OpResult::Continue
                    }
                    Op::SetPalette { index } => {
                        state.palette_priority |= index & 0xF1;
                        OpResult::Continue
                    }
                    Op::SetAnimation { anim_index } => {
                        OpResult::Continue
                    }
                    Op::WaitThenAnimate { delay } => {
                        OpResult::Yield
                    }
                    Op::VectorX { magnitude } => {
                        state.vector_x = magnitude as f64 / 65536.0;
                        OpResult::Continue
                    }
                    Op::VectorY { magnitude } => {
                        state.vector_y = magnitude as f64 / 65536.0;
                        OpResult::Continue
                    }
                    Op::Scroll { steps } => {
                        OpResult::Yield
                    }
                    Op::ChangeLocation { destination } => {
                        OpResult::Continue
                    }
                    Op::Move { steps } => {
                        if state.counter != 0 {
                            state.counter -= 1;
                        } else {
                            state.counter = steps;
                        }
                        if state.counter != 0 {

                            // Move actor by vector.
                            state.x += state.vector_x;
                            state.y += state.vector_y;

                            // Clamp to map.
                            state.x = state.x.min(world_state.world_map.pixel_width as f64).max(0.0);
                            state.y = state.y.min(world_state.world_map.pixel_height as f64).max(0.0);

                            // TODO: fn_overworld_object_animate

                            OpResult::Yield
                        } else {
                            OpResult::Continue
                        }
                    }
                    Op::PaletteLoad { address, palette_index, mode } => {
                        let new_index = self.add_actor(world_state, ACTION_FUNC_PALETTE_LOAD);
                        let new_state = world_state.actors.get_mut(new_index).unwrap();
                        new_state.memory.put_u8(0x32, palette_index);
                        new_state.memory.put_u8(0x33, 0);
                        new_state.memory.put_u8(0x34, mode);
                        new_state.memory.put_u24(0x35, address as u32);

                        OpResult::Continue
                    },

                    // For unimplemented ops.
                    _ => {
                        OpResult::Continue
                    }
                };
            } else {
                result = OpResult::Continue;
            }

            match result {
                OpResult::Continue => {
                    state.current_address = self.data.position();
                }
                OpResult::ContinueFrom { address }=> {
                    state.current_address = address;
                }
                OpResult::Yield => {
                    break;
                }
            }
        }

        world_state.actors[actor_index] = state;
    }

    fn run_palette_load(&mut self, actor_index: usize, world_state: &mut WorldState) {
        let state = world_state.actors.get_mut(actor_index).unwrap();
        state.action_function = 0;

        let palette_index = state.memory.get_u8(0x32);
        let mode = state.memory.get_u8(0x34);
        let address = state.memory.get_u24(0x35) as usize;

        // Mode 0 copies palette data from palette animations.
        if mode == 0 {
            if address < 0x7EC000 {
                println!("Invalid palette copy mode 0 address 0x{:06X}", address);
            } else {
                let src_start = ((address - 0x7EC000) / 32) * 16;
                let dest_start = palette_index as usize * 16;
                world_state.palette.palette.colors[dest_start..dest_start + 16].copy_from_slice(&world_state.palette_animation.palette.colors[src_start..src_start + 16]);
            }

        // Other modes.
        } else {
            self.add_actor(world_state, ACTION_FUNC_PALETTE_LOAD_MODES);
        }
    }

    fn run_layer_animation(&mut self, actor_index: usize, world_state: &mut WorldState, world_index: usize) {
        match world_index {
            0 => {
                world_state.map.layers[2].scroll.x -= 0.25;
                world_state.map.layers[2].scroll.y += 0.125;
            }
            1 => {
                world_state.map.layers[2].scroll.x += 0.25;
                world_state.map.layers[2].scroll.y -= 0.25;
            }
            2 => {
                world_state.map.layers[2].scroll.x -= 30414.00006103516;
                world_state.map.layers[2].scroll.y += 2.0;
            }
            4 => {
                world_state.map.layers[2].scroll.x -= 21003.0002746582;
                world_state.map.layers[2].scroll.y += 10.0;
            }
            5 => {
                world_state.map.layers[0].scroll.x -= 0.25;
            }
            _ => {},
        }
    }

    fn run_palette_load_mode(&mut self, actor_index: usize, world_state: &mut WorldState) {
        let state = world_state.actors.get_mut(actor_index).unwrap();
        state.action_function = 0;
        // TODO: full palette_load behaviour is unknown
    }

    pub fn add_special_actor(&mut self, world_state: &mut WorldState, action_func: u64) -> usize {
        for index in 0..4 {
            let state = &world_state.actors[index];
            if state.action_function == 0 {
                let mut new_state = WorldActorState::new();
                new_state.action_function = action_func;
                world_state.actors[index] = new_state;
                return index;
            }
        }

        panic!("Out of world special actor slots!");
    }

    pub fn add_actor(&mut self, world_state: &mut WorldState, action_func: u64) -> usize {
        for index in 4..world_state.actors.len() {
            let state = &world_state.actors[index];
            if state.action_function == 0 {
                let mut new_state = WorldActorState::new();
                new_state.action_function = action_func;
                world_state.actors[index] = new_state;
                return index;
            }
        }

        panic!("Out of world actor slots!");
    }

    pub fn disassemble(&self) {
        let mut disassembler = WorldScriptDisassembler::new(&self.data.get_ref(), self.mode);
        disassembler.disassemble();
        disassembler.dump();
    }

}
