use std::io::Cursor;
use crate::{Context, GameMode};
use crate::gamestate::gamestate_world::WorldState;
use crate::memory::MemoryRegion;
use crate::shared_op::{BitMathOp, ByteMathOp, CompareOp};
use crate::util::rect::Rect;
use crate::world_script::exec::tile_copy::exec_tile_copy;
use crate::world_script::world_script_decoder::op_decode;
use crate::world_script::world_script_disassembler::WorldScriptDisassembler;
use crate::world_script::world_script_ops::Op;

const ACTION_FUNC_RUN_SCRIPT: u32 = 0x0F63;

const ACTION_FUNC_FADE_IN: u32 = 0x2105;
const ACTION_FUNC_FADE_OUT: u32 = 0x20A2;

const ACTION_FUNC_PALETTE_LOAD: u32 = 0x1DD4;
const ACTION_FUNC_PALETTE_LOAD_MODES: u32 = 0x1E38;

const ACTION_FUNC_SCROLL_LAYERS_WORLD0: u32 = 0x75C3;
const ACTION_FUNC_SCROLL_LAYERS_WORLD1: u32 = 0x75FD;
const ACTION_FUNC_SCROLL_LAYERS_WORLD2: u32 = 0x7702;
const ACTION_FUNC_SCROLL_LAYERS_WORLD4: u32 = 0x77F2;
const ACTION_FUNC_SCROLL_LAYERS_WORLD5: u32 = 0x7849;

const FUNC_SEAGULL_RANDOM_POS: u32 = 0x7575;
const FUNC_SEAGULL_RANDOM_VECTOR: u32 = 0x7598;
const FUNC_ACTOR_IS_OFFSCREEN: u32 = 0x78A1;

enum OpResult {
    Yield,
    Continue,
    ContinueFrom {
        address: u64
    },
}

#[derive(Clone)]
pub struct WorldActorState {
    pub action_function: u32,
    pub cycles: u16,
    pub counter: u8,

    pub script_return_address: u64,
    pub script_current_address: u64,

    pub palette_priority: u8,
    pub animation_address: u64,
    pub animation_counter: u8,
    pub sprite_assembly_key: u64,

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
            action_function: 0,
            cycles: 0,
            counter: 0,

            script_current_address: 0,
            script_return_address: 0,

            palette_priority: 0,
            animation_counter: 0,
            animation_address: 0,
            sprite_assembly_key: 0,

