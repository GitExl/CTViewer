# World script ops

- `u8` unsigned 8-bit integer
- `i8` signed 8-bit integer
- `u16` unsigned 16-bit integer
- `[n]` array of n values
- `destination` 2 byte destination data, formatted for SNES or PC

| Dec | Hex   | Name         | Arguments                                                       | Description                                                        |
|-----|-------|--------------|-----------------------------------------------------------------|--------------------------------------------------------------------|
| 0   | `$00` | `initialize` |                                                                 | Clears local memory.                                               |
| 1   | `$01` | `colofs`     | u8 palette                                                      | Sets current actor palette.                                        |
| 2   | `$02` | `priset`     | u8 priority                                                     | Sets current actor priority.                                       |
| 3   | `$03` | `grp`        | u8[9] ?                                                         | Unknown.                                                           |
| 4   | `$04` | `pal`        | u8 address, u8 pal_index, u8 mode                               | Copies palette data into this actor's palette. 3 modes?            |
| 5   | `$05` | `mapjump`    | destination                                                     | Changes location.                                                  |
| 6   | `$06` | `mappos`     |                                                                 | Unused.                                                            |
| 7   | `$07` | `putmap`     | u8 layer, u8 x, u8 y, u8 tile                                   | Changes a single map tile.                                         |
| 8   | `$08` | `bind`       | u16 address, u8 pc                                              | Binds actor to a player character.                                 |
| 9   | `$09` | `newevent`   | u16 address, u8 unused                                          | Creates a new actor executing script ops from the address.         |
| 10  | `$0a` | `clr`        | u8 address                                                      | Clears local byte.                                                 |
| 11  | `$0b` | `incr`       | u8 address                                                      | Increments local byte.                                             |
| 12  | `$0c` | `decr`       | u8 address                                                      | Decrements local byte.                                             |
| 13  | `$0d` | `setr`       | u8 address, u8 value                                            | Sets local byte.                                                   |
| 14  | `$0e` | `bitsetr`    | u8 address, u8 bits                                             | Sets local bits.                                                   |
| 15  | `$0f` | `bitclr`     | u8 address, u8 bits                                             | Clears local bits.                                                 |
| 16  | `$10` | `memclr`     | u8 address                                                      | Clears global byte.                                                |
| 17  | `$11` | `meminc`     | u8 address                                                      | Increments global byte.                                            |
| 18  | `$12` | `memdec`     | u8 address                                                      | Decrements global byte.                                            |
| 19  | `$13` | `memset`     | u8 address, u8 value                                            | Sets global byte.                                                  |
| 20  | `$14` | `membitset`  | u8 address, u8 bits                                             | Sets global bits.                                                  |
| 21  | `$15` | `membitclr`  | u8 address, u8 bits                                             | Clears global bits.                                                |
| 22  | `$16` | `trnlg`      | u8 address, u16 address                                         | Copies byte from local to global memory.                           |
| 23  | `$17` | `trngl`      | u16 address, u16 address                                        | Copies byte global to local memory.                                |
| 24  | `$18` | `trnr`       | u8 address, u8 address                                          | Copies byte from local to local memory.                            |
| 25  | `$19` | `trnmem`     | u16 address, u16 address                                        | Copies byte from global to global memory.                          |
| 26  | `$1a` | `jp`         | u16 address                                                     | Jump to address.                                                   |
| 27  | `$1b` | `jdjnz`      | u8 address, i8 offset                                           | Decrement local byte and jump if non-zero.                         |
| 28  | `$1c` | `jz`         | u8 address, i8 offset                                           | Jump if local byte is zero.                                        |
| 29  | `$1d` | `jnz`        | u8 address, u8 value, i8 offset                                 | Jump if local byte is non-zero.                                    |
| 30  | `$1e` | `jcpnz`      | u8 address, u8 value, i8 offset                                 | Jump if local byte is not equal to value.                          |
| 31  | `$1f` | `jcpz`       | u8 address, u8 value, i8 offset                                 | Jump if local byte is equal to value.                              |
| 32  | `$20` | `jandnz`     | u8 address, u8 value, i8 offset                                 | Jump if local byte has one or more bits from value set.            |
| 33  | `$21` | `jandz`      | u8 address, u8 value, i8 offset                                 | Jump if local byte has no no bits from value set.                  |
| 34  | `$22` | `jz_g`       | u16 address, i8 offset                                          | Jump if global byte is zero.                                       |
| 35  | `$23` | `jnz_g`      | u16 address, i8 offset                                          | Jump if global byte is non-zero.                                   |
| 36  | `$24` | `jcpnz_g`    | u16 address, u8 value, i8 offset                                | Jump if global byte is not equal to value.                         |
| 37  | `$25` | `jcpz_g`     | u16 address, u8 value, i8 offset                                | Jump if global byte is equal to value.                             |
| 38  | `$26` | `jandnz_g`   | u16 address, u8 value, i8 offset                                | Jump if global byte has one or more bits from value set.           |
| 39  | `$27` | `jandz_g`    | u16 address, u8 value, i8 offset                                | Jump if global byte has no bits from value set.                    |
| 40  | `$28` | `fadeout`    | u8 mode                                                         | Fade to black.                                                     |
| 41  | `$29` | `fadein`     | u8 mode                                                         | Fade from black.                                                   |
| 42  | `$2a` | `mozin`      | u8 mode                                                         | Mosaic in                                                          |
| 43  | `$2b` | `mozout`     | u8 mode                                                         | Mosaic out                                                         |
| 44  | `$2c` | `pos`        | u16 x, u16 y                                                    | Sets sprite position.                                              |
| 45  | `$2d` |              |                                                                 | Skips 1 byte.                                                      |
| 46  | `$2e` | `vecx`       | i32 magnitude                                                   | Set X movement vector. 16.16 fixed point value.                    |
| 47  | `$2f` | `vecy`       | i32 magnitude                                                   | Set Y movement vector. 16.16 fixed point value.                    |
| 48  | `$30` | `anmseq`     | u8 animation                                                    | Set sprite animation.                                              |
| 49  | `$31` | `move`       | u8 steps                                                        | Move actor by vector components for given amount of steps.         |
| 50  | `$32` | `scroll`     | u8 steps                                                        | Scroll map by vector components for given amount of steps.         |
| 51  | `$33` | `bganm`      | u8 unknown, u16[3] unknown                                      | Animate background somehow?                                        |
| 52  | `$34` | `func`       | u16 address                                                     | Directly calls a native function, with a 16 bit address.           |
| 53  | `$35` | `link`       | u16 address                                                     | Creates a new actor with the specified action function.            |
| 54  | `$36` | `call`       | u16 address                                                     | Stores the next op address and jumps to a new address.             |
| 55  | `$37` | `return`     |                                                                 | Jumps back to the stored op address.                               |
| 56  | `$38` | `wait`       | u8 delay                                                        | Waits for a given number of cycles.                                |
| 57  | `$39` | `anmwait`    | u8 delay                                                        | Waits for a given number of cycles, then animates                  |
| 58  | `$3a` | `timer`      | u8 unknown                                                      | Waits for actor timer to reach zero? Unused.                       |
| 59  | `$3b` | `effect1`    | u8 sound, i8 panning                                            | Play sound effect with sound command `$18`?                        |
| 60  | `$3c` | `effect2`    | u8 sound, i8 panning                                            | Play sound effect with sound command `$19`?                        |
| 61  | `$3d` | `sound`      | u8 music                                                        | Play music. Does nothing if it is already playing.                 |
| 62  | `$3e` | `initscreen` | u8 layer                                                        | Some screen setup layer for the given layer.                       |
| 63  | `$3f` | `tpxmove`    | u16 steps, u8 animation1, u8 animation2                         | Move actor by X, also animates in some way.                        |
| 64  | `$40` | `tpymove`    | u16 steps, u8 animation1, u8 animation2                         | Move actor by Y, also animates in some way.                        |
| 65  | `$41` | `trigger`    |                                                                 | Unused.                                                            |
| 66  | `$42` | `slink`      | u16 address                                                     | Creates a new special actor with the specified action function.    |
| 67  | `$43` | `s_newevent` | u16 address, u8 unused                                          | Creates a new special actor executing script ops from the address. |
| 68  | `$44` | `wake`       | u16 address                                                     | Open up a (scripted) exit by address.                              |
| 69  | `$45` | `sleep`      | u16 address                                                     | Close down a (scripted) exit by address.                           |
| 70  | `$46` | `addr`       | u8 address, u8 value                                            | Adds a value to a local byte.                                      |
| 71  | `$47` | `subr`       | u8 address, u8 value                                            | Subtracts a value from a local byte.                               |
| 72  | `$48` | `memadd`     | u16 address, u8 value                                           | Adds a value to a global byte.                                     |
| 73  | `$49` | `memsub`     | u16 address, u8 value                                           | Subtracts a value from a global byte.                              |
| 74  | `$4a` | `s_sound`    | u8 music                                                        | Plays music, always interrups already playing track?               |
| 75  | `$4b` | `musiccmd`   | u8 flags1, u8 music, u8 flags2, u8 extra                        | Plays music, with additional data sent to sound command?           |
| 76  | `$4c` | `jcpcc`      | u16 address, u8 value, i8 offset                                | Jump if global byte is less than the value.                        |
| 77  | `$4d` | `jcpcs`      | u16 address, u8 value, i8 offset                                | Jump if global byte is equal to or greater than the value.         |
| 78  | `$4e` | `func2`      | u8[3] address                                                   | Directly calls a native function, with a 24 bit address.           |
| 79  | `$4f` | `copymap`    | u8 src, u8 srcx, u8 srcy, u8 dst, u8 destx, u8 dsty, u8 w, u8 h | Copy map tiles from a source layer to a destination layer.         |
| 80  | `$50` | `putmapr`    | u8 layer, u8 x, u8 y, u8 tile                                   | Changes a single map tile like `putmap`, but different somehow.    |
| 81  | `$51` | `scrollr`    | u8 layer, u8 steps                                              | Like `scroll`, but only for a single layer.                        |
| 82  | `$52` | `taskend`    |                                                                 | Terminates this actor.                                             |
| 83  | `$53` | `moveEX`     |                                                                 | PC/DS specific, unknown.                                           |
| 84  | `$54` | `palEX`      |                                                                 | PC/DS specific, unknown.                                           |
