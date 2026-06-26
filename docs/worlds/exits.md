# World exits

Exits can be activated by players after they touch the exit coordinates and press A. There's two types, normal
exits that warp the player to a destination, or a scripted exit that executes ops from the world script from a
specific address.

The list starts with a byte describing the number of exits. The SNES and PC data differs only in how the destination
facing is stored separately by the PC version to make more room for destinations.

The name string is the string displayed above the xit when a player is on top of it.

If the facing byte has bit `$08` set, the exit coordinates are shifted left by 8 pixels. If the facing byte has bit
`$10` set, the exit coordinates are shifted up by 8 pixels.

## SNES data format

| Offset | Type  | Description                             |
|--------|-------|-----------------------------------------|
| 0      | `u8`  | Exit tile X, bit `$80` enables the exit |
| 1      | `u8`  | Exit tile Y                             |
| 2      | `u8`  | Name string index                       |
| 3      | `u16` | Destination scene index and facing      |
| 5      | `u8`  | Destination tile X                      |
| 6      | `u8`  | Destination tile Y                      |

## PC data format

| Offset | Type  | Description                             |
|--------|-------|-----------------------------------------|
| 0      | `u8`  | Exit tile X, bit `$80` enables the exit |
| 1      | `u8`  | Exit tile Y                             |
| 2      | `u8`  | Name string index                       |
| 3      | `u16` | Destination scene index                 |
| 5      | `u8`  | Destination facing                      |
| 6      | `u8`  | Destination tile X                      |
| 7      | `u8`  | Destination tile Y                      |

## Scripted exits

If the desintation scene index is `$1FF`, the facing byte indicates the index into the world script address list to
execute ops from when the exit is activated.
