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

The following are values available for each of the 64 possible actors in a scene script. The default values are set for
during scene initialization.

| Name                   | SNES address | Default value                     |
|------------------------|--------------|-----------------------------------|
| ?                      | `$7E0B00`    | `$0000`                           |
| ?                      | `$7E0B80`    | `$FFFF`                           |
| ?                      | `$7E0F80`    | `$0080`                           |
| `script_delay`         | `$7E1000`    | `$04`                             |
| `script_delay_counter` | `$7E1001`    | `$00`                             |
| ?                      | `$7E1080`    | `$0080`                           |
| `type_status`          | `$7E1100`    | `$80` or `$07` if it has a script |
| `player_index`         | `$7E1101`    | ?                                 |
| `script_ptr`           | `$7E1180`    | `$0000`                           |
| ?                      | `$7E1200`    | ?                                 |
| ?                      | `$7E1280`    | ?                                 |
| ?                      | `$7E1300`    | ?                                 |
| `sprite_frame`         | `$7E1301`    | ?                                 |
| ?                      | `$7E1380`    | ?                                 |
| ?                      | `$7E1400`    | ?                                 |
| ?                      | `$7E1480`    | ?                                 |
| ?                      | `$7E1500`    | ?                                 |
| `anim_set_pointer`     | `$7E1580`    | ?                                 |
| ?                      | `$7E1600`    | ?                                 |
| `anim_delay`           | `$7E1601`    | ?                                 |
| `anim_index`           | `$7E1680`    | ?                                 |
| `anim_frame`           | `$7E1681`    | ?                                 |
| ?                      | `$7E1700`    | ?                                 |
| `anim_mode`            | `$7E1780`    | `$00`                             |
| `anim_index_loop`      | `$7E1781`    | `$00`                             |
| `x`                    | `$7E1800`    | `$00FF`                           |
| `y`                    | `$7E1880`    | `$00FF`                           |
| ?                      | `$7E1900`    | ?                                 |
| ?                      | `$7E1980`    | ?                                 |
| `move_speed`           | `$7E1A00`    | `$0010`                           |
| `is_moving`            | `$7E1A80`    | `$00`                             |
| ?                      | `$7E1A81`    | `$00`                             |
| ?                      | `$7E1B00`    | ?                                 |
| `solidity_flags`       | `$7E1B01`    | `$00`                             |
| ?                      | `$7E1B80`    | `$00`                             |
| ?                      | `$7E1B81`    | ?                                 |
| `current_priority`     | `$7E1C00`    | `$07`                             |
| `calls_disabled`       | `$7E1C01`    | `$00`                             |
| ?                      | `$7E1C80`    | ?                                 |
| `priority_ptrs`        | `$7F0580`    | `$0000`                           |
| `anim_loops_remaining` | `$7F0B01`    | ?                                 |
| `move_dest_flags`      | `$7F1C80`    | `$0003`                           |
