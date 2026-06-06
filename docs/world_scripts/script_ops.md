# World script ops

- `u8` unsigned 8-bit integer
- `i8` signed 8-bit integer
- `u16` unsigned 16-bit integer
- `[n]` array of n values
- `destination` 2 byte destination data, formatted for SNES or PC

The I column indicates if an implementation of the op is complete. / is a partial implementation, X a full.

| Dec | Hex   | I | Name         | Arguments                                              | Description                                                                          |
|-----|-------|---|--------------|--------------------------------------------------------|--------------------------------------------------------------------------------------|
| 0   | `$00` |   | `initialize` |                                                        | Clears local memory.                                                                 |
| 1   | `$01` | / | `colofs`     | u8 palette                                             | Sets current actor palette.                                                          |
| 2   | `$02` | / | `priset`     | u8 priority                                            | Sets current actor priority.                                                         |
| 3   | `$03` |   | `grp`        | u8[9] ?                                                | Unknown.                                                                             |
| 4   | `$04` | / | `pal`        | u8 address, u8 pal_index, u8 mode                      | Copies palette data into this actor's palette. 3 modes?                              |
| 5   | `$05` |   | `mapjump`    | destination                                            | Changes location.                                                                    |
| 6   | `$06` |   | `mappos`     |                                                        | Unused.                                                                              |
| 7   | `$07` | X | `putmap`     | u8 layer, u8 x, u8 y, u8 tile                          | Changes a single map tile.                                                           |
| 8   | `$08` |   | `bind`       | u16 address, u8 pc                                     | Binds actor to a player character.                                                   |
| 9   | `$09` | X | `newevent`   | u16 address, u8 unused                                 | Creates a new actor executing script ops from the address.                           |
| 10  | `$0a` | X | `clr`        | u8 address                                             | Clears local byte.                                                                   |
| 11  | `$0b` | X | `incr`       | u8 address                                             | Increments local byte.                                                               |
| 12  | `$0c` | X | `decr`       | u8 address                                             | Decrements local byte.                                                               |
| 13  | `$0d` | X | `setr`       | u8 address, u8 value                                   | Sets local byte.                                                                     |
| 14  | `$0e` | X | `bitsetr`    | u8 address, u8 bits                                    | Sets local bits.                                                                     |
| 15  | `$0f` | X | `bitclr`     | u8 address, u8 bits                                    | Clears local bits.                                                                   |
| 16  | `$10` | X | `memclr`     | u8 address                                             | Clears global byte.                                                                  |
| 17  | `$11` | X | `meminc`     | u8 address                                             | Increments global byte.                                                              |
| 18  | `$12` | X | `memdec`     | u8 address                                             | Decrements global byte.                                                              |
| 19  | `$13` | X | `memset`     | u8 address, u8 value                                   | Sets global byte.                                                                    |
| 20  | `$14` | X | `membitset`  | u8 address, u8 bits                                    | Sets global bits.                                                                    |
| 21  | `$15` | X | `membitclr`  | u8 address, u8 bits                                    | Clears global bits.                                                                  |
| 22  | `$16` | X | `trnlg`      | u8 address, u16 address                                | Copies byte from local to global memory.                                             |
| 23  | `$17` | X | `trngl`      | u16 address, u16 address                               | Copies byte global to local memory.                                                  |
| 24  | `$18` | X | `trnr`       | u8 address, u8 address                                 | Copies byte from local to local memory.                                              |
| 25  | `$19` | X | `trnmem`     | u16 address, u16 address                               | Copies byte from global to global memory.                                            |
| 26  | `$1a` | X | `jp`         | u16 address                                            | Jump to address.                                                                     |
| 27  | `$1b` | X | `jdjnz`      | u8 address, i8 offset                                  | Decrement local byte and jump if non-zero.                                           |
| 28  | `$1c` | X | `jz`         | u8 address, i8 offset                                  | Jump if local byte is zero.                                                          |
| 29  | `$1d` | X | `jnz`        | u8 address, u8 value, i8 offset                        | Jump if local byte is non-zero.                                                      |
| 30  | `$1e` | X | `jcpnz`      | u8 address, u8 value, i8 offset                        | Jump if local byte is not equal to value.                                            |
| 31  | `$1f` | X | `jcpz`       | u8 address, u8 value, i8 offset                        | Jump if local byte is equal to value.                                                |
| 32  | `$20` | X | `jandnz`     | u8 address, u8 value, i8 offset                        | Jump if local byte has one or more bits from value set.                              |
| 33  | `$21` | X | `jandz`      | u8 address, u8 value, i8 offset                        | Jump if local byte has no no bits from value set.                                    |
| 34  | `$22` | X | `jz_g`       | u16 address, i8 offset                                 | Jump if global byte is zero.                                                         |
| 35  | `$23` | X | `jnz_g`      | u16 address, i8 offset                                 | Jump if global byte is non-zero.                                                     |
| 36  | `$24` | X | `jcpnz_g`    | u16 address, u8 value, i8 offset                       | Jump if global byte is not equal to value.                                           |
| 37  | `$25` | X | `jcpz_g`     | u16 address, u8 value, i8 offset                       | Jump if global byte is equal to value.                                               |
| 38  | `$26` | X | `jandnz_g`   | u16 address, u8 value, i8 offset                       | Jump if global byte has one or more bits from value set.                             |
| 39  | `$27` | X | `jandz_g`    | u16 address, u8 value, i8 offset                       | Jump if global byte has no bits from value set.                                      |
| 40  | `$28` |   | `fadeout`    | u8 mode                                                | Fade to black.                                                                       |
| 41  | `$29` |   | `fadein`     | u8 mode                                                | Fade from black.                                                                     |
| 42  | `$2a` |   | `mozin`      | u8 mode                                                | Mosaic in                                                                            |
| 43  | `$2b` |   | `mozout`     | u8 mode                                                | Mosaic out                                                                           |
| 44  | `$2c` | X | `pos`        | u16 x, u16 y                                           | Sets sprite position.                                                                |
| 45  | `$2d` |   |              |                                                        | Skips 1 byte.                                                                        |
| 46  | `$2e` | X | `vecx`       | i32 magnitude                                          | Set X movement vector. 16.16 fixed point value.                                      |
| 47  | `$2f` | X | `vecy`       | i32 magnitude                                          | Set Y movement vector. 16.16 fixed point value.                                      |
| 48  | `$30` |   | `anmseq`     | u8 animation                                           | Set sprite animation.                                                                |
| 49  | `$31` | X | `move`       | u8 steps                                               | Move actor by vector components for given amount of cycles. Animates on every cycle. |
| 50  | `$32` | X | `scroll`     | u8 steps                                               | Scroll map by vector components for given amount of cycles.                          |
| 51  | `$33` |   | `bganm`      | u8 unknown, u16[3] unknown                             | Sets up DMA copy, to animate background tiles?                                       |
| 52  | `$34` | X | `func`       | u16 address                                            | Directly calls a native function, with a 16 bit address.                             |
| 53  | `$35` | X | `link`       | u16 address                                            | Creates a new actor with the specified action function.                              |
| 54  | `$36` | X | `call`       | u16 address                                            | Stores the next op address and jumps to a new address.                               |
| 55  | `$37` | X | `return`     |                                                        | Jumps back to the stored op address.                                                 |
| 56  | `$38` | X | `wait`       | u8 delay                                               | Waits for a given number of cycles.                                                  |
| 57  | `$39` |   | `anmwait`    | u8 delay                                               | Waits for a given number of cycles. Animates on every cycle.                         |
| 58  | `$3a` |   | `timer`      | u8 unknown                                             | Waits for actor timer to reach zero? Unused.                                         |
| 59  | `$3b` |   | `effect1`    | u8 sound, i8 panning                                   | Play sound effect with sound command `$18`?                                          |
| 60  | `$3c` |   | `effect2`    | u8 sound, i8 panning                                   | Play sound effect with sound command `$19`?                                          |
| 61  | `$3d` |   | `sound`      | u8 music                                               | Play music. Does nothing if it is already playing.                                   |
| 62  | `$3e` |   | `initscreen` | u8 layer                                               | Prepares tiles for the given camera position for the specified layer?                |
| 63  | `$3f` |   | `tpxmove`    | u16 x, u8 anim_l, u8 anim_r                            | Move actor to X, animate for moving left or right.                                   |
| 64  | `$40` |   | `tpymove`    | u16 y, u8 anim_u, u8 anim_d                            | Move actor to Y, animate for moving up or down.                                      |
| 65  | `$41` |   | `trigger`    |                                                        | Unused.                                                                              |
| 66  | `$42` | X | `slink`      | u16 address                                            | Creates a new special actor with the specified action function.                      |
| 67  | `$43` | X | `s_newevent` | u16 address, u8 unused                                 | Creates a new special actor executing script ops from the address.                   |
| 68  | `$44` |   | `wake`       | u16 address                                            | Open up a (scripted) exit by address.                                                |
| 69  | `$45` |   | `sleep`      | u16 address                                            | Close down a (scripted) exit by address.                                             |
| 70  | `$46` | X | `addr`       | u8 address, u8 value                                   | Adds a value to a local byte.                                                        |
| 71  | `$47` | X | `subr`       | u8 address, u8 value                                   | Subtracts a value from a local byte.                                                 |
| 72  | `$48` | X | `memadd`     | u16 address, u8 value                                  | Adds a value to a global byte.                                                       |
| 73  | `$49` | X | `memsub`     | u16 address, u8 value                                  | Subtracts a value from a global byte.                                                |
| 74  | `$4a` |   | `s_sound`    | u8 music                                               | Plays music, always interrups already playing track?                                 |
| 75  | `$4b` |   | `musiccmd`   | u8 flags1, u8 music, u8 flags2, u8 extra               | Plays music, with additional data sent to sound command?                             |
| 76  | `$4c` | X | `jcpcc`      | u16 address, u8 value, i8 offset                       | Jump if global byte is less than the value.                                          |
| 77  | `$4d` | X | `jcpcs`      | u16 address, u8 value, i8 offset                       | Jump if global byte is equal to or greater than the value.                           |
| 78  | `$4e` |   | `func2`      | u8[3] address                                          | Directly calls a native function, with a 24 bit address.                             |
| 79  | `$4f` | X | `copymap`    | u8 src, u8[2] srcpos, u8 dst, u8[2] dstpos, u8[2] size | Copy map tiles from a source layer to a destination layer.                           |
| 80  | `$50` |   | `putmapr`    | u8 layer, u8 x, u8 y, u8 tile                          | Changes a single map tile like `putmap`, but different somehow.                      |
| 81  | `$51` | X | `scrollr`    | u8 layer, u8 steps                                     | Like `scroll`, but only scrolls a single layer.                                      |
| 82  | `$52` | X | `taskend`    |                                                        | Terminates the current actor.                                                        |
| 83  | `$53` |   | `moveEX`     | ?                                                      | PC/DS specific, unknown.                                                             |
| 84  | `$54` |   | `palEX`      | ?                                                      | PC/DS specific, unknown.                                                             |
