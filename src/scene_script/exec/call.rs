use crate::actor::{Actor, ActorFlags, ActorTask};
use crate::scene_script::scene_script::{ActorScriptState, OpResult};

pub fn exec_call_return(state: &mut ActorScriptState) -> OpResult {

    // If the current priority is the least urgent, just yield.
    // Priority 7 is what the object init scripts are started at.
    if state.current_priority == 7 {
        return OpResult::YIELD | OpResult::COMPLETE;
    }

    // Remove the current priority pointer.
    state.priority_return_ptrs[state.current_priority] = 0;

    // Find the next non-zero lower priority pointer and use that as the current priority.
    // Priority 7 should always be available, as that is the init function.
    for priority_index in state.current_priority + 1..8 {
        state.current_priority = priority_index;
        state.current_address = state.priority_return_ptrs[priority_index];
        if state.priority_return_ptrs[priority_index] != 0 {
            break;
        }
    }

    OpResult::COMPLETE | OpResult::JUMPED
}

pub fn exec_call(target_actor: &mut Actor, target_state: &mut ActorScriptState, function: usize, priority: usize) -> OpResult {

    // Complete immediately if the object is not interactive, dead or disabled.
    if target_actor.flags.contains(ActorFlags::CALLS_DISABLED) {
        return OpResult::COMPLETE;
    }
    if target_actor.flags.contains(ActorFlags::DEAD) {
        return OpResult::COMPLETE;
    }
    if target_actor.flags.contains(ActorFlags::SCRIPT_DISABLED) {
        return OpResult::COMPLETE;
    }

    // Complete if the current priority is the same as the call priority.
    if target_state.current_priority == priority {
        return OpResult::COMPLETE;
    }

    // If the current priority is more important than the new one, store the new function address
    // at that priority pointer, if not already set. A future call to return can then exit to the
    // new function.
    if target_state.current_priority < priority {
        if target_state.priority_return_ptrs[target_state.current_priority] > 0 {
            return OpResult::COMPLETE;
        }

        target_state.priority_return_ptrs[priority] = target_state.function_ptrs[function];
        return OpResult::COMPLETE;
    }

    // The new priority is more important than the current one. Set the priority pointer to
    // the new function and immediately continue execution there.
    target_state.priority_return_ptrs[target_state.current_priority] = target_state.current_address;
    target_state.current_address = target_state.function_ptrs[function];
    target_state.current_priority = priority;

    // The new function must interrupt any active movement/task.
    target_actor.task = ActorTask::None;

    OpResult::COMPLETE
}

pub fn exec_call_wait_completion(target_actor: &mut Actor, target_state: &mut ActorScriptState, function: usize, priority: usize) -> OpResult {
    // Wait until a non-interactive target object becomes interactive.
    if target_actor.flags.contains(ActorFlags::CALLS_DISABLED) {
        return OpResult::YIELD;
    }

    // Complete immediately if the object is dead or disabled.
    if target_actor.flags.contains(ActorFlags::DEAD) {
        return OpResult::COMPLETE;
    }
    if target_actor.flags.contains(ActorFlags::SCRIPT_DISABLED) {
        return OpResult::COMPLETE;
    }

    // Complete if the current priority is the same as the call priority.
    if target_state.current_priority <= priority {
        return OpResult::YIELD;
    }

    // The new priority is more important than the current one. Set the priority pointer to
    // the new function and immediately continue execution there.
    target_state.priority_return_ptrs[target_state.current_priority] = target_state.current_address;
    target_state.current_address = target_state.function_ptrs[function];
    target_state.current_priority = priority;

    // The new function must interrupt any active movement/task.
    target_actor.task = ActorTask::None;

    OpResult::COMPLETE
}

pub fn exec_call_wait_return(state: &mut ActorScriptState, target_actor: &mut Actor, target_state: &mut ActorScriptState, function: usize, priority: usize) -> OpResult {
    if !state.call_waiting {

        // Wait until a non-interactive target object becomes interactive.
        if target_actor.flags.contains(ActorFlags::CALLS_DISABLED) {
            return OpResult::YIELD;
        }

        // Complete immediately if the object is dead or disabled.
        if target_actor.flags.contains(ActorFlags::DEAD) {
            return OpResult::COMPLETE;
        }
        if target_actor.flags.contains(ActorFlags::SCRIPT_DISABLED) {
            return OpResult::COMPLETE;
        }

        // Wait until the target object is done executing a function of
        // the same or more importance.
        if target_state.current_priority <= priority {
            return OpResult::YIELD;
        }

        // The new priority is more important than the current one. Set the priority pointer to the
        // new function and immediately continue execution there.
        target_state.priority_return_ptrs[target_state.current_priority] = target_state.current_address;
        target_state.current_address = target_state.function_ptrs[function];
        target_state.current_priority = priority;

        // The new function must interrupt any active movement.
        target_actor.task = ActorTask::None;

        // We are now waiting on the target object to finish their function.
        state.call_waiting = true;

        return OpResult::YIELD;
    }

    // Complete immediately if the object is dead or disabled.
    if target_actor.flags.contains(ActorFlags::DEAD) {
        state.call_waiting = false;
        return OpResult::COMPLETE;
    }
    if target_actor.flags.contains(ActorFlags::SCRIPT_DISABLED) {
        state.call_waiting = false;
        return OpResult::COMPLETE;
    }

    // Wait until the target object is done executing our previously set
    // function call.
    if target_state.current_priority <= priority {
        return OpResult::YIELD;
    }

    // The call we were waiting for has completed.
    state.call_waiting = false;
    OpResult::COMPLETE
}
