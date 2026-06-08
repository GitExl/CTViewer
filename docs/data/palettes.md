# Palettes

WIP

15 bit SNES RGB = 5 bits per component and one unused bit.

## Scene palettes

first 16 colors are for L3 tilesF
then 7 16 colors sets selectable by tiles
last 8 sets of 16 colors are used by sprites, loaded as needed from sprite palette data

### SNES
`$3624C0` contains x palettes. 210 bytes per palette, with 7 sets of 15 colors in each set. The first color of
each set is always assumed to be black. Contains 15 bit SNES RGB colors. Contains all tile palettes for the scene. 

### PC
`Game/field/palette_bin/plt{}.bin`, 256 colors per palette, where each color is a 15 bit SNES RGB value.

## Sprite palettes

### SNES

### PC

## World palettes

### SNES
32 24 bit pointers to palette data are stored at $6FEA0. The palette data itself starts at $3E000.

### PC

## Menu palettes

### SNES

### PC
