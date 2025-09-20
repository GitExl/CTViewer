use crate::actor::{Actor, DebugSprite};
use crate::scene_script::scene_script::OpResult;
use crate::sprites::sprite_state::{AnimationMode, SpriteState};

pub fn exec_animation(state: &mut SpriteState, anim_index: usize) -> OpResult {
    state.anim_index = anim_index;
    state.anim_delay = 0;
    state.anim_loops_remaining = 0;
    state.anim_frame = 0;
    state.anim_mode = AnimationMode::Loop;

    OpResult::COMPLETE
}

pub fn exec_animation_loop_count(state: &mut SpriteState, actor: &mut Actor, anim_index: usize, loop_count: u32) -> OpResult {

    // Start playback.
    if state.anim_loops_remaining == 0 {
        state.anim_index_looped = anim_index;
        state.anim_frame = 0;
        state.anim_delay = 0;

        if state.anim_mode == AnimationMode::None {
            state.anim_index = 0xFF;
        }
        state.anim_mode = AnimationMode::LoopCount;
        state.anim_loops_remaining = loop_count + 1;

        actor.debug_sprite = DebugSprite::Animating;

        return OpResult::YIELD;
    }

    // Check loops remaining, yield if not done.
    if state.anim_loops_remaining > 1 && state.anim_index_looped == anim_index {
        return OpResult::YIELD;
    }

    // Stop playback.
    state.anim_index_looped = 0;
    state.anim_delay = 0;
    state.anim_frame = 0;
    state.anim_loops_remaining = 0;

    if state.anim_index == 0xFF {
        state.anim_index = 0;
        state.anim_mode = AnimationMode::None;
    } else {
        state.anim_mode = AnimationMode::Loop;
    }

    actor.debug_sprite = DebugSprite::None;

    OpResult::COMPLETE
}

pub fn exec_animation_reset(state: &mut SpriteState) -> OpResult {
    state.anim_index = 0;
    state.anim_delay = 0;
    state.anim_loops_remaining = 0;
    state.anim_frame = 0;
    state.anim_mode = AnimationMode::None;

    OpResult::COMPLETE
}

pub fn exec_animation_static_frame(state: &mut SpriteState, frame_index: usize) -> OpResult {
    state.anim_frame_static = frame_index;
    state.anim_frame = 0;
    state.anim_delay = 0;
    if state.anim_mode == AnimationMode::None {
        state.anim_index = 0xFF;
    }
    state.anim_mode = AnimationMode::Static;

    OpResult::YIELD | OpResult::COMPLETE
}
