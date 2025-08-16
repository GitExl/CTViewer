# CT Viewer

A utility to display Chrono Trigger scene and world maps complete with debug information. It supports the North American
SNES version and the Steam version. It can display relevant tile and palette animations for the maps, as well as display
some animated test sprites.

## Usage

`ctviewer [OPTIONS] <PATH>`

Arguments:
- `<PATH>`  The source data path.

The source data path can be a headered or unheadered North American SNES ROM file, the path to the `resources.bin` file
from the Steam version, or a directory with the extracted contents of that `resources.bin` file. You can use the
"CT Explore" utility by River Nixx (download from https://rivernyxx.com/downloads.html) to extract the data from the
Steam version into a directory.

Options:
- `-w`, `--world <WORLD>`  Index of the world to load [default: -1 / none]
- `-s`, `--scene <SCENE>`  Index of the scene to load [default: -1 / none]
- `--scale <SCALE>`        Display scale [default: -1 / auto]
- `--scale-linear`         Scale output using linear scaling
- `-a`, `--aspect-ratio`   Set the output aspect ratio [default: 1.333]
- `-d`, `--dump`           Dump information and debug data to `stdout` and `/debug_output`
- `--no-vsync`             Disable vertical sync
- `-h`, `--help`           Print help

Examples:
- `ctviewer chrono-trigger.smc -s 144`: view Denadaro North Face from the SNES version.
- `ctviewer chrono-trigger.smc -w 2`: view 2100 A.D. from the SNES version.
- `ctviewer "C:\Steam\steamapps\common\Chrono Trigger\resources.bin" -s 5 -d`: view Leene Square and output debug info and bitmaps from the PC version.

## Keys

When viewing scenes or worlds, the following keys are available:

- `wasd` to move around
- `esc` to exit
- `1` toggle rendering of layer 1
- `2` toggle rendering of layer 2
- `3` toggle rendering of layer 3
- `4` toggle rendering of sprites (disabled by default, only displays some test sprites)
- `5` toggle rendering of the map palette
- `\` to write a screenshot of the internal render buffer to `debug_output/screenshot.bmp`

When viewing worlds, the following keys are available:

- `z` disable debug rendering
- `x` render collision data 
- `c` render exits
- `v` render music data

When viewing scenes, the following keys are available:

- `z` disable debug rendering
- `x` render player collision data
- `c` render NPC and battle collision data
- `v` render Z plane data and flags
- `b` render tile movement data
- `n` render door data
- `m` render sprite priority data
- `,` render exits
- `.` render treasures

Information about exits and treasure items is displayed when the mouse is over them. You can move to another scene
or world by clicking on exits.

## Examples

These are some screenshots from various scenes. They have been scaled to a 4/3 aspect ratio.

![Castle Magus Throne of Strength](/readme/Castle%20Magus%20Throne%20of%20Strength.png "Castle Magus Throne of Strength (SNES)")
![Denadoro South Face](/readme/Denadoro%20South%20Face.png "Denadoro South Face (SNES)")
![Crono's Kitchen](/readme/Crono's%20Kitchen.png "Chrono's Kitchen (SNES)")
![Zeal Kingdom](/readme/Zeal%20Kingdom.png "Zeal Kingdom world. (SNES)")
![Frog's Burrow](/readme/Frog's%20Burrow.png "Frog's Burrow with treasure contents. (PC)")
![Mountain of Woe Z Debug data](/readme/Mt%20Woe%20Debug.png "Mountain of Woe with Z debug information. (PC)")
![1000 A.D.](/readme/1000%20AD.png "1000 A.D. with exit debug information. (PC)")

## Compiling

1. Install Rust from https://www.rust-lang.org/tools/install/.
2. SDL3 libraries are required to be in the path to be able to build and run it. Get the `SDL3.lib`, `SDL3.dll`, `SDL3_ttf.lib` and `SDL3_ttf.dll` development libraries for your platform from https://github.com/libsdl-org/SDL/releases
3. Build and run a debug build using Cargo with `cargo run -- [parameters]`.
4. Build and run an optimized release build with `cargo run -r -- [parameters]`.
