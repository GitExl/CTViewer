# Strings

## Huffman encoded strings

The SNES version stores some text (mostly dialogue) as Huffman-encoded strings. There is a single dictionary of
substrings that contain the parts that make up all the decoded strings. The strings can refer to that table to include
a substring of it. This saves a considerable amount of space in the SNES ROM.

### Dictionary

The dictionary of substrings is stored from `$1EFA00` to `$1EFF00` and contains 128 16-bit bank-relative pointers, one
for each substring in the dictionary. Each substring can be decoded with:

- `$00` - end of substring.
- `$01` or `$02` - decode next byte as a regular character.
- `$03` - decode the next byte as a delay in ticks. A delay of 0 ends the substring.
- `>= $9F` - decode *this* byte as a regular character.
- anything else is decoded as a special character.

### Regular characters

- From `$A0` up to and including `$B9` - an ASCII character starting from `$41`.
- From `$BA` up to and including `$D3` - an ASCII character starting from `$61`.
- From `$D4` up to and including `$DD` - an ASCII character starting from `$30`.
- `$DE` - !
- `$DF` - ?
- `$E0` - /
- `$E1` - “
- `$E2` - ”
- `$E3` - :
- `$E4` - &
- `$E5` - (
- `$E6` - )
- `$E7` - '
- `$E8` - .
- `$E9` - ,
- `$EA` - =
- `$EB` - -
- `$EC` - +
- `$ED` - %
- `$EE` - ♫
- `$EF` - a space
- `$F0` - ♥
- `$F1` - …
- `$F2` - ∞
- `$F3` - #

### Special characters

Most special characters are output as one or more control codes for easier parsing. See
[Dialogue](../scene_scripts/dialogue.md) for an explanation of what these control codes do.

- `$04` - unknown purpose
- `$05` - `<BR>`
- `$06` - `<BR><INDENT>`
- `$07` - `<WAIT>03</WAIT><BR>`
- `$08` - `<WAIT>03</WAIT><BR><INDENT>` - the amount of time to wait is not yet verified
- `$09` - `<AUTO_PAGE>`
- `$0A` - `<AUTO_PAGE><INDENT>`
- `$0B` - `<PAGE>`
- `$0C` - `<PAGE><INDENT>`
- `$0D` - `<NUMBER 8>`
- `$0E` - `<NUMBER 16>`
- `$0F` - `<NUMBER 24>`
- `$10` - unknown purpose
- `$11` - should likely display the previous substring, but unverified
- `$12` - `<NAME_TEC>$xx</NAME_TEC>` displays the tech name from the next byte
- `$13` - `<NAME_CRO>`
- `$14` - `<NAME_MAR>`
- `$15` - `<NAME_LUC>`
- `$16` - `<NAME_ROB>`
- `$17` - `<NAME_FRO>`
- `$18` - `<NAME_AYL>`
- `$19` - `<NAME_MAG>`
- `$1A` - `<NICK_CRO>`
- `$1B` - `<NAME_PT1>`
- `$1C` - `<NAME_PT2>`
- `$1D` - `<NAME_PT3>`
- `$1E` - `<NAME_LEENE>`
- `$1F` - `<NAME_ITM>`
- `$20` - `<NAME_SIL>`

### Decoding a string

This works the same way as decoding a substring. So regular or special characters can appear just like in a substring,
but codes from `$21` up to and including `$9F` refer to substrings from the dictionary. Subtract `$21` to get the
substring index. Decoded strings also end on a `$00` code or a 0 tick delay.

## Mapped strings

Some strings like item names are not encoded in any special way. Preparing them for display only involves mapping their
values into the ASCII range.

|      | `00`      | `01`      | `02`      | `03`      | `04`      | `05`     | `06`       | `07`     | `08`      | `09`     | `0A`       | `0B`     | `0C`      | `0D`    | `0E`       | `0F`     |
|------|-----------|-----------|-----------|-----------|-----------|----------|------------|----------|-----------|----------|------------|----------|-----------|---------|------------|----------|
| `00` |           |           |           |           |           |          |            |          |           |          |            |          |           |         |            |          |
| `10` |           |           |           |           |           |          |            |          |           |          |            |          |           |         |            |          |
| `20` | `<BLADE>` | `<BOW>`   | `<GUN>`   | `<ARM>`   | `<SWORD>` | `<FIST>` | `<SCYTHE>` | `<HELM>` | `<ARMOR>` | `<RING>` | `<H>`      | `<M>`    | `<P>`     | `<:>`   | `<SHIELD>` | `<STAR>` |
| `30` |           |           |           |           |           |          |            |          |           |          |            |          |           |         |            |          |
| `40` |           |           |           |           |           |          |            |          |           |          |            |          |           |         |            |          |
| `50` |           |           |           |           |           |          |            |          |           |          |            | `<LEFT>` | `<RIGHT>` | (       | )          | :        |
| `60` | `<HAND1>` | `<HAND2>` | `<HAND3>` | `<HAND4>` | `<H>`     | `<M>`    | `<P>`      | `<HP0>`  | `<HP1>`   | `<HP2>`  | `<HP3>`    | `<HP4>`  | `<HP5>`   | `<HP6>` | `<HP7>`    | `<HP8>`  |
| `70` |           |           | °         | 0         | 1         | 2        | 3          | 4        | 5         | 6        | 7          | 8        | 9         | `<D>`   | `<Z>`      | `<UP>`   |
| `80` |           |           |           |           |           |          |            |          |           |          |            |          |           |         |            |          |
| `90` |           |           |           |           |           |          |            |          |           |          |            |          |           |         |            |          |
| `A0` | A         | B         | C         | D         | E         | F        | G          | H        | I         | J        | K          | L        | M         | N       | O          | P        |
| `B0` | Q         | R         | S         | T         | U         | V        | W          | X        | Y         | Z        | a          | b        | c         | d       | e          | f        |
| `C0` | g         | h         | i         | j         | k         | l        | m          | n        | o         | p        | q          | r        | s         | t       | u          | v        |
| `D0` | w         | x         | y         | z         | 0         | 1        | 2          | 3        | 4         | 5        | 6          | 7        | 8         | 9       | !          | ?        |
| `E0` | /         | “         | ”         | :         | &         | (        | )          | '        | .         | ,        | =          | -        | +         | %       | #          |          |
| `F0` | °         | `<A>`     | #         | #         | `<L>`     | `<R>`    | `<H>`      | `<M>`    | `<P>`     |          | `<CORNER>` | (        | )         |         |            |          |

