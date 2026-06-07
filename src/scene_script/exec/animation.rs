use crate::scene::actor::{SceneActor, DebugSprite};
use crate::scene_script::scene_script::OpResult;
use crate::sprites::sprite_state::AnimationMode;

pub fn exec_animation(actor: &mut SceneActor, anim_index: usize) -> OpResult {
    actor.anim_index = anim_index;
    actor.anim_delay = 0;
    actor.anim_loops_remaining = 0;
    actor.anim_frame = 0;
    actor.anim_mode = AnimationMode::Loop;

    OpResult::COMPLETE
}

pub fn exec_animation_loop_count(actor: &mut SceneActor, anim_index: usize, loop_count: u32) -> OpResult {

    // Stop playback.
    if actor.anim_loops_remaining == 1 {
        actor.anim_loops_remaining = 0;
        actor.anim_delay = 0;
        actor.anim_frame = 0;

        if actor.anim_index == 0xFF {
            actor.anim_index = 0;
            actor.anim_mode = AnimationMode::None;
        } else {
            actor.anim_mode = AnimationMode::Loop;
        }

        actor.debug_sprite = DebugSprite::None;

        return OpResult::COMPLETE;
    }

    // Start playback.
    if actor.anim_loops_remaining == 0 || actor.anim_index_looped != anim_index {
        actor.anim_index_looped = anim_index;
        actor.anim_frame = 0;
        actor.anim_delay = 0;

        if actor.anim_mode == AnimationMode::None {
            actor.anim_index = 0xFF;
        }
        actor.anim_mode = AnimationMode::LoopCount;
        actor.anim_loops_remaining = loop_count + 1;

        actor.debug_sprite = DebugSprite::Animating;
    }

    OpResult::YIELD
}

pub fn exec_animation_reset(actor: &mut SceneActor) -> OpResult {
    actor.anim_index = 0;
    actor.anim_delay = 0;
    actor.anim_loops_remaining = 0;
    actor.anim_frame = 0;
    actor.anim_mode = AnimationMode::None;

    OpResult::COMPLETE
}

pub fn exec_animation_static_frame(actor: &mut SceneActor, frame_index: usize) -> OpResult {
    actor.anim_frame_static = frame_index;
    actor.anim_frame = 0;
    actor.anim_delay = 0;
    if actor.anim_mode == AnimationMode::None {
        actor.anim_index = 0xFF;
    }
    actor.anim_mode = AnimationMode::Static;

    OpResult::YIELD | OpResult::COMPLETE
}
