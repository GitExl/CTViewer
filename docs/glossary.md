# Glossary

| Name            | Temporal Flux name | Description                                                                                                                                      |
|-----------------|--------------------|--------------------------------------------------------------------------------------------------------------------------------------------------|
| Actor           | Object             | Something present in a scene that can move, run script ops, show a sprite, etc.                                                                  |
| Chip            | Subtile            | A quadrant of a tile. Usually 8x8 pixels.                                                                                                        |
| Backend         |                    | A source of game data. Currently there are backends for the US SNES and the PC version of Chrono Trigger.                                        |
| Bitmap          |                    | Paletted graphics data. 8 bits per pixel or less.                                                                                                |
| Destination     |                    | Information about where to warp the player party to, such as a scene or world, and what coordinates.                                             |
| Map             |                    | A grid of tiles that define the appearance and interactivity of a scene or world.                                                                |
| Op              | Command            | A single instruction that is part of a script.                                                                                                   |
| Scene           | Location           | A combination of a map, palette, script, music that forms a single playable area.                                                                |
| Scene script    | Location events    | A script containing all the actors and their behaviour for a given scene.                                                                        |
| Script cycle    |                    | A single execution of an actor's script. A maximum number of ops are executed, unless an op yields execution.                                    |
| Sprite          |                    | A graphic that can be placed in a map, and possibly animate.                                                                                     |
| Sprite assembly |                    | Instructions on how to assemble multiple pieces of graphics into sprite frames.                                                                  |
| Tick            |                    | A single step in the game. Runs 60 times per second. The SNES runs at 60.098 Hz so 60 times per second is close enough.                          |
| Tile            |                    | A square piece of a map that describes what it looks like and how it interacts with actors. Usually 16x16 pixels and made up out of 4 8x8 chips. |
| Tileset         |                    | Graphics and metadata for all the possible tiles used on a map.                                                                                  |
| World           | Overworld          | A traversable combination of a map, palette, script and music that connects multiple scenes to each other.                                       |
