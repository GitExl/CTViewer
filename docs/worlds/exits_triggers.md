# World exit and trigger data

This data stores 4 blocks of data, each starting with a byte describing the number of items.

- Exits: warps players to a destination
- Triggers: runs a script from a preset address
- Unknown: a still unknown type of location on the world map, that can be triggered by walking onto it
- Script addresses: a list of addresses into the world script. These can be referred to by index by exits and triggers.  

## Exits

Exits can be activated by players after they touch the exit coordinates and press A. The tile that the exit is on must
be marked to contain an exit. There's two types of exits, normal exits that warp the player to a destination and a
scripted exit that starts op execution from somewhere in the world script.

- The SNES and PC data differs only in how the destination facing is stored separately by the PC version to make more
room for destinations.
- The name string is the string displayed above the exit when a player is on top of it.
- If the facing byte has bit `$08` set, the exit coordinates are shifted left by 8 pixels. If the facing byte has bit
`$10` set, the exit coordinates are shifted up by 8 pixels.

### SNES data format

| Offset | Type  | Description                             |
|--------|-------|-----------------------------------------|
| 0      | `u8`  | Exit tile X, bit `$80` enables the exit |
| 1      | `u8`  | Exit tile Y                             |
| 2      | `u8`  | Name string index                       |
| 3      | `u16` | Destination scene index and facing      |
| 5      | `u8`  | Destination tile X                      |
| 6      | `u8`  | Destination tile Y                      |

### PC data format

| Offset | Type  | Description                             |
|--------|-------|-----------------------------------------|
| 0      | `u8`  | Exit tile X, bit `$80` enables the exit |
| 1      | `u8`  | Exit tile Y                             |
| 2      | `u8`  | Name string index                       |
| 3      | `u16` | Destination scene index                 |
| 5      | `u8`  | Destination facing                      |
| 6      | `u8`  | Destination tile X                      |
| 7      | `u8`  | Destination tile Y                      |

### Scripted exits

If the destination scene index is `$1FF`, the facing byte indicates the index into the world script address list to
execute ops from when the exit is activated.

## Triggers

These are associated loctions on a world map that refer to a world script address. They are activated when the
player touches their coordinates, and the tile properties are marked to contain an exit. For example the world 0
Vortex Point exit from Heckran Cave is such a trigger, activated when the Heckran Cave exits the player at it's
coordinates. Once activated by a player, a trigger is disabled by clearing bit `$80` from the X coordinate.

The script address index refers to one of the script addresses at the end of world header data. These are addresses
into the world script ops to be run when the trigger activates.

| Offset | Type | Description                                   |
|--------|------|-----------------------------------------------|
| 0      | `u8` | Trigger tile X, bit `$80` enables the trigger |
| 1      | `u8` | Trigger tile Y                                |
| 2      | `u8` | Script address index                          |

A trigger with a 0 X and Y coordinate indicates the end of the list of triggers.

## Unknown exits

There is room for storing a third type of "exit" with the same data format as triggers. These are activated in the same
way, but so far their purpose is unknown. These might be vestigial and are not used by any worlds.

## Script addresses

A list of 2 byte addresses into the world script. All 3 exit types can refer to these by index.
