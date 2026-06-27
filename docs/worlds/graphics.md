# World graphics

## Tile graphics

The layer 1, 2 and 3 chip indices refer to 42 offsets. Each offset points to 8 Kb of graphics data that is loaded into
VRAM in sequence. An offset index of `$80` indicates there is no data for that index and that slot in VRAM is skipped.

Layer 1 and 2 graphics are 4 bits per pixel, layer 3 graphics are 2 bits per pixel. The PC version always stores these
graphics as 4 bits per pixel, even for layer 3 tile chips.

The assembly data itself is 4 chips per tile, 2 bytes per chip. Of each chip:
- The first 10 bits, or `$3FF`, is the chip index.
- The next 3 bits, or `$7`, is the subpalette index.
- Bit `$2000` is the priority bit.
- Bit `$4000` horizontally flips the chip.
- Bit `$8000` vertically flips the chip.

The layer 3 assembly data is a direct representation of the resulting layer 3 tilemap from left to right, top to bottom.

## Sprite graphics

Sprite graphics to be displayed are assembled by animation data, see
[Sprite animation](../world_scripts/sprite_animation.md). The actual graphics tiles used are loaded into VRAM from the
list of sprite data from the world header. Each index points to up to 8 Kb of graphics data. The sprite assembly in the
world sprite animation data refers to these.

Below is the mapping of sprite data index to graphics data for the PC version. 4 through 15 are referenced by the world
header. The first 4 are loaded on demand by the game itself.

| Index | PC file                            | Description                 | VRAM address |
|-------|------------------------------------|-----------------------------|--------------|
| 0     | `Game/common/worldChara.bmp`       | Player characters           | `$2000`      |
| 1     | `Game/common/silbird.png`          | Epoch                       | `$2000`      |
| 2     | `Game/common/lavos.bmp`            | Lavos                       | `$0000`      |
| 3     | `Game/common/blackdream.bmp`       | Black omen                  | `$0000`      |
| 4     | `Game/world/gif/0_wobj0.bmp`       | 1000 AD ferry, vortex       | `$0000`      |
| 5     | `Game/world/gif/0_wobj1.bmp`       | 1000 AD ferry smoke         | `$1000`      |
| 6     | -                                  | Placeholder for PCs         | `$2000`      |
| 7     | `Game/world/gif/<world>_wboa.bmp`  | Epoch, year, birds, etc.    | `$3000`      |
| 8     | `Game/world/gif/1_wobj0.bmp`       | 600 AD magus caste a.o.     | `$0000`      |
| 9     | `Game/world/gif/1_wobj1.bmp`       | 600 AD sunken cave          | `$1000`      |
| 10    | `Game/world/gif/3_wobj0.bmp`       | 65M BC dactyl               | `$0000`      |
| 11    | `Game/world/gif/4_wobj0.bmp`       | 12000 BC mt. woe            | `$0000`      |
| 12    | `Game/world/gif/4_wobj1.bmp`       | 12000 BC mt. woe, blackbird | `$1000`      |
| 13    | `Game/world/gif/4_kodai_break.bmp` | Deteriorating zeal          | `$0000`?     |

The PC world characters are in a different size from the SNES version, so these cannot be used without extra work. The
PC Epoch graphics (`silbird.png`) are not currently available in their original non upscaled form.

## Palettes & palette animations

World palettes are a total of 256 colors, 16 sets of 16 colors. The first 8 are used by the tiles, the latter 8 by
sprites and are set through world scripts.

World scripts can also use extra palette data for animating palettes for sprites and tiles. Script op `$04` `pal` can
copy this data from anywhere in memory into VRAM. The extra palette data is loaded to `$7EC000` in SNES RAM. The exact
sequence is determined by the world script itself.
