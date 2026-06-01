use std::io::Cursor;
use crate::{Context, GameMode};
use crate::gamestate::gamestate_world::WorldState;
use crate::shared_op::{BitMathOp, ByteMathOp, CompareOp};
use crate::world_script::world_script_decoder::op_decode;
use crate::world_script::world_script_disassembler::WorldScriptDisassembler;
use crate::world_script::world_script_ops::Op;

const ACTION_FUNC_RUN_SCRIPT: u64 = 0x0F63;
const ACTION_FUNC_FADE_IN: u64 = 0x2105;
const ACTION_FUNC_FADE_OUT: u64 = 0x20A2;

enum OpResult {
    Yield,
    Continue,
    ContinueFrom {
        address: u64
    },
}

#[derive(Clone, Copy)]
pub struct WorldActorState {
    pub action_function: u64,
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
            let state = world_state.actors[actor_index];
            if state.action_function == ACTION_FUNC_RUN_SCRIPT {
                self.run_script(ctx, actor_index, world_state);
            }
        }
    }

    fn run_script(&mut self, ctx: &mut Context, actor_index: usize, world_state: &mut WorldState) {
        let mut state = world_state.actors[actor_index].clone();

        loop {
            self.data.set_position(state.current_address);

            let result;
            if let Some(op) = op_decode(&mut self.data, self.mode) {
                result = match op {
                    Op::InitMemory => {
                        OpResult::Continue
                    }
                    Op::Copy8 { lhs, rhs } => {
                        lhs.put_world_u8(ctx, world_state, rhs.get_world_u8(ctx, world_state, actor_index));
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
                    Op::JumpConditional { lhs, cmp, rhs, offset } => {
                        let lhs_value = lhs.get_world_u8(ctx, world_state, actor_index);
                        let rhs_value = rhs.get_world_u8(ctx, world_state, actor_index);
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
                        if !result {
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
                        OpResult::Continue
                    }
                    Op::SetTile { layer, x, y, tile_index } => {
                        OpResult::Continue
                    }
                    Op::BitMath { dest, lhs, op, rhs } => {
                        let lhs_value = lhs.get_world_u8(ctx, world_state, actor_index);
                        let rhs_value = rhs.get_world_u8(ctx, world_state, actor_index);
                        let result = match op {
                            BitMathOp::And => lhs_value & rhs_value,
                            BitMathOp::Or => lhs_value | rhs_value,
                            BitMathOp::Xor => lhs_value ^ rhs_value,
                            BitMathOp::ShiftLeft => lhs_value << rhs_value,
                            BitMathOp::ShiftRight => lhs_value >> rhs_value,
                        };
                        dest.put_world_u8(ctx, world_state, result);

                        OpResult::Continue
                    }
                    Op::ByteMath { dest, lhs, op ,rhs } => {
                        let lhs_value = lhs.get_world_u8(ctx, world_state, actor_index);
                        let rhs_value = rhs.get_world_u8(ctx, world_state, actor_index);

                        let result = match op {
                            ByteMathOp::Add => lhs_value.overflowing_add(rhs_value).0,
                            ByteMathOp::Subtract => lhs_value.overflowing_sub(rhs_value).0,
                        };
                        dest.put_world_u8(ctx, world_state, result);

                        OpResult::Continue
                    }
                    Op::SetPosition { x, y } => {
                        OpResult::Continue
                    }
                    Op::SetPriority { priority } => {
                        OpResult::Continue
                    }
                    Op::SetPalette { index } => {
                        OpResult::Continue
                    }
                    Op::SetAnimation { anim_index } => {
                        OpResult::Continue
                    }
                    Op::WaitThenAnimate { delay } => {
                        OpResult::Continue
                    }
                    Op::VectorX { magnitude } => {
                        OpResult::Continue
                    }
                    Op::VectorY { magnitude } => {
                        OpResult::Continue
                    }
                    Op::Scroll { steps } => {
                        OpResult::Continue
                    }
                    Op::ChangeLocation { destination } => {
                        OpResult::Continue
                    }
                    Op::Move { steps } => {
                        OpResult::Continue
                    }
                    Op::PaletteLoad { address, palette_index, mode } => {
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

    pub fn add_special_actor(&mut self, world_state: &mut WorldState, action_func: u64) -> usize {
        for index in 0..4 {
            let state = world_state.actors[index];
            if state.action_function == 0 {
                let mut new_state = WorldActorState::default();
                new_state.action_function = action_func;
                world_state.actors[index] = new_state;
                return index;
            }
        }

        panic!("Out of world special actor slots!");
    }

    pub fn add_actor(&mut self, world_state: &mut WorldState, action_func: u64) -> usize {
        for index in 4..world_state.actors.len() {
            let state = world_state.actors[index];
            if state.action_function == 0 {
                let mut new_state = WorldActorState::default();
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
