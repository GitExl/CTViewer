# World triggers

These are associated loctions on a world map that refer to a world script address. They are activated when the
player touches their coordinates, and the tile properties have bit `$01` set. For example the world 0 Vortex Point exit
from Heckran Cave is such a trigger, activated when the Heckran Cave exits the player at it's coordinates. Once
activated by a player, a trigger is disabled by clearing bit `$80` from the X coordinate.

The script address index refers to one of the script addresses at the end of world header data. These are addresses
into the world script ops to be run when the trigger activates.

| Offset | Type | Description                                   |
|--------|------|-----------------------------------------------|
| 0      | `u8` | Trigger tile X, bit `$80` enables the trigger |
| 1      | `u8` | Trigger tile Y                                |
| 2      | `u8` | Script address index                          |

A trigger with a 0 X and Y coordinate indicates the end of the list of triggers.
