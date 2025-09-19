# Execution flow

## Actor data

These execution-flow related properties are available for each actor in a scene script.

| Name               | SNES address | Description                                                                                                                          |
|--------------------|-------------|--------------------------------------------------------------------------------------------------------------------------------------|
| `script_delay`     | `$7E1000`   | Current script execution delay. If bit `$80` is set, script execution is disabled.                                                   |
| `type_status`      | `$7E1100`   | The type of actor. If bit `$80` is set, the actor is considered to be "dead".                                                        |
| `script_ptr`       | `$7E1180`   | A 16-bit pointer to the current op data in the scene script that is being executed.                                                  |
| `current_priority` | `$7E1C00`   | The current priority level.                                                                                                          |
| `calls_disabled`   | `$7E1C01`   | If set, function calls on this actor cannot be executed. Call op `$04` and `$07` will wait for this flag to be cleared.              |
| `priority_ptrs`    | `$7F0580`   | 8 16-bit pointers of addresses to return to once a lower priority call returns. Lower pointers correspond to higher priority levels. |
| `call_busy`        | `$7F0980`   | Used by call op `$04` and `$07` to track that the current actor is busy waiting for a function call on another actor.                |

## Ops

- `$00` Return
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

## Priority levels

## Calling functions and returning

## Yielding
