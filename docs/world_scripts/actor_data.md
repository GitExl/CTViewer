# World actor data

| Offset | Width | Description                                                                                  |
|--------|-------|----------------------------------------------------------------------------------------------|
| `$00`  | `W`   | Address of the action function that runs this actor, in bank `$C20000`.                      |
| `$02`  | `B`   | Used for state tracking by `fade_*` amongst others.                                          |
| `$03`  | `W`   | Incremented every time the action function is run.                                           |
| `$05`  | `W`   | The op address to continue from when returning from a `gosub` op.                            |
| `$07`  | `BBB` | The current script op execution address.                                                     |
| `$0A`  | `B`   | A general step counter used by ops `move`, `scroll`, `wait`, `setspeed`, `scrollr`.          |
| `$0B`  | `BBB` | The address of currently displaying world sprite assembly data.                              |
| `$0E`  | `B`   | Current animation frame timer.                                                               |
| `$0F`  | `B`   | Palette and sprite priority. Bits `$F1` are set by `colofs`, bits `$4F` are set by `priset`. |
| `$10`  | `W`   | Sprite assembly source graphics tile offset                                                  |
| `$12`  | `L`   | Current sprite X pixel in 16.16 fixed point                                                  |
| `$16`  | `L`   | Current sprite Y pixel in 16.16 fixed point                                                  |
| `$1A`  | `L`   | X movement vector in 16.16 fixed point                                                       |
| `$1E`  | `L`   | Y movement vector in 16.16 fixed point                                                       |
| `$22`  | `?`   |                                                                                              |
| `$23`  | `?`   |                                                                                              |
| `$24`  | `?`   |                                                                                              |
| `$25`  | `?`   |                                                                                              |
| `$26`  | `?`   | Used by player character sprite assembly ops 0, 1 and 2.                                     |
| `$27`  | `?`   |                                                                                              |
| `$28`  | `B`   | Facing. Bit `$80` unknown.                                                                   |
| `$29`  | `?`   |                                                                                              |
| `$2A`  | `W`   | Current X tile for player characters                                                         |
| `$2C`  | `?`   |                                                                                              |
| `$2D`  | `?`   |                                                                                              |
| `$2E`  | `W`   | Current Y tile for player characters                                                         |
| `$30`  | `?`   |                                                                                              |
| `$31`  | `?`   |                                                                                              |
| `$32`  | `?`   |                                                                                              |
| `$33`  | `?`   |                                                                                              |
| `$34`  | `?`   |                                                                                              |
| `$35`  | `?`   |                                                                                              |
| `$36`  | `?`   |                                                                                              |
| `$37`  | `?`   |                                                                                              |
| `$38`  | `?`   |                                                                                              |
| `$39`  | `?`   |                                                                                              |
| `$3A`  | `?`   |                                                                                              |
| `$3B`  | `?`   |                                                                                              |
| `$3C`  | `?`   |                                                                                              |
| `$3D`  | `?`   |                                                                                              |
| `$3E`  | `?`   |                                                                                              |
| `$3F`  | `?`   |                                                                                              |
