use crate::Context;
use crate::gamestate::gamestate_world::WorldState;
use crate::shared_op::{BitMathOp, ByteMathOp, CompareOp};
use crate::world_script::exec::tile_copy::exec_tile_copy;
use crate::world_script::function_dispatch::function_dispatch;
use crate::world_script::task_dispatch::WorldActorTask;
use crate::world_script::world_actor::WorldActor;
use crate::world_script::world_script::{world_script_add_actor, world_script_add_special_actor};
use crate::world_script::world_script_decoder::op_decode;
use crate::world_script::world_script_ops::Op;

enum OpResult {
    Yield,
    Continue,
    ContinueFrom {
        address: u64
    },
}

pub fn task_run_script(ctx: &mut Context, world_state: &mut WorldState, actor: &mut WorldActor) {
    loop {
        world_state.script_data.set_position(actor.script_current_address);

        let result;
        if let Some(op) = op_decode(&mut world_state.script_data, ctx.mode) {
            result = match op {
                Op::InitMemory => {
                    actor.memory.clear();
                    actor.script_current_address = 0;
                    actor.sprite_assembly_key = 0;
                    actor.animation_counter = 0;
                    actor.pos.x = 0.0;
                    actor.pos.y = 0.0;
                    actor.vec.x = 0.0;
                    actor.vec.y = 0.0;
                    OpResult::Continue
                }
                Op::Copy8 { lhs, rhs } => {
                    let value = rhs.get_world_u8(ctx, world_state, actor);
                    lhs.put_world_u8(ctx, world_state, actor, value);
                    OpResult::Continue
                }
                Op::Unknown03 { .. } => {
                    world_script_add_actor(world_state, &actor, WorldActorTask::UnknownGrp);
                    OpResult::Continue
                }
                Op::GoSub { address } => {
                    actor.script_return_address = world_state.script_data.position();
                    OpResult::ContinueFrom { address }
                }
                Op::Return => {
                    OpResult::ContinueFrom { address: actor.script_return_address }
                },
                Op::GoTo { address } => {
                    OpResult::ContinueFrom { address }
                }
                Op::InitBackgroundLayer { .. } => {
                    // No implementation needed.
                    OpResult::Continue
                },
                Op::DecrementAndJumpIfNonZero { src, dest, offset } => {
                    let mut value = src.get_world_u8(ctx, world_state, actor);
                    value = value.wrapping_sub(1);
                    dest.put_world_u8(ctx, world_state, actor, value);
                    if value != 0 {
                        OpResult::ContinueFrom { address: (actor.script_current_address as i64 + offset) as u64 }
                    } else {
                        OpResult::Continue
                    }
                }
                Op::JumpConditional { lhs, cmp, rhs, offset } => {
                    let lhs_value = lhs.get_world_u8(ctx, world_state, actor);
                    let rhs_value = rhs.get_world_u8(ctx, world_state, actor);
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
                    let index = world_script_add_actor(world_state, &actor, WorldActorTask::RunScript);
                    let new_actor = &mut world_state.actors[index];
                    new_actor.script_current_address = address;
                    OpResult::Continue
                }
                Op::AddActorSpecial { address, .. } => {
                    let index = world_script_add_special_actor(world_state, &actor, WorldActorTask::RunScript);
                    let new_actor = &mut world_state.actors[index];
                    new_actor.script_current_address = address;
                    OpResult::Continue
                }
                Op::Link { task, .. } => {
                    world_script_add_actor(world_state, &actor, task);
                    OpResult::Continue
                }
                Op::LinkSpecial { task, .. } => {
                    world_script_add_special_actor(world_state, &actor, task);
                    OpResult::Continue
                }
                Op::FadeIn { delay } => {
                    let index = world_script_add_special_actor(world_state, &actor, WorldActorTask::FadeIn);
                    let new_actor = &mut world_state.actors[index];
                    new_actor.memory.put_u8(0x0A, delay);
                    OpResult::Continue
                }
                Op::FadeOut { delay } => {
                    let index = world_script_add_special_actor(world_state, &actor, WorldActorTask::FadeOut);
                    let new_actor = &mut world_state.actors[index];
                    new_actor.memory.put_u8(0x0A, delay);
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
                    actor.task = WorldActorTask::None;
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
                    let lhs_value = lhs.get_world_u8(ctx, world_state, actor);
                    let rhs_value = rhs.get_world_u8(ctx, world_state, actor);
                    let result = match op {
                        BitMathOp::And => lhs_value & rhs_value,
                        BitMathOp::Or => lhs_value | rhs_value,
                        BitMathOp::Xor => lhs_value ^ rhs_value,
                        BitMathOp::AndXor => lhs_value & (rhs_value ^ 0xFF),
                        BitMathOp::ShiftLeft => lhs_value << rhs_value,
                        BitMathOp::ShiftRight => lhs_value >> rhs_value,
                    };
                    dest.put_world_u8(ctx, world_state, actor, result);

                    OpResult::Continue
                }
                Op::ByteMath { dest, lhs, op ,rhs } => {
                    let lhs_value = lhs.get_world_u8(ctx, world_state, actor);
                    let rhs_value = rhs.get_world_u8(ctx, world_state, actor);

                    let result = match op {
                        ByteMathOp::Add => lhs_value.overflowing_add(rhs_value).0,
                        ByteMathOp::Subtract => lhs_value.overflowing_sub(rhs_value).0,
                    };
                    dest.put_world_u8(ctx, world_state, actor, result);

                    OpResult::Continue
                }
                Op::SetPosition { x, y } => {
                    actor.pos.x = x as f64;
                    actor.pos.y = y as f64;
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
                        if actor.counter != 0 {
                            world_state.animations.run(ctx, actor);
                            OpResult::Yield
                        } else {
                            OpResult::Continue
                        }
                    } else {
                        actor.counter = steps;
                        world_state.animations.run(ctx, actor);
                        OpResult::Yield
                    }
                }
                Op::VectorX { magnitude } => {
                    actor.vec.x = magnitude as f64 / 65536.0;
                    OpResult::Continue
                }
                Op::VectorY { magnitude } => {
                    actor.vec.y = magnitude as f64 / 65536.0;
                    OpResult::Continue
                }
                Op::Scroll { steps } => {
                    if actor.counter != 0 {
                        actor.counter -= 1;
                    } else {
                        actor.counter = steps;
                    }
                    if actor.counter != 0 {
                        actor.pos.x += actor.vec.x;
                        actor.pos.y += actor.vec.y;

                        world_state.camera.pos.x += actor.vec.x;
                        world_state.camera.pos.y += actor.vec.y;

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
                        actor.pos.x += actor.vec.x;
                        actor.pos.y += actor.vec.y;

                        world_state.map.layers[layer].scroll.x += actor.vec.x;
                        world_state.map.layers[layer].scroll.y += actor.vec.x;

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
                        actor.pos.x += actor.vec.x;
                        actor.pos.y += actor.vec.y;

                        // Clamp to map.
                        actor.pos.x = actor.pos.x.min(world_state.world_map.pixel_width as f64).max(0.0);
                        actor.pos.y = actor.pos.y.min(world_state.world_map.pixel_height as f64).max(0.0);

                        world_state.animations.run(ctx, actor);

                        OpResult::Yield
                    } else {
                        OpResult::Continue
                    }
                }
                Op::PaletteLoad { address, palette_index, mode } => {
                    let new_index = world_script_add_actor(world_state, &actor, WorldActorTask::PaletteLoad);
                    let new_actor = &mut world_state.actors[new_index];
                    new_actor.memory.put_u8(0x32, palette_index);
                    new_actor.memory.put_u8(0x33, 0);
                    new_actor.memory.put_u8(0x34, mode);
                    new_actor.memory.put_u24(0x35, address as u32);

                    OpResult::Continue
                }
                Op::CallFunction { function, .. } => {
                    function_dispatch(ctx, world_state, actor, function);
                    OpResult::Continue
                }
                Op::CallFunctionFar { function, .. } => {
                    function_dispatch(ctx, world_state, actor, function);
                    OpResult::Continue
                }

                Op::ExitOpen { exit_type, exit_index } => {
                    match exit_type {
                        0 => {
                            world_state.exits[exit_index].is_available = true;
                        },
                        1 => {
                            world_state.triggers[exit_index].is_available = true;
                        },
                        _ => println!("Cannot open unsupported exit type {}", exit_type),
                    }
                    OpResult::Continue
                }
                Op::ExitClose { exit_type, exit_index } => {
                    match exit_type {
                        0 => {
                            world_state.exits[exit_index].is_available = false;
                        },
                        1 => {
                            world_state.triggers[exit_index].is_available = false;
                        },
                        _ => println!("Cannot close unsupported exit type {}", exit_type),
                    }
                    OpResult::Continue
                }

                _ => {
                    println!("Unimplemented world script op {:?}", op);
                    OpResult::Continue
                }
            };
        } else {
            result = OpResult::Continue;
        }

        match result {
            OpResult::Continue => {
                actor.script_current_address = world_state.script_data.position();
            }
            OpResult::ContinueFrom { address }=> {
                actor.script_current_address = address;
            }
            OpResult::Yield => {
                break;
            }
        }
    }
}
