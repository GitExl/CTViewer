# World scripts

## Script data

The SNES version stores 8 24-bit pointers to compressed world script data at `$06FFE0`. The PC version stores
the uncompressed world scripts in `Game\world\esl\Event_{index}.dat`. The world header data refers to the world
scripts by the index into the offsets or the index of the event data file.

## World actors

Actor data in memory starts at `$000B30` and goes up to `$001B30`. There are 64 (`$40`) bytes per actor for a maximum
of 64 actors. The first 4 actors are "special", there are special functions for creating these that only find an
unused actor in the first 4 slots (from `$0B30`). Normal actor creation functions start their search for an unused
actor from the 5th slot (`$0C30`).

Actor data starts with a local pointer to the actual code that executes the actor, the "task function".
`$(C2)0F64` for example executes commands from the world script address assigned to it in `$07`. But an actor can also
be created that takes care of fading out the screen using the task function `$(C2)20A2` or controlling the player
character sprite using function `$(C2)3404`.

For reference, sprite animation data is located at `$C3E000`. Script commands are loaded at `$7F0400`. So pointers to
ops inside the script are sometimes offset by `+$400`.

## Memory

Global memory refers to the range of `$000000` to `$00FFFF`. Local memory refers to the 64 bytes that store each
actor's data. In theory local memory ops can access memory past their own, up to and including the next 3 actors.

## Execution

When the world is initialized, an actor is created that starts executing ops from byte 0 in the script. Every frame
all actors are iterated. Actors that have a non-zero action function have that function run with the actor as the
"context". Every frame an actor's task is run, the 16-bit actor memory value at `$03` is incremented.
