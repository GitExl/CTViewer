# World strings

The exit names visible for the overworld exits are read from 106 strings at `$6F400` in the SNES ROM, stored as
Huffman encoded data. The PC version reads these directly from `Localize/<language>/msg/w_map.txt`. These are referred
to from regular exit data. The game performs control code subsitution on these to insert things like player character
names.

Each of the 7 worlds also have a full name. The SNES ROM stores these in Huffman encoded form at `$6F4D4`. The PC
version reads these from the last 6 entries from `Localize/<language>/msg/w_map.txt`.
