| Decimal | Hexadecimal | Name             | Description                               |
|---------|-------------|------------------|-------------------------------------------|
| 0       | `0x00`      | `initialize`     | Clears local memory.                      |
| 1       | `0x01`      | `colofs`         | Sets current object palette.              |
| 2       | `0x02`      | `priset`         | Sets current object priority.             |
| 3       | `0x03`      | `grp`            | Unknown, unusued.                         |
| 4       | `0x04`      | `pal`            | Unknown, data copy?                       |
| 5       | `0x05`      | `mapjump`        | Changes location.                         |
| 6       | `0x06`      | `mappos`         | Unknown, unused.                          |
| 7       | `0x07`      | `putmap`         | Changes a single map tile.                |
| 8       | `0x08`      | `bind`           | Binds object to a player character.       |
| 9       | `0x09`      | `newevent`       | Creates a new object at the op address.   |
| 10      | `0x0a`      | `clr`            | Clears local byte.                        |
| 11      | `0x0b`      | `incr`           | Increments local byte.                    |
| 12      | `0x0c`      | `decr`           | Decrements local byte.                    |
| 13      | `0x0d`      | `setr`           | Sets local byte.                          |
| 14      | `0x0e`      | `bitsetr`        | Sets local bits.                          |
| 15      | `0x0f`      | `bitclr`         | Clears local bits.                        |
| 16      | `0x10`      | `memclr`         | Clears global byte.                       |
| 17      | `0x11`      | `meminc`         | Increments global byte.                   |
| 18      | `0x12`      | `memdec`         | Decrements global byte.                   |
| 19      | `0x13`      | `memset`         | Sets global byte.                         |
| 20      | `0x14`      | `membitset`      | Sets global bits.                         |
| 21      | `0x15`      | `membitclr`      | Clears global bits.                       |
| 22      | `0x16`      | `trnlg`          | Copies byte from local to global memory.  |
| 23      | `0x17`      | `trngl`          | Copies byte global to local memory.       |
| 24      | `0x18`      | `trnr`           | Copies byte from local to local memory.   |
| 25      | `0x19`      | `trnmem`         | Copies byte from global to global memory. |
| 26      | `0x1a`      | `jp`             |                                           |
| 27      | `0x1b`      | `jdjnz`          |                                           |
| 28      | `0x1c`      | `jz`             |                                           |
| 29      | `0x1d`      | `jnz`            |                                           |
| 30      | `0x1e`      | `jcpnz`          |                                           |
| 31      | `0x1f`      | `jcpz`           |                                           |
| 32      | `0x20`      | `jandnz`         |                                           |
| 33      | `0x21`      | `jandz`          |                                           |
| 34      | `0x22`      | `jz_g`           |                                           |
| 35      | `0x23`      | `jnz_g`          |                                           |
| 36      | `0x24`      | `jcpnz_g`        |                                           |
| 37      | `0x25`      | `jcpz_g`         |                                           |
| 38      | `0x26`      | `jandnz_g`       |                                           |
| 39      | `0x27`      | `jandz_g`        |                                           |
| 40      | `0x28`      | `fadeout`        |                                           |
| 41      | `0x29`      | `fadein`         |                                           |
| 42      | `0x2a`      | `mozin`          |                                           |
| 43      | `0x2b`      | `mozout`         |                                           |
| 44      | `0x2c`      | `pos`            |                                           |
| 45      | `0x2d`      | ???, skip 1 byte |                                           |
| 46      | `0x2e`      | `vecx`           |                                           |
| 47      | `0x2f`      | `vecy`           |                                           |
| 48      | `0x30`      | `anmseq`         |                                           |
| 49      | `0x31`      | `move`           |                                           |
| 50      | `0x32`      | `scroll`         |                                           |
| 51      | `0x33`      | `bganm`          |                                           |
| 52      | `0x34`      | `func`           |                                           |
| 53      | `0x35`      | `link`           |                                           |
| 54      | `0x36`      | `call`           |                                           |
| 55      | `0x37`      | `return`         |                                           |
| 56      | `0x38`      | `wait`           |                                           |
| 57      | `0x39`      | `anmwait`        |                                           |
| 58      | `0x3a`      | `timer`          |                                           |
| 59      | `0x3b`      | `effect1`        |                                           |
| 60      | `0x3c`      | `effect2`        |                                           |
| 61      | `0x3d`      | `sound`          |                                           |
| 62      | `0x3e`      | `initscreen`     |                                           |
| 63      | `0x3f`      | `tpxmove`        |                                           |
| 64      | `0x40`      | `tpymove`        |                                           |
| 65      | `0x41`      | `trigger`        |                                           |
| 66      | `0x42`      | `slink`          |                                           |
| 67      | `0x43`      | `s_newevent`     |                                           |
| 68      | `0x44`      | `wake`           |                                           |
| 69      | `0x45`      | `sleep`          |                                           |
| 70      | `0x46`      | `addr`           |                                           |
| 71      | `0x47`      | `subr`           |                                           |
| 72      | `0x48`      | `memadd`         |                                           |
| 73      | `0x49`      | `memsub`         |                                           |
| 74      | `0x4a`      | `s_sound`        |                                           |
| 75      | `0x4b`      | `musiccmd`       |                                           |
| 76      | `0x4c`      | `jcpcc`          |                                           |
| 77      | `0x4d`      | `jcpcs`          |                                           |
| 78      | `0x4e`      | `func2`          |                                           |
| 79      | `0x4f`      | `copymap`        |                                           |
| 80      | `0x50`      | `putmapr`        |                                           |
| 81      | `0x51`      | `scrollr`        |                                           |
| 82      | `0x52`      | `taskend`        |                                           |
| 83      | `0x53`      | `moveEX`         |                                           |
| 84      | `0x54`      | `palEX`          |                                           |
