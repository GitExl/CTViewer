# Sprite animation

## Actor data

These animation related properties are available for each actor in a scene script.

| Name                   | SNES address | Description                                                                                                                              |
|------------------------|--------------|------------------------------------------------------------------------------------------------------------------------------------------|
| `sprite_frame`         | `$7E1301`    | The sprite frame index when in static animation mode.                                                                                    |
| `anim_set_pointer`     | `$7E1580`    | Pointer to the actor's loaded animation set.                                                                                             |
| `anim_delay`           | `$7E1601`    | Tracks how many ticks the current animation frame has been visible for. Only incremented if the sprite is visible and on screen.         |
| `anim_index`           | `$7E1680`    | The current animation for infinitely looping animation ops. `$FF` = no animation is playing.                                             |
| `anim_frame`           | `$7E1681`    | The current frame of animation.                                                                                                          |
| `anim_mode`            | `$7E1780`    | The current animation mode.                                                                                                              |
| `anim_index_loop`      | `$7E1781`    | The animation index used for ops that loop a specific number of times.                                                                   |
| `anim_loops_remaining` | `$7F0B01`    | How many animation loops remain for ops that play a specific amount of them. This is one more than the amount of loops to actually play. |

## Ops

- `$AA` Play animation `x`, loop indefinitely (mode 1)
- `$AB` Play animation, once, wait (mode 2)
- `$AC` Set sprite frame `x` (mode 3)
- `$AE` Reset animation
- `$B3` Play animation 0, loop indefinitely (mode 1)
- `$B4` Play animation 1, loop indefinitely (mode 1)
- `$B7` Play animation `x`, loop `y` times, wait (mode 2)
- `$47` Animation limiter, not implemented

## Animation modes

- In mode 0 the animation loops indefinitely.
- In mode 1 the animation loops indefinitely.
- In mode 2 the animation loops a specific number of times, then stops at the last frame of the animation.
- In mode 3 only a single sprite frame is displayed indefinitely.

## Updating

Animations are updated every frame, for every object. If the `anim_delay` value is larger than 0, decrement is. If it is
0, and the object is not dead, increment `anim_index`. If the next frame is a valid one, set is as the
new one and copy its delay value to `anim_delay`, then complete the update.

If the next frame is not valid (the delay is 0, or it is beyond the number of frames available), loop back to frame 0.
If the `anim_mode` is 2 and `anim_loops_remaining` is larger than 1 (not 0), decrement it, else go back to the previous
animation frame (the last one of the animation).

In animation mode 2 the `anim_index_loop` value is used as the animation to advance instead of `anim_idx`. Ops that set
mode 2 set the `anim_idx` value to `$FF`.

The original game has a concept of limiting the amount of sprites that animate. This is used to keep performance
acceptable when there are lots of animations and actors on screen. That functionality is not implemented.
