# World scripts

## Script data

The SNES version stores 8 24-bit pointers to compressed world script data at `$06FFE0`. The PC version stores
the uncompressed world scripts in `Game\world\esl\Event_{index}.dat`. The world header data refers to the world
scripts by the index into the offsets or the index of the event data file.

## World objects

Object data in memory starts at `$000B30` and goes up to `$001B30`. There are 64 (`$40`) bytes per object for a maximum
of 64 objects. The first 4 objects are "special", there are special functions for creating these that only find an
unused object in the first 4 slots (from `$0B30`). Normal object creation functions start their search for an unused
object from the 5th slot (`$0C30`).

Object data starts with a local pointer to the actual code that executes the object, the "action function".
`$(C2)0F64` for example executes commands from the world script address assigned to it in `$07`. But an object can also
be created that takes care of fading out the screen using the action function `$(C2)20A2` or controlling the player
character sprite using function `$(C2)3404`.

For reference, sprite animation data is located at `$C3E000`. Script commands are loaded at `$7F0400`. So pointers to
ops inside the script are sometimes offset by `+$400`.

## Execution

When the world is initialized, an object is created that starts executing ops from byte 0 in the script. Every frame
all objects are iterated. Objects that have a non-zero action function have that function run with the object as the
"context".

## Memory

Global memory refers to the range of `$000000` to `$00FFFF`. Local memory refers to the 64 bytes that stores each
object's data. In theory local memory ops can access memory past their own, up to and including the next 3 objects.

## World object data

| Offset | Width | Description                                                                                  |
|--------|-------|----------------------------------------------------------------------------------------------|
| `$00`  | `W`   | Address of the action function that runs this object, in bank `$C20000`.                     |
| `$02`  | `B`   | Unknown, either 01 or 00. Fade in/out related?                                               |
| `$03`  | `W`   | Op timer/countdown? Used by `timer`.                                                         |
| `$05`  | `W`   | The op address to continue from when returning from a `gosub` op.                            |
| `$07`  | `BBB` | The current script op execution address.                                                     |
| `$0A`  | `B`   | A general step counter used by ops `move`, `scroll`, `wait`, `setspeed`, `scrollr`.          |
| `$0B`  | `BBB` | The address of currently displaying world sprite assembly data.                              |
| `$0E`  | `B`   | Current animation frame timer.                                                               |
| `$0F`  | `B`   | Palette and sprite priority. Bits `$F1` are set by `colofs`, bits `$4F` are set by `priset`. |
| `$10`  | `?`   |                                                                                              |
| `$11`  | `?`   |                                                                                              |
| `$12`  | `?`   |                                                                                              |
| `$13`  | `?`   |                                                                                              |
| `$14`  | `W`   | Current sprite X pixel                                                                       |
| `$16`  | `?`   |                                                                                              |
| `$17`  | `?`   |                                                                                              |
| `$18`  | `W`   | Current sprite Y pixel                                                                       |
| `$1A`  | `W`   | X scroll vector?                                                                             |
| `$1C`  | `W`   | X move vector                                                                                |
| `$1E`  | `W`   | Y scroll vector?                                                                             |
| `$20`  | `W`   | Y move vector                                                                                |
| `$22`  | `?`   |                                                                                              |
| `$23`  | `?`   |                                                                                              |
| `$24`  | `?`   |                                                                                              |
| `$25`  | `?`   |                                                                                              |
| `$26`  | `?`   |                                                                                              |
| `$27`  | `?`   |                                                                                              |
| `$28`  | `?`   |                                                                                              |
| `$29`  | `?`   |                                                                                              |
| `$2A`  | `W`   | Current X tile for player characters                                                         |
| `$2C`  | `?`   |                                                                                              |
| `$2D`  | `?`   |                                                                                              |
| `$2E`  | `W`   | Current Y tile for player characters                                                         |
| `$30`  | `?`   |                                                                                              |
| `$31`  | `?`   |                                                                                              |
| `$32`  | `?`   |                                                                                              |
| `$33`  | `?`   |                                                                                              |
| `$34`  | `?`   |                                                                                              |
| `$35`  | `?`   |                                                                                              |
| `$36`  | `?`   |                                                                                              |
| `$37`  | `?`   |                                                                                              |
| `$38`  | `?`   |                                                                                              |
| `$39`  | `?`   |                                                                                              |
| `$3A`  | `?`   |                                                                                              |
| `$3B`  | `?`   |                                                                                              |
| `$3C`  | `?`   |                                                                                              |
| `$3D`  | `?`   |                                                                                              |
| `$3E`  | `?`   |                                                                                              |
| `$3F`  | `?`   |                                                                                              |
