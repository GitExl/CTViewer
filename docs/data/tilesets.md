# Tilesets

Tilesets describe all the graphics used to draw maps. Maps are drawn with 16x16 tiles. Each tile in turn is made up of
4 8x8 chips. What chips are used in each tile is described by tile assemblies.

Tilesets graphics, assemblies and chip animations are directly referenced by the data for each scene and world.

## Scene tilesets

### Chip graphics

The chips used for layer 1/2 tiles are stored in sets of 128 chips at `$2000` bytes each. What sets are loaded for a
given tileset are stored in these 8 byte tables:

- SNES: `$361C00 + tileset_index * 8`
- PC: `Game/field/BGSetTable/bgsettable_<tileset_index>.dat`

Each byte is the chip set graphics index to load. A value of `$FF` means none are loaded for that set. A copy of set
index 6 is made ande stored separately because this set contains the animated chips for the tileset. The chip graphics
themselves are referenced by:

- SNES: `$362220`, 204 pointers, 24 bits each, compressed
- PC: `Game/field/map_bin/cg<chip_set_index>.bin`

In the SNES version the layer 3 chip graphics are loaded from the same list as the regular tiles. The PC version loads
these from `Game/field/weather_bin/cg<chip_set_index>.bin`.

Layer 1/2 graphics are stored with 4 bits per pixel. Layer 3 graphics use 2 bits per pixel. 

### Tile assemblies

The assemblies for the layer 1/2 tiles are referenced by:

- SNES: `$362100`, 64 pointers, 24 bits each, compressed
- PC: `Game/field/ChipTable/ChipTable_<assembly_index>.dat`

The layer 3 assemblies are referenced by:

- SNES: `$3621C0`, 19 pointers, 24 bits each, compressed
- PC: `Game/field/ChipTable/ChipTableBg3_<assembly_index>.dat`

A layer 1/2 tile assembly describes 512 tiles. Layer 3 assemblies describe 256 tiles. Each tile has 4 chips, from top
left, top right to bottom left, bottom right.

The SNES describes each chip in 2 bytes with:

- 10 bits, `$03FF` for the chip index (256 should be added for layer 1/2 assemblies)
- 3 bits, `$1C00` for the palette index
- 1 bit, `$2000` set if the chip is drawn with priority
- 1 bit, `$4000` set if the chip should be flipped horizontally
- 1 bit, `$8000` set if the chip should be flipped vertically

The PC describes each chip in 3 bytes with:

- 10 bits, `$03FF` for the chip index
- 1 bit, `$0400` set if the chip should be flipped horizontally
- 1 bit, `$0800` set if the chip should be flipped vertically
- 4 bits, `$1C00` for the palette index
- 1 bit, byte 3, `$01` set if the chip is drawn with priority

The palette index value is multiplied by 16 for layer 1/2 assemblies to get the starting color index. It is multiplied
by 4 for layer 3 assemblies.

### Chip animations

Individual layer 1/2 chip graphics are animated in sets of 4 (from left to right in the chip graphics). The animation
description data is referenced by:

- SNES: `$3DF290`, 64 relative pointers, 16 bits each
- PC: `Game/field/BGAnime/bganimeinfo_<chip_animation_index>.dat`

Each version follows a mostly identical format. The PC version lists the number of animations in the first byte, then
the data for each animation. The SNES version just starts with the first animation. Tracking how many animations there
are in the SNES version can be done by only reading data up to the next pointer from the above pointer list.

- 1 byte, indicates the number of frames in this animation. If this is `$00` or `$80` we can also stop reading further
animations.
- 2 bytes, the chip data offset of the first of 4 chips that is animated. Convert it to a direct tileset chip index
with:
    - SNES: `(offset - 0x2000) / 16`
    - PC: `offset / 32`
- 1 byte per frame that indicates how many ticks to display the animation frame for in the upper nibble. The lower
nibble sometimes contains data, but the purpose of it is unknown.
    - `$10` = 16 ticks
    - `$20` = 12 ticks
    - `$40` = 8 ticks
    - `$80` = 4 ticks
- 2 bytes per frame for the chip data offset of the first of 4 source chips. Convert it to a direct tileset chip index
with:  
  - SNES: `(offset - 0x6000) / 32`
  - PC: `offset / 32`

An expanded version of this data (for easier DMA transfer) is loaded into SNES memory at `$7F0400` to `$7F0580`. This
is read during game execution and can be directly altered by scene scripts.

## World tilesets

### Chip graphics

The world data lists the chip graphic set indexes to load for that world. The layer 1/2 graphics are `$2000` bytes each.
Only one set of chips is loaded for layer 3 tiles. Each set contains graphics for 128 8x8 chips. 

Tileset layer 1/2/3 chip graphics are referenced by:

- SNES: `$6FE20`, 42 pointers, 24 bits each, compressed
- PC: `Game/world/map_bin/cg<chip_set_index>.bin`

### Tile assemblies

Both layer 1/2 and layer 3 tile assemblies list 512 tiles. The layer 1/2 tile assembly data is referenced by:

- SNES: `$6FF00`, 6 pointers, 24 bits each, compressed
- PC: `Game/world/Chip/Chip_<assembly_index>.dat`

The layer 3 tile assembly data is referenced by:

- SNES: `$6FF40`, 6 pointers, 24 bits each, compressed
- PC: not stored in separate files for some reason, but instead the entire SNES `$C6` data bank is present in
`Game/common/bankc6.bin`. Get the 16 bit pointer into that bank to the start of the assembly data from address
- `$FF40 + assembly_index * 3`. 

The layer 1/2/3 tile assemblies follow the same format as the scene layer 1/2 tile assemblies.

- 10 bits, `$03FF` for the chip index
- 3 bits, `$1C00` for the palette index
- 1 bit, `$2000` set if the chip is drawn with priority
- 1 bit, `$4000` set if the chip should be flipped horizontally
- 1 bit, `$8000` set if the chip should be flipped vertically

The palette index value is multiplied by 16 for layer 1/2 assemblies to get the starting color index. It is multiplied
by 4 for layer 3 assemblies.

The layer 3 assembly is actually a direct representation of how the layer 3 tilemap is built, so a separate layer 3
does not exist. Instead, the layer 3 assembly describes a 32x16 tile area.
