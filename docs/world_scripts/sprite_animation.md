# Sprite animation

The sprites displayed on the world map are described by data from `$03E000` for the SNES version,
and `Game/common/shapeSeqTbl.bin` for the PC version. The data starts with 166 2-byte offsets for the SNES version,
and 168 offsets for the PC version. Each offset points to a series of ops for displaying an animated sprite. These
offsets are referred to by the `anmseq`, `tpxmove` and `tpymove` world script ops.

## Ops

| Dec | Hex    | Name        | Arguments                               | Description                                                                           |
|-----|--------|-------------|-----------------------------------------|---------------------------------------------------------------------------------------|
| 0   | `0x00` | `set`       | u8 value                                | Sets current actor memory `$26` to the value. Only used by PC animations.             |
| 1   | `0x01` | `inc`       |                                         | Increments current actor memory `$26` value by 1. Only used by PC animations.         |
| 2   | `0x02` | `dec`       |                                         | Decrements current actor memory `$26` value by 1. Only used by PC animations. Unused. |
| 3   | `0x03` | `goto`      | i8 offset                               | Change execution position by number of bytes.                                         |                                                                                                                                   
| 4   | `0x04` | `anm`       | u8[3] pointer, u8 duration              | A pointer to frame assembly data, and a frame duration.                               |
| 5   | `0x05` | `wait`      | u8 frames                               | Waits for a number of frames before continuing.                                       |
| 6   | `0x06` | `trncgx`    | u8[3] unknown, u16 unknown, u16 unknown | First 3 bytes go into `$60`. Next 2 bytes into `$63`. Next 2 bytes into `$65`.        |
| 7   | `0x07` | `unknown07` |                                         | Unknown. Unused.                                                                      |

## Execution

`anm` sets up the spriute data to be displayed, then waits until the frame duration has passed before advancing to the
next op. If actor memory byte `$0F` has bit `$40` set, then an `anm` op will immediately show the animation frame and
will not advance to the next op.

Animations have no real end except for a 0-duration `anm`, which will show that frame forever.

The `goto` op can loop back to an earlier point in the animation data, or even entirely different animations.

The actor memory byte at `$0E` is used for the `anm` and `wait` countdown.
