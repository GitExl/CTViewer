# Actor movement

## Actor data

These movement related properties are available for each actor in a scene script.

| Name              | SNES address | Description                                                                                                   |
|-------------------|--------------|---------------------------------------------------------------------------------------------------------------|
| `x`               | `$7E1800`    | 2 byte X coordinate of the actor. The low byte equals the current tile, the high byte the pixel from 1 to 16. |
| `y`               | `$7E1880`    | 2 byte Y coordinate of the actor. The low byte equals the current tile, the high byte the pixel from 1 to 16. |
| `x_speed`         | `$7E1900`    | The X movement speed of the actor. 16 = 1 pixel per tick.                                                     |
| `y_speed`         | `$7E1980`    | The Y movement speed of the actor. 16 = 1 pixel per tick.                                                     |
| `move_speed`      | `$7E1A00`    | The speed to move at. The X and Y movement speed are calculated from this.                                    |
| `move_length`     | `$7E1A01`    | How many ticks to keep moving.                                                                                |
| `is_moving`       | `$7E1A80`    | If set, the actor is currently moving. Used by ops that need to move in a relative way.                       |
| `solidity_flags`  | `$7E1B01`    | Flags describing how solid the actor is to others.                                                            |
| `move_dest_flags` | `$7F1C80`    | Flags describing how the actor moves onto the destination.                                                    |

## Ops

- TODO `$7A` Jump towards `x` x `y`, at height `z`
- TODO `$7B` Jump at speed `x` x `y`, with unknown `z` value, for `a` script cycles 
- TODO `$8F` Move towards actor `x`, do not change facing
- `$92` Move at angle `x` for `y` script cycles
- TODO `$94` Move towards actor `x`
- TODO `$95` Move towards party member `x`
- `$96` Move towards tile coordinates `x` x `y`
- `$97` Move towards tile coordinates `x` x `y` from local memory
- TODO `$98` Move towards actor `x` for `y` script cycles
- TODO `$99` Move towards party member `x` for `y` script cycles
- `$9A` Move towards tile coordinates `x` x `y` for `z` script cycles
- `$9C` Move at angle `x` for `y` script cycles, do not change facing
- `$9D` Move at angle `x` for `y` script cycles from local memory
- TODO `$9E` Move towards actor `x`, do not change facing
- TODO `$9F` Move towards actor `x` from local memory
- `$A0` Move towards tile coordinates `x` x `y`, do not change facing
- `$A1` Move towards tile coordinates `x` x `y` from local memory, do not change facing
- TODO `$B5` Keep moving towards actor `x`, yield forever
- TODO `$B6` Keep moving towards party member `x`, yield forever
- TODO `$D9` Move party members to positions `x` x `y`, `a` x `b` and `c` x `d`

## Move at angle

Ops that move actors at an angle will calculate that angle's movement vector, multiplied by the actor's speed. The
actor will then move at that vector for the specified number of script cycles. 

## Move towards tile coordinates

Ops that move to tile coordinates calculate the angle between the actor's current tile coordinates and the destination
tile coordinates. The angle is multiplied by the actor's speed to get the movement vector. The actor will move at
that vector until the next script cycles is run. It will then recalculate the movement vector, unless it reached the
destination tile coordinates. If the actor has bit `$01` of `move_dest_flags` set, it will then continue moving towards
the bottom center of the destination tile at 1 pixel per tick. First on the X axis, then on the Y axis.

## Solid object avoidance

If bit `$02` of `move_dest_flags` is set, actors will try to detect other solid actors during movent updates. These have
bit `$01` of `solidity_flags` set. If one is found in front of where the actor is facing, the calculated movement angle
is offset by 67.5 degrees clockwise.
