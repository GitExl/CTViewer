# World map data

## Tiles

World maps are 96x64 tiles in size, where each tile is assembled from 4 tile chips from the tile assembly data. They
have 3 layers, where the 3rd layer is used for the clouds or other "weather" effects. Layer 3 tiles are not present
in the map data, but are instead transposed directly from the layer 3 tile assembly tiles.

## Tile properties

The properties of each tile are determined by looking at the layer 2 tile index, and looking up the properties for it
in the property data. Each tile has 2 bytes of data, with one nibble for each tile chip. Valid values for these
properties are:

| Value | Description                        |
|-------|------------------------------------|
| 0     | Nothing                            |
| 1     | Blocks walking                     |
| 2     | Blocks landing by Epoch/dactyl     |
| 3     | Blocks flying over by Epoch/dactyl |
| 4     | Has an exit or trigger             |

Tile chips without property value 4 will not detect an exit or trigger at that chip's location.

## Music transitions

Data for music transitions is stored in blocks of 8x16 pixels. Each byte determines what music track is
played when the player steps on that block. The exact music track played is stored in a list of music indexes at
`$1B9B` to `$1BA2` in memory. This list is initialized by world scripts.
