# Sprite animation

## Animation modes

Actors can have their sprite animate in certain modes which determine how the animation loops.

- In mode 0 and 1 the animation loops indefinitely.
- In mode 2 the animation loops a specific number of times, then stops at the last frame of the animation. If the
animation mode is 0 when starting this mode, an alternate animation index value is tracked internally and the regular
animation index is set to 0xFF. Once the animation loops have completed and the animation index is set to 0xFF, the
animation mode returns to mode 0, otherwise the mode returns to mode 1.
- In mode 3 only a single static sprite frame is displayed, indefinitely.

## Updating

Animations are updated every frame, for every object. If the object is either dead or not visible, animation is
skipped. If the `anim_delay` value is larger than 0, decrement is. If it is 0, and the object is not dead, increment
`anim_index`. If the next frame is a valid one, set is as the new one and copy its delay value to `anim_delay`, then
complete the update.

If the next frame is not valid (the delay is 0, or it is beyond the number of frames available), loop back to frame 0.
If the `anim_mode` is 2 and `anim_loops_remaining` is larger than 1 (not 0), decrement it, else go back to the previous
animation frame (the last one of the animation).

In animation mode 2 the `anim_index_looped` value is used as the animation to advance instead of `anim_index`. Ops that set
mode 2 also set the `anim_index` value to `$FF`.

## Actor data

These animation related properties are available for each actor in a scene script.

| Name                   | SNES address | Description                                                                                                                              |
|------------------------|--------------|------------------------------------------------------------------------------------------------------------------------------------------|
| `anim_frame_static`    | `$7E1301`    | The sprite frame index when in static animation mode.                                                                                    |
| `anim_set_pointer`     | `$7E1580`    | Pointer to the actor's loaded animation set.                                                                                             |
| `anim_delay`           | `$7E1601`    | Tracks how many ticks the current animation frame has been visible for. Only incremented if the sprite is visible and on screen.         |
| `anim_index`           | `$7E1680`    | The current animation for infinitely looping animation ops. `$FF` = no animation is playing.                                             |
| `anim_frame`           | `$7E1681`    | The current frame of animation.                                                                                                          |
| `anim_mode`            | `$7E1780`    | The current animation mode.                                                                                                              |
| `anim_index_looped`    | `$7E1781`    | The animation index used for ops that loop a specific number of times.                                                                   |
| `anim_loops_remaining` | `$7F0B01`    | How many animation loops remain for ops that play a specific amount of them. This is one more than the amount of loops to actually play. |

## Op descriptions

### `$47 anim_limit`

The original game has a concept of limiting the amount of sprites that animate. This is used to keep performance
acceptable when there are lots of animations and actors on screen. That functionality is not implemented.

---

### `$AA anim_play_loop <animation>`

- `animation` byte: the animation index to play

Plays the animation and loops it indefinitely. Sets animation mode 1. Does not yield.

---

### `$AB anim_play_halt <animation>`

- `animation` byte: the animation index to play

Same as op `$B7 anim_play_loop_halt` but with a hardcoded loop count argument of 1.

---

### `$AC anim_set_frame <frame>`

- `frame` byte: the sprite frame to set

Directly sets the sprite frame as defined in the sprite assembly. Sets animation mode 3, so will not actually
animate further. Yields execution.

---

### `$AE anim_reset`

Resets the animation to animation 0 frame 0, unless the current animation is already 0 or the current animation mode is
something other than mode 0. Does not change the animation mode. Does not yield execution.

---

### `$B3 anim_play_loop 0`

Same as op `$AA anim_play_loop`, but with a hardcoded animation argument of 0.

---

### `$B4 anim_play_loop 1`

Same as op `$AA anim_play_loop`, but with a hardcoded animation argument of 1.

---

### `$B7 anim_play_loop_halt <animation> <loops>`

- `animation` byte: the animation index to play
- `loop_count` byte: how many loops to play the animation for.

Plays the animation once, and waits for it to finish playing. Sets animation mode 2. Internally the `loop_count`
argument is incremented before it is used. Yields when starting the animation and when waiting for it to complete.
