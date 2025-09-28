# Script execution flow

## Actor script functions

Each actor has 16 script function pointers. Function pointer 0 is the initialization code for the actor. This will also
continue running when the actor has no other active function. Function pointer 1 is an activation function. This will be
called for the actor if the player presses A while facing the actor. Function pointer 2 is a touch function, called
when a player touches the actor.

## Script cycles

All actor scripts are iterated every tick. Their `script_delay_counter` value is decremented. When it is 1 or 0, the
value is reset to `script_delay` and the script for that actor is run; this constituates a single script cycle. During
a cycle a maximum of 5 ops are executed, starting from the `current_address` address. If an op yields execution, the cycle
ends early. The default `script_delay` value is `$04`.

## Priority levels

Each actor has 8 pointers, one for each priority level. When a function is called on an actor through one of the
call ops, the caller must either not call the function if it is of lower priority than the current priority, or wait
calling it until later time. Note that a lower priority value, is a higher priority level; 0 is the highest priority,
7 is the lowest.

When a function is successfully called on an actor, the current `current_address` of the actor is stored at the current
priority level in the `priority_ptrs` list, and execution continues from the called function.

When the `return` op is called, the current priority level pointer is zeroed. The next non-zero lower priority address
is searched for, which becomes the new `current_address` so that execution can continue from there. In effect the `return`
function resumes execution from the next lower priority pointer.

## Yielding

Any op can yield execution. This means that instead of executing the 5 ops this script cycle, the cycle ends. So this
yields execution to other actor scripts. Yielding is used when actors are waiting for something else to happen or want
to interrupt the script cycle execution to ensure something happens right away, such as changing an actor's facing.

Op documentation will mention under what circumstances an op yields execution.

## Initialization

After a scene has loaded, actors are created for each actor in the script. Their priority level is set to 7, and their
entry address is set to the start of the first actor function. Execution starts there. When a scene is loaded that
initialization function of each actor is run until the first occurrence of a `return`. Execution continues past
the return op during normal code execution. An actual return while at priority level 7 is ignored and will yield
forever.

After all actors have been initialized, function 1 (activation) of actor 0 is run until it reaches a `return` op.
This is a sort of "scene initialization" function.

## Actor data

These execution-flow related properties are available for each actor in a scene script.

| Name                   | SNES address | Description                                                                                                                          |
|------------------------|--------------|--------------------------------------------------------------------------------------------------------------------------------------|
| `script_delay`         | `$7E1000`    | Current script execution delay. If bit `$80` is set, script execution is disabled.                                                   |
| `script_delay_counter` | `$7E1001`    | Decremented every tick. Scripts are executed when this reaches the `script_delay` value.                                             |
| `type_status`          | `$7E1100`    | The type of actor. If bit `$80` is set, the actor is considered to be "dead". Actors of type 7 (the default) are not drawn.          |
| `current_address`      | `$7E1180`    | A 16-bit pointer to the current op data in the scene script that is being executed.                                                  |
| `current_priority`     | `$7E1C00`    | The current priority level.                                                                                                          |
| `calls_disabled`       | `$7E1C01`    | If set, function calls on this actor cannot be executed. Call op `$04` and `$07` will wait for this flag to be cleared.              |
| `priority_ptrs`        | `$7F0580`    | 8 16-bit pointers of addresses to return to once a lower priority call returns. Lower pointers correspond to higher priority levels. |
| `call_waiting`         | `$7F0980`    | Used by call op `$04` and `$07` to track that the current actor is busy waiting for a function call on another actor.                |

## Op descriptions

### `$00 return`

Return to the first available lower priority pointer. If the current priority level is 7, this will yield indefinitely.
An exception is when it is executed for the first time during an actor initialization function. It will end the
initialization by yielding, but move to the next op.

---

### `$02 call_actor <actor> <priority_function>`

- `actor`: the target actor index * 2
- `priority_function`
    - lower 4 bits: call priority
    - upper 4 bits: function to call

Calls a function on another actor. If the actor is dead, has function calling disabled or has script execution disabled,
the function will not be called.

---

### `$03 call_actor_wait <actor> <priority_function>`

- `actor`: the target actor index * 2
- `priority_function`
    - lower 4 bits: call priority
    - upper 4 bits: function to call

Calls a function on another actor. If the actor is dead or has script execution disabled, the function will not be
called. If the actor has function calling disabled, it will yield until that becomes enabled. If the actor is
running a function with a priority <= than the specified priority, yields until it gas finished before calling the new
one.

---

### `$04 call_actor_wait_halt <actor> <priority_function>`

- `actor`: the target actor index * 2
- `priority_function`
    - lower 4 bits: call priority
    - upper 4 bits: function to call

Calls a function on another actor. If the actor is dead or has script execution disabled, the function will not be
called. If the actor has function calling disabled, it will yield until that becomes enabled. If the actor is running a
function with a priority <= than the specified priority, yields until it has finished before calling the new one. Then
it yields until that function is complete. It will stop yielding if the actor to becomes dead or has script execution
disabled.

Waiting for the function call to complete is tracked in the `call_waiting` actor variable.

---

### `$05 call_player <player> <priority_function>`

- `player`: the target player index
- `priority_function`
    - lower 4 bits: call priority
    - upper 4 bits: function to call

Same as op `$02 call_actor`, but for a player index.

---

### `$06 call_player_wait <player> <priority_function>`

- `player`: the target player index
- `priority_function`
    - lower 4 bits: call priority
    - upper 4 bits: function to call

Same as op `$03 call_actor_wait`, but for a player index.

---

### `$07 call_player_wait_halt <player> <priority_function>`

- `player`: the target player index
- `priority_function`
    - lower 4 bits: call priority
    - upper 4 bits: function to call

Same as op `$04 call_actor_wait_halt`, but for a player index.

---

### `$87 script_set_cycle_delay <delay>`

- `delay`: the number of ticks to wait between script cycles

Sets the new `script_delay` value. The actual value set internally is one higher than the specified delay. The
`script_delay_counter` value is also immediately reset to that value.

---

### `$AD wait <cycles>`

- `cycles`: the number of script cycles to wait for

Suspends execution for the specified number of script cycles. Internally this starts a counter at 1, then increments
this every script cycle while yielding. When it reaches the number of cycles the op completes.

---

### `$AF yield`

End the current script execution cycle.

---

### `$B0 yield_forever`

End the current script execution cycle, but never advances to the next op. Essentially pauses execution forever.

---

### `$B9 wait 4`

Suspends execution for 4 script cycles. See `$AD wait`.

---

### `$BA wait 8`

Suspends execution for 8 script cycles. See `$AD wait`.

---

### `$BC wait 16`

Suspends execution for 16 script cycles. See `$AD wait`.

---

### `$BD wait 32`

Suspends execution for 32 script cycles. See `$AD wait`.
