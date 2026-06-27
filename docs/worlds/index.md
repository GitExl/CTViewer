# Worlds

There are 8 bank-local offsets to world headers stored at `$6FD00` in the ROM. The PC version stores this at address
`$FD10` in `Game/common/bankc6.bin`. The 8th world header is unused. Each world header refers to the data needed to
for that world.

| Offset | Type    | Description                     |
|--------|---------|---------------------------------|
| 0      | `u8[8]` | Layer 1/2 tile graphics indices |
| 8      | `u8[2]` | Layer 3 tile graphics indices   |
| 10     | `u8`    | Palette data index              |
| 11     | `u8`    | Palette animation data index    |
| 12     | `u8[4]` | Sprite graphics data indices    |
| 16     | `u8`    | Layer 1/2 tile assembly index   |
| 17     | `u8`    | Map tile data index             |
| 18     | `u8`    | Map tile property data index    |
| 19     | `u8`    | Map music transition data index |
| 20     | `u8`    | Layer 3 tile assembly index     |
| 21     | `u8`    | Exit & trigger data index       |
| 22     | `u8`    | World script index              |

Note that the PC version uses the world index to load the correct animated palette data, so the value from the header
is unused.

The following is a list of location where the world data is actually stored, as referenced by the world header.

| ROM address | Address type | Address count | Compressed | PC filename                                    | Description                     |
|-------------|--------------|---------------|------------|------------------------------------------------|---------------------------------|
| `$6FE20`    | 24 bit       | 42            | SNES only  | `Game/world/map_bin/cg<index>.bin`             | Layer 1, 2, 3 tile graphics     |
| `$6FF00`    | 24 bit       | 6             | SNES only  | `Game/world/Chip/Chip_<index>.dat`             | Layer 1, 2 tile assemblies      |
| `$6FF40`    | 24 bit       | 6             | Yes        | `Game/common/bankc6.bin` @ `$FF40`             | Layer 3 tile assemblies         |
| `$6FEA0`    | 24 bit       | 32            | Yes        |                                                | Palette (animation) data (SNES) |
|             |              |               | No         | `Game/world/plt_bin/plt<index>.bin`            | Palette data (PC)               |
|             |              |               | No         | `Game/world/colanim_bin/<index>_colanim.bin`   | Palette animation data (PC)     |
| `$6FF20`    | 24 bit       | 8             | SNES only  | `Game/world/Map/Map_<index>.dat`               | Map tile data                   |
| `$6FF80`    | 24 bit       | 8             | SNES only  | `Game/world/Id/Id_<index>.dat`                 | Map tile property data          |
| `$6FFA0`    | 24 bit       | 3             | SNES only  | `Game/world/SeId/SeId_<index>.dat`             | Map music transition data       |
| `$6FFE0`    | 24 bit       | 8             | SNES only  | `Game/world/esl/Event_<index>.dat`             | World script                    |
| `$6FFC0`    | 24 bit       | 8             | SNES only  | `Game/world/EventTable/EventTable_<index>.dat` | Exit & trigger data             |
| `$6FDF0`    | 24 bit       | 16            | SNES only  | See [Graphics](graphics.md)                    | Sprite graphics                 |
