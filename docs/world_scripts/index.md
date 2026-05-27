# World scripts

TODO: offset of pointers, files from PC version, general execution


## Objects

Object data in memory starts at `$000C30` and goes up to `$001B30`. There are 64 (`$40`) bytes per object for a maximum
of 60 objects.

Object data starts with a local pointer to the actual code that executes the object, the "executor function".
`$(C2)0F64` for example executes commands from the world script address assigned to it in `$07`. But an object can also
be created that takes care of fading out the screen using the executor function `$(C2)20A2` or controlling the player
character sprite using function `$(C2)3404`.

When the world is initialized, object 0 starts from byte 0 in the script.
TODO: exact starting state and steps after loading world.

For reference, sprite animation data is located at `$C3E000`. Script commands are loaded at `$7F0400`. So pointers to
ops inside the script are sometimes offset by `+$400`.


## Object data

- `$00` W address of the executor function that runs this object, in bank `$C20000`.
- `$02` B unknown, 01 or 00.
- `$03` W command timer/countdown?
- `$05` W the op address to continue from when returning from a `gosub` op.
- `$07` BBB current script op execution address.
- `$0A` B general timer used by ops `$31 move`, `$32 scroll`, `$38 wait`, `$39 setspeed`, `$51 scrollr`.
- `$0B` BBB address of currently displaying world sprite assembly data.
- `$0E` B animation timer.
- `$0F` B palette and sprite priority. Bits `$F1` are set by op `$01 colofs`, bits `$4F` are set by `$02 priset`.
- `$10`
- `$12`
- `$14` W x pixel coord
- `$16`
- `$18` W y pixel coord
- `$1A`
- `$1C` W x move vector
- `$1E`
- `$20` W y move vector
- `$22`
- `$24`
- `$26`
- `$28`
- `$2A` W x tile coord for PC
- `$2C`
- `$2E` W y tile coord for PC
- `$30`
- `$32`
- `$34`
- `$36`
- `$38`
- `$3A`
- `$3C`
- `$3E`
