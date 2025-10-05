# Memory areas

## Referenced by ops

Scripts can reference several areas of memory (though some ops can address all 24 bits of memory). In the SNES version
these are part of RAM in the `$7E0000` or `$7F0000` banks. These areas can be divided into the following:

| Name      | Address range         | Description                                                                                                                     |
|-----------|-----------------------|---------------------------------------------------------------------------------------------------------------------------------|
| Temporary | `$7E0000` - `$7E0200` | 512 bytes starting at `$7E0000`. Scripts use these as temporary values, often for math.                                         |
| Global    | `$7F0000` - `$7F0200` | 512 bytes starting at `$7F0000`. Scripts use these mostly for "global" variables that are shared between all scenes and worlds. |
| Local     | `$7F0200` - `$7F0400` | 512 bytes starting at `$7F0200`. Scripts use these mostly for variables "local" to the current scene.                           |
| Upper     | `$7F0000` - `$800000` | The entire upper bank of SNES RAM. Usually accessed using a 16-bit value.                                                       |
| Extended  |                       | 512 bytes that are only used by the PC version. This has no SNES equivalent address range.                                      |

## Actor data

The following are values available for each of the 64 possible actors in a scene script. The default values are set
during scene initialization.

| Name                   | SNES address          | Purpose                          | Default value                     |
|------------------------|-----------------------|----------------------------------|-----------------------------------|
| ?                      | `$7E0A00`             | Screen X coordinate              |                                   |
| ?                      | `$7E0A80`             | Screen Y coordinate              |                                   |
| ?                      | `$7E0B00`             |                                  | `$00`                             |
| ?                      | `$7E0B01`             |                                  | `$00`                             |
| ?                      | `$7E0B80`             |                                  | `$FFFF`                           |
| ?                      | `$7E0C00`             | Bottom sprite priority           | `$20`                             |
| ?                      | `$7E0C01`             | Top sprite priority              | `$20`                             |
| ?                      | `$7E0C80`             |                                  |                                   |
| ?                      | `$7E0D00`             |                                  |                                   |
| ?                      | `$7E0D80`             |                                  |                                   |
| ?                      | `$7E0E00`             |                                  |                                   |
| ?                      | `$7E0E80`             |                                  |                                   |
| ?                      | `$7E0F00`             | Actor is on screen & visible     |                                   |
| ?                      | `$7E0F01`             | Current sprite frame             |                                   |
| ?                      | `$7E0F80`             | Sprite priority info             | `$80`                             |
| ?                      | `$7E0F81`             | Palette index                    | `$00`                             |
| `script_delay`         | `$7E1000`             | Script cycle time                | `$04`                             |
| `script_delay_counter` | `$7E1001`             | Script cycle delay               | `$00`                             |
| ?                      | `$7E1080`             | Script is ready to run           | `$80`                             |
| ?                      | `$7E1081`             |                                  | `$00`                             |
| `type_status`          | `$7E1100`             | Actor type                       | `$80` or `$07` if it has a script |
| `player_index`         | `$7E1101`             | Index of player owner            | ?                                 |
| `current_address`      | `$7E1180`             | Pointer to next op               | `$0000`                           |
| ?                      | `$7E1200`             | Sprite graphics bank?            | ?                                 |
| ?                      | `$7E1280`             | Sprite graphics pointer?         | ?                                 |
| ?                      | `$7E1300`             |                                  | ?                                 |
| `sprite_frame`         | `$7E1301`             | Static sprite frame              | ?                                 |
| ?                      | `$7E1380`             | Sprite info pointer?             |                                   |
| ?                      | `$7E1400`             | Palette pointers                 |                                   |
| ?                      | `$7E1480`             |                                  | ?                                 |
| ?                      | `$7E1500`             |                                  | ?                                 |
| `anim_set_pointer`     | `$7E1580`             | Animation set pointer            | ?                                 |
| ?                      | `$7E1600`             | Actor facing                     | ?                                 |
| `anim_delay`           | `$7E1601`             | Animation frame delay            | ?                                 |
| `anim_index`           | `$7E1680`             | Current animation                | ?                                 |
| `anim_frame`           | `$7E1681`             | Current animation frame          | ?                                 |
| ?                      | `$7E1700`             |                                  |                                   |
| `anim_mode`            | `$7E1780`             | Animation mode                   | `$00`                             |
| `anim_index_loop`      | `$7E1781`             | Current loop tracked animation   | `$00`                             |
| `x`                    | `$7E1800`             | X position                       | `$00FF`                           |
| `y`                    | `$7E1880`             | Y position                       | `$00FF`                           |
| ?                      | `$7E1900`             | X movement speed                 |                                   |
| ?                      | `$7E1980`             | Y movement speed                 |                                   |
| `move_speed`           | `$7E1A00`             | Movement speed                   | `$0010`                           |
| `is_moving`            | `$7E1A80`             | Is currently moving?             | `$00`                             |
| ?                      | `$7E1A81`             | Actor drawing mode               | `$01`                             |
| ?                      | `$7E1B00`             |                                  | ?                                 |
| `solidity_flags`       | `$7E1B01`             | Collision flags                  | `$00`                             |
| ?                      | `$7E1B80`             | Actor is jumping                 | `$00`                             |
| ?                      | `$7E1B81`             |                                  | `$00`                             |
| `current_priority`     | `$7E1C00`             | Current script priority          | `$07`                             |
| `calls_disabled`       | `$7E1C01`             | Cannot have functions called     | `$00`                             |
| `move_flags`           | `$7E1C80`             | Movement flags                   | `$03`                             |
| `move_dest_flags`      | `$7E1C81`             | Movement destination flags       | `$00`                             |
| `priority_ptrs`        | `$7F0580` - `$7F0900` | Script priority pointers         | `$0000`                           |
| `call_busy`            | `$7F0980`             | Waiting for a call to finish     |                                   |
| ?                      | `$7F0A00`             | Text dialog string               |                                   |
| ?                      | `$7F0A80`             | Text dialog choice result        |                                   |
| ?                      | `$7F0B00`             |                                  |                                   |
| `anim_loops_remaining` | `$7F0B01`             | Number of animation loops left   |                                   |
| ?                      | `$7F0B80`             | Related to op 88, palette cycle? | ?                                 |