            memory: MemoryRegion::new(64),
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
        self.add_actor(world_state, ACTION_FUNC_RUN_SCRIPT, 0);
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
                self.run_layer_animation(world_state, 0);
            } else if state.action_function == ACTION_FUNC_SCROLL_LAYERS_WORLD1 {
                self.run_layer_animation(world_state, 1);
            } else if state.action_function == ACTION_FUNC_SCROLL_LAYERS_WORLD2 {
                self.run_layer_animation(world_state, 2);
            } else if state.action_function == ACTION_FUNC_SCROLL_LAYERS_WORLD4 {
                self.run_layer_animation(world_state, 4);
            } else if state.action_function == ACTION_FUNC_SCROLL_LAYERS_WORLD5 {
                self.run_layer_animation(world_state, 5);

            } else if state.action_function == ACTION_FUNC_FADE_IN {
                self.run_fade_in(ctx, actor_index, world_state);
            } else if state.action_function == ACTION_FUNC_FADE_OUT {
                self.run_fade_out(ctx, actor_index, world_state);
            }

            world_state.actors.get_mut(actor_index).unwrap().cycles = cycles;
        }
        // println!("-------------");
    }

    fn run_script(&mut self, ctx: &mut Context, actor_index: usize, world_state: &mut WorldState) {
        let mut actor = world_state.actors[actor_index].clone();

        loop {
            self.data.set_position(actor.script_current_address);

            let result;
            if let Some(op) = op_decode(&mut self.data, self.mode) {
                // println!("{:02} {:04X} {:?}", actor_index, actor.script_current_address, op);
                result = match op {
                    Op::InitMemory => {
                        OpResult::Continue
                    }
                    Op::Copy8 { lhs, rhs } => {
                        let value = rhs.get_world_u8(ctx, world_state, &mut actor);
                        lhs.put_world_u8(ctx, world_state, &mut actor, value);
                        OpResult::Continue
                    }
                    Op::GoSub { address } => {
                        actor.script_return_address = self.data.position();
                        OpResult::ContinueFrom { address }
                    }
                    Op::Return => {
                        OpResult::ContinueFrom { address: actor.script_return_address }
                    },
                    Op::GoTo { address } => {
                        OpResult::ContinueFrom { address }
                    }
                    Op::DecrementAndJumpIfNonZero { src, dest, offset } => {
                        let mut value = src.get_world_u8(ctx, world_state, &mut actor);
                        value = value.wrapping_sub(1);
                        dest.put_world_u8(ctx, world_state, &mut actor, value);
                        if value != 0 {
                            OpResult::ContinueFrom { address: (actor.script_current_address as i64 + offset) as u64 }
                        } else {
                            OpResult::Continue
                        }
                    }
                    Op::JumpConditional { lhs, cmp, rhs, offset } => {
                        let lhs_value = lhs.get_world_u8(ctx, world_state, &mut actor);
                        let rhs_value = rhs.get_world_u8(ctx, world_state, &mut actor);
                        let result = match cmp {
                            CompareOp::Eq => lhs_value == rhs_value,
                            CompareOp::NotEq => lhs_value != rhs_value,
                            CompareOp::Gt => lhs_value > rhs_value,
                            CompareOp::GtEq => lhs_value >= rhs_value,
                            CompareOp::Lt => lhs_value < rhs_value,
                            CompareOp::LtEq => lhs_value <= rhs_value,
                            CompareOp::And => (lhs_value & rhs_value) > 0,
                            CompareOp::Or => (lhs_value | rhs_value) > 0,
                            CompareOp::AndZero => (lhs_value & rhs_value) == 0,
                        };
                        if result {
                            OpResult::ContinueFrom {
                                address: (actor.script_current_address as i64 + offset) as u64,
                            }
                        } else {
                            OpResult::Continue
                        }
                    }
                    Op::AddActor { address, .. } => {
                        let index = self.add_actor(world_state, ACTION_FUNC_RUN_SCRIPT, actor_index);
                        let new_state = world_state.actors.get_mut(index).unwrap();
                        new_state.script_current_address = address;
                        OpResult::Continue
                    }
                    Op::AddActorSpecial { address, .. } => {
                        let index = self.add_special_actor(world_state, ACTION_FUNC_RUN_SCRIPT, actor_index);
                        let new_state = world_state.actors.get_mut(index).unwrap();
                        new_state.script_current_address = address;
                        OpResult::Continue
                    }
                    Op::Link { address } => {
                        self.add_actor(world_state, address, actor_index);
                        OpResult::Continue
                    }
                    Op::LinkSpecial { address } => {
                        self.add_special_actor(world_state, address, actor_index);
                        OpResult::Continue
                    }
                    Op::FadeIn { delay } => {
                        let index = self.add_special_actor(world_state, ACTION_FUNC_FADE_IN, actor_index);
                        let new_state = world_state.actors.get_mut(index).unwrap();
                        new_state.memory.put_u8(0x0A, delay);
                        OpResult::Continue
                    }
                    Op::FadeOut { delay } => {
                        let index = self.add_special_actor(world_state, ACTION_FUNC_FADE_OUT, actor_index);
                        let new_state = world_state.actors.get_mut(index).unwrap();
                        new_state.memory.put_u8(0x0A, delay);
                        OpResult::Continue
                    }
                    Op::Wait { steps } => {
                        if actor.counter != 0 {
                            actor.counter -= 1;
                        } else {
                            actor.counter = steps;
                        }
                        if actor.counter != 0 {
                            OpResult::Yield
                        } else {
                            OpResult::Continue
                        }
                    }
                    Op::End => {
                        actor.action_function = 0;
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
                        let lhs_value = lhs.get_world_u8(ctx, world_state, &mut actor);
                        let rhs_value = rhs.get_world_u8(ctx, world_state, &mut actor);
                        let result = match op {
                            BitMathOp::And => lhs_value & rhs_value,
                            BitMathOp::Or => lhs_value | rhs_value,
                            BitMathOp::Xor => lhs_value ^ rhs_value,
                            BitMathOp::ShiftLeft => lhs_value << rhs_value,
                            BitMathOp::ShiftRight => lhs_value >> rhs_value,
                        };
                        dest.put_world_u8(ctx, world_state, &mut actor, result);

                        OpResult::Continue
                    }
                    Op::ByteMath { dest, lhs, op ,rhs } => {
                        let lhs_value = lhs.get_world_u8(ctx, world_state, &mut actor);
                        let rhs_value = rhs.get_world_u8(ctx, world_state, &mut actor);

                        let result = match op {
                            ByteMathOp::Add => lhs_value.overflowing_add(rhs_value).0,
                            ByteMathOp::Subtract => lhs_value.overflowing_sub(rhs_value).0,
                        };
                        dest.put_world_u8(ctx, world_state, &mut actor, result);

                        OpResult::Continue
                    }
                    Op::SetPosition { x, y } => {
                        actor.x = x as f64;
                        actor.y = y as f64;
                        OpResult::Continue
                    }
                    Op::SetPriority { priority } => {
                        actor.palette_priority = (actor.palette_priority & 0x4F) | priority;
                        OpResult::Continue
                    }
                    Op::SetPalette { index } => {
                        actor.palette_priority = (actor.palette_priority & 0xF1) | index;
                        OpResult::Continue
                    }
                    Op::SetAnimation { anim_index } => {
                        actor.animation_address = world_state.animations.get_animation_address(anim_index);
                        OpResult::Continue
                    }
                    Op::WaitAndAnimate { steps } => {
                        if actor.counter != 0 {
                            actor.counter -= 1;
                        } else {
                            actor.counter = steps;
                        }
                        if actor.counter != 0 {
                            world_state.animations.run(ctx, &mut actor);
                            OpResult::Yield
                        } else {
                            OpResult::Continue
                        }
                    }
                    Op::VectorX { magnitude } => {
                        actor.vector_x = magnitude as f64 / 65536.0;
                        OpResult::Continue
                    }
                    Op::VectorY { magnitude } => {
                        actor.vector_y = magnitude as f64 / 65536.0;
                        OpResult::Continue
                    }
                    Op::Scroll { steps } => {
                        if actor.counter != 0 {
                            actor.counter -= 1;
                        } else {
                            actor.counter = steps;
                        }
                        if actor.counter != 0 {
                            actor.x += actor.vector_x;
                            actor.y += actor.vector_y;

                            world_state.camera.pos.x += actor.vector_x;
                            world_state.camera.pos.y += actor.vector_y;

                            OpResult::Yield
                        } else {
                            OpResult::Continue
                        }
                    }
                    Op::ScrollLayer { layer, steps } => {
                        if actor.counter != 0 {
                            actor.counter -= 1;
                        } else {
                            actor.counter = steps;
                        }
                        if actor.counter != 0 {
                            actor.x += actor.vector_x;
                            actor.y += actor.vector_y;

                            world_state.map.layers[layer].scroll.x += actor.vector_x;
                            world_state.map.layers[layer].scroll.y += actor.vector_x;

                            OpResult::Yield
                        } else {
                            OpResult::Continue
                        }
                    }
                    Op::ChangeLocation { destination } => {
                        world_state.next_destination.set(destination, true);
                        OpResult::Continue
                    }
                    Op::Move { steps } => {
                        if actor.counter != 0 {
                            actor.counter -= 1;
                        } else {
                            actor.counter = steps;
                        }

                        if actor.counter != 0 {

                            // Move actor by vector.
                            actor.x += actor.vector_x;
                            actor.y += actor.vector_y;

                            // Clamp to map.
                            actor.x = actor.x.min(world_state.world_map.pixel_width as f64).max(0.0);
                            actor.y = actor.y.min(world_state.world_map.pixel_height as f64).max(0.0);

                            world_state.animations.run(ctx, &mut actor);

                            OpResult::Yield
                        } else {
                            OpResult::Continue
                        }
                    }
                    Op::PaletteLoad { address, palette_index, mode } => {
                        let new_index = self.add_actor(world_state, ACTION_FUNC_PALETTE_LOAD, actor_index);
                        let new_state = world_state.actors.get_mut(new_index).unwrap();
                        new_state.memory.put_u8(0x32, palette_index);
                        new_state.memory.put_u8(0x33, 0);
                        new_state.memory.put_u8(0x34, mode);
                        new_state.memory.put_u24(0x35, address as u32);

                        OpResult::Continue
                    }
                    Op::CallFunction { address } => {
                        self.call_function(address, ctx, actor_index, world_state);
                        OpResult::Continue
                    }
                    Op::CallFunctionFar { address } => {
                        self.call_function(address, ctx, actor_index, world_state);
                        OpResult::Continue
                    }

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
                    actor.script_current_address = self.data.position();
                }
                OpResult::ContinueFrom { address }=> {
                    actor.script_current_address = address;
                }
                OpResult::Yield => {
                    break;
                }
            }
        }

        world_state.actors[actor_index] = actor;
    }

    fn call_function(&mut self, func_address: u32, ctx: &mut Context, actor_index: usize, world_state: &mut WorldState) {
        if func_address == FUNC_SEAGULL_RANDOM_POS {
            self.run_seagull_random_pos(ctx, actor_index, world_state);
        } else if func_address == FUNC_SEAGULL_RANDOM_VECTOR {
            self.run_seagull_random_vector(ctx, actor_index, world_state);
        } else if func_address == FUNC_ACTOR_IS_OFFSCREEN {
            self.run_actor_is_offscreen(ctx, actor_index, world_state);
        }
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
            self.add_actor(world_state, ACTION_FUNC_PALETTE_LOAD_MODES, actor_index);
        }
    }

    fn run_layer_animation(&mut self, world_state: &mut WorldState, world_index: usize) {
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

                // Do not interpolate this because of the fast scrolling effect.
                world_state.map.layers[2].scroll_last = world_state.map.layers[2].scroll;
                world_state.map.layers[2].scroll_lerp = world_state.map.layers[2].scroll;
            }
            4 => {
                world_state.map.layers[2].scroll.x -= 21003.0002746582;
                world_state.map.layers[2].scroll.y -= 10.0;

                // Do not interpolate this because of the fast scrolling effect.
                world_state.map.layers[2].scroll_last = world_state.map.layers[2].scroll;
                world_state.map.layers[2].scroll_lerp = world_state.map.layers[2].scroll;
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

    fn run_fade_in(&mut self, ctx: &mut Context, actor_index: usize, world_state: &mut WorldState) {
        let state = world_state.actors.get_mut(actor_index).unwrap();
        state.action_function = 0;

        let delay = state.memory.get_u8(0x0A) as usize;
        ctx.screen_fade.start(1.0, delay);
    }

    fn run_fade_out(&mut self, ctx: &mut Context, actor_index: usize, world_state: &mut WorldState) {
        let state = world_state.actors.get_mut(actor_index).unwrap();
        state.action_function = 0;

        let delay = state.memory.get_u8(0x0A) as usize;
        ctx.screen_fade.start(0.0, delay);
    }

    fn run_seagull_random_pos(&mut self, ctx: &mut Context, actor_index: usize, world_state: &mut WorldState) {
        let actor = world_state.actors.get_mut(actor_index).unwrap();

        actor.x = 376.0 + ctx.random.get_u8() as f64;
        actor.y = world_state.camera.pos.y + 256.0;
    }

    fn run_seagull_random_vector(&mut self, ctx: &mut Context, actor_index: usize, world_state: &mut WorldState) {
        static VECTORS: [[f64; 2]; 16] = [
            [0.0, -1.0],
            [0.0, -1.5],
            [0.0, -1.0],
            [0.7071075439453125, -0.7071075439453125],
            [1.414215087890625, -1.414215087890625],
            [0.7071075439453125, -0.7071075439453125],
            [0.5, -0.86602783203125],
            [0.75, -1.299041748046875],
            [0.5, -0.86602783203125],
            [0.5, -0.86602783203125],
            [0.75, -1.299041748046875],
            [0.5, -0.86602783203125],
            [-0.5, -0.86602783203125],
            [-0.25, -1.299041748046875],
            [-0.5, -0.86602783203125],
            [0.0, -1.0],
        ];

        let actor = world_state.actors.get_mut(actor_index).unwrap();
        let index = (ctx.random.get_u8() & 0x0F) as usize;
        actor.vector_x = VECTORS[index][0];
        actor.vector_y = VECTORS[index][1];
    }

    fn run_actor_is_offscreen(&mut self, ctx: &mut Context, actor_index: usize, world_state: &mut WorldState) {
        let actor = &world_state.actors[actor_index];

        let camera_rect = Rect::new(
            world_state.camera.pos.x as i32,
            world_state.camera.pos.y as i32,
            world_state.camera.pos.x as i32 + world_state.camera.size.x as i32,
            world_state.camera.pos.y as i32 + world_state.camera.size.y as i32,
        );
        let actor_rect = Rect::new(
            actor.x as i32 - 32,
            actor.y as i32 - 32,
            actor.x as i32 + 32,
            actor.y as i32 + 32,
        );

        if actor_rect.intersects(&camera_rect) {
            ctx.memory.write_u8(0x7E0000, 1);
        } else {
            ctx.memory.write_u8(0x7E0000, 0);
        }
    }

    pub fn add_special_actor(&mut self, world_state: &mut WorldState, action_func: u32, source_actor_index: usize) -> usize {
        for index in 0..4 {
            let state = &world_state.actors[index];
            if state.action_function == 0 {
                let source_actor = &world_state.actors[source_actor_index];
                world_state.actors[index] = source_actor.clone();
                world_state.actors[index].action_function = action_func;
                world_state.actors[index].cycles = 0;
                return index;
            }
        }

        panic!("Out of world special actor slots!");
    }

    pub fn add_actor(&mut self, world_state: &mut WorldState, action_func: u32, source_actor_index: usize) -> usize {
        for index in 4..world_state.actors.len() {
            let actor = &world_state.actors[index];
            if actor.action_function == 0 {
                let source_actor = &world_state.actors[source_actor_index];
                world_state.actors[index] = source_actor.clone();
                world_state.actors[index].action_function = action_func;
                // Clear what is in $02 once we know what it is.
                world_state.actors[index].cycles = 0;
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
