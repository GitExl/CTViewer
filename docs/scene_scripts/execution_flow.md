# Execution flow

## Actor data

These execution-flow related properties are available for each actor in a scene script.

| Name                   | SNES address | Description                                                                                                                          |
|------------------------|--------------|--------------------------------------------------------------------------------------------------------------------------------------|
| `script_delay`         | `$7E1000`    | Current script execution delay. If bit `$80` is set, script execution is disabled.                                                   |
| `script_delay_counter` | `$7E1001`    | Decremented every tick. Scripts are executed when this reaches the `script_delay` value.                                             |
| `type_status`          | `$7E1100`    | The type of actor. If bit `$80` is set, the actor is considered to be "dead".                                                        |
| `script_ptr`           | `$7E1180`    | A 16-bit pointer to the current op data in the scene script that is being executed.                                                  |
| `current_priority`     | `$7E1C00`    | The current priority level.                                                                                                          |
| `calls_disabled`       | `$7E1C01`    | If set, function calls on this actor cannot be executed. Call op `$04` and `$07` will wait for this flag to be cleared.              |
| `priority_ptrs`        | `$7F0580`    | 8 16-bit pointers of addresses to return to once a lower priority call returns. Lower pointers correspond to higher priority levels. |
| `call_busy`            | `$7F0980`    | Used by call op `$04` and `$07` to track that the current actor is busy waiting for a function call on another actor.                |

## Ops

- `$00` Return to the first available lower priority pointer
- `$02` Call function `x` on actor `y` with priority `z`
- `$03` Call function `x` on actor `y` with priority `z`, wait for higher priority functions
- `$04` Call function `x` on actor `y` with priority `z`, wait for higher priority functions, then wait for completion
- `$05` Call function `x` on player `y` with priority `z`
- `$06` Call function `x` on player `y` with priority `z`, wait for higher priority functions
- `$07` Call function `x` on player `y` with priority `z`, wait for higher priority functions, then wait for completion
- `$87` Set script execution delay to `x`
- `$AD` Wait for `x` script cycles
- `$AF` Yield
- `$B0` Yield forever
- `$B9` Wait for 4 script cycles
- `$BA` Wait for 8 script cycles
- `$BC` Wait for 16 script cycles
- `$BD` Wait for 32 script cycles

## Script cycles

All scripts are iterated every tick. Their `script_delay_counter` value is decremented. When it is 1 or 0, the value is
reset to `script_delay`and the script for that actor is run; this is a single script cycle. During a cycle a maximum of
5 ops are executed, starting from the `script_ptr` address. If an op yields execution, the cycle ends early.

## Priority levels

Each actor has 8 pointers, one for each priority level. When a function is called on an actor through one of the
call ops, the caller must either not call the function if it is of lower priority than the current priority, or wait
calling it until later time. When a function is successfully called on an actor, the current `script_ptr` of the actor
is stored at the current priority level from the `priority_ptrs`, and execution continues from the called function. When
the return op is called, the current priority level pointer is zeroed. The next valid lower priority address is searched
for, which becomes the new `script_ptr` so that execution continues there. In effect the return function resumes
execution from the next lower priority pointer.

## Initialization

After a scene has loaded, actors are created for each actor in the script. Their priority level is set to 7, and their
entry address is set to the start of the first actor function. Execution starts there. When a scene is loaded the
initialization function is run until the first occurrence of a return op. An actual return while at priority level 7 is
ignored. Execution continues past the return op during normal code execution.
