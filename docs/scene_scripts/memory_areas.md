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

