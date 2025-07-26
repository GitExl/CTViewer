# CT Viewer

A utility to display Chrono Trigger scene and world maps. It supports the North American SNES version and the Steam
version. It can display relevant tile and palette animations for the maps, as well as display some animated test
sprites.

## Usage

`ctviewer [OPTIONS] --path <PATH>`

Options:
- `-p`, `--path <PATH>`    Source data path
- `-w`, `--world <WORLD>`  Index of the world to load [default: -1]
- `-s`, `--scene <SCENE>`  Index of the scene to load [default: -1]
- `--scale <SCALE>`        Display scale [default: 4]
- `-d`, `--dump`           Dump information and debug data
- `-h`, `--help`           Print help

Examples:
- `ctviewer -p ./chrono-trigger.smc -s 144`  View Denadaro North Face
- `ctviewer -p ./chrono-trigger.smc -w 2`    View 2100 A.D.
- `ctviewer -p ./ct_steam -s 1 -d`           View Chrono's Kitchen and output debug info and bitmaps from the PC version.

The source data path can be a headered or unheadered North American SNES ROM file, or the extracted contents of the
`resources.bin` file from the Steam version. You can use the "CT Explore" utility by River Nixx (download from
https://rivernyxx.com/downloads.html) to extract the data from the Steam version.

## Keys

When viewing scenes or worlds, the following keys are available:

- `wasd` to move around
- `esc` to exit
- `1` toggle rendering of layer 1
- `2` toggle rendering of layer 2
- `3` toggle rendering of layer 3
- `4` toggle rendering of test sprites
- `5` toggle rendering of the map palette
- `\` to write a screenshot of the internal render buffer to `debug_output/screenshot.bmp`

When viewing worlds, the following keys are available:

- `z` disable debug rendering
- `x` render collision data 
- `c` render exit data
- `v` render music transition data

When viewing scenes, the following keys are available:

- `z` disable debug rendering
- `x` render collision data
- `c` render NPC collision data
- `v` render battle collision data
- `b` render Z plane data
- `n` render tile movement data
- `m` render door data
- `,` render tile layer 1/layer 2 source data
- `.` render sprite over layer 1 or layer 2 data
- `/` render exit data

## Examples

These are some screenshots from various scenes. They have been scaled to a 4/3 aspect ratio.

![Castle Magus Throne of Strength](/readme/Castle%20Magus%20Throne%20of%20Strength.png)
![Denadoro South Face](/readme/Denadoro%20South%20Face.png)
![Crono's Kitchen](/readme/Crono's%20Kitchen.png)
![Zeal Kingdom](/readme/Zeal%20Kingdom.png)

## Compiling

Install Rust from https://www.rust-lang.org/tools/install and build it like any other Cargo project by running
`cargo run -- [parameters]`.

The SDL2 libraries are required to be in the path to be able to run. Get the `SDL2.lib` and `SDL2.dll` development
libraries for your platform from https://github.com/libsdl-org/SDL/releases
