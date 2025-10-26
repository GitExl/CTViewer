# Dialogue

All text output into the textbox at the top or bottom of the screen is considered to be dialogue.

## String tables

The strings for all dialogue are stored in string tables. Any strings referenced by a textbox op use a string from the
currently set table.

In the SNES version string tables are spread throughout the ROM and are referenced by their full 24-bit address. They
start with up to 256 segment-relative pointers to the start of each string. Each string itself is compressed as
described in [Strings](../data/strings.md). The PC version only refers to string tables by an index. The mapping to
files in the `Localize/<language>/msg` directory is as follows:

0. `cmes0.txt`
1. `cmes1.txt`
2. `cmes2.txt`
3. `cmes3.txt`
4. `cmes4.txt`
5. `cmes5.txt`
6. `kmes0.txt`
7. `kmes1.txt`
8. `kmes2.txt`
9. `mesi0.txt`
10. `mesk0.txt`
11. `mesk1.txt`
12. `mesk2.txt`
13. `mesk3.txt`
14. `mesk4.txt`
15. `mess0.txt`
16. `mest0.txt`
17. `mest1.txt`
18. `mest2.txt`
19. `mest3.txt`
20. `mest4.txt`
21. `mest5.txt`
22. `msg01.txt`
23. `msg02.txt`
24. `msg03.txt`
25. `msg04.txt`
26. `exms0.txt`
27. `exms1.txt`
28. `exms2.txt`
29. `exms3.txt`
30. `wireless1.txt`
31. `wireless2.txt`

The PC string tables are all UTF-8 encoded. Each line is prefixed with a key, but this is not used by the textbox ops.
Some PC string tables have variants for players using a control pad, these contain `_pad` at the end of their filename.

## Control codes

All dialogue text is split up into pages by `<PAGE>` or `<AUTO_PAGE>` control codes. Players can advance the dialogue
to the next page after it has finished "typing" onto the screen. Neither the SNES nor PC version do any sort of
automatic word wrapping.

Both the SNES and PC version dialogue can be decoded into a similar set of control codes. The PC version codes are
used as a basis. Note that some of the control codes listed are split from a single control code in the SNES dialogue
text, as described in [Strings](../data/strings.md).

### Text flow

- `<PAGE>` Page end
- `<AUTO_PAGE>` Page end and automatically go to the next page
- `<INDENT>` 3 space indent
- `<S10>` Some amount of indentation, PC only
- `<BR>` Line break
- `<WAIT>$xx</WAIT>` Wait for `$xx` number of ticks, then auto-progress
- `<CT>` Center this line horizontally - not yet implemented

### Variable replacements

- `<NAME_CRO>` Crono's name
- `<NAME_MAR>` Marle's name
- `<NAME_LUC>` Lucca's name
- `<NAME_FRO>` Frog's name
- `<NAME_ROB>` Robo's name
- `<NAME_AYL>` Ayla's name
- `<NAME_MAG>` Magus' name
- `<NICK_CRO>` The nickname for Crono used by Ayla in the Japanese version
- `<NAME_PT1>` Party member 1 name
- `<NAME_PT2>` Party member 2 name
- `<NAME_PT3>` Party member 3 name
- `<NAME_LEENE>` Always replaced by "Leene", otherwise unused, SNES only
- `<NAME_SIL>` Epoch's name (from "Sil Bird")
- `<NUMBER>` A number from a textbox choice result, PC only
- `<NUMBER 8>` A 8-bit number from the textbox choice result at `$7E0200`, SNES only
- `<NUMBER 16>` A 16-bit number from the textbox choice result at `$7E0200`, SNES only
- `<NUMBER 24>` A 24-bit number from the textbox choice result at `$7E0200`, SNES only
- `<NAME_ITM>` An item name from the value stored at `$7F0200`
- `<NAME_TEC>` A tech name - Not yet implemented

### Choices

These are only used by the PC version to tag a choice's text. The actual line index is still used to track what choice
was made.

- `<C1>...</C1>` Choice option 1
- `<C2>...</C2>` Choice option 2
- `<C3>...</C3>` Choice option 3
- `<C4>...</C4>` Choice option 4

### Coliseum related

- `<STR>` - not yet implemented
- `<NAME_MON>` - not yet implemented

### Icons

These precede item names in the SNES version. These are not yet implemented.

- `<BLADE>`
- `<BOW>`
- `<GUN>`
- `<ARM>`
- `<SWORD>`
- `<FIST>`
- `<SCYTHE>`
- `<HELM>`
- `<ARMOR>`
- `<RING>`

### Battle UI symbols

These are likely used by the battle UI in the SNES version. These are not implemented.

- `<H>` `<M>` `<P>` `<:>`
- `<SHIELD>` `<STAR>`
- `<LEFT>` `<RIGHT>`
- `<HAND1>` `<HAND2>` `<HAND3>` `<HAND4>`
- `<H>` `<M>` `<P>`
- `<HP0>` `<HP1>` `<HP2>` `<HP3>` `<HP4>` `<HP5>` `<HP6>` `<HP7>` `<HP8>`
- `<D>` `<Z>`
- `<UP>` `<A>` `<L>` `<R>`
- `<H>` `<M>` `<P>`
- `<CORNER>`

## Choices

If a choice op is used to display a dialogue string, the player can choose an option on the last page of dialogue. The
choice parameter byte indicates where the lines containing a choice start and end. The first 2 bits of that byte is the
last choice line on the page, the next 2 bits is the first line. The selected choice is stored in the `result` value
of the actor that displays the dialogue.

## Textbox positioning

A textbox can be placed at the top or bottom of the screen. This can also be determined automatically based on the
position of the actor that displays the textbox. If that actor is above the camera center, the textbox shows on the
bottom and vice versa.

## Op descriptions

### `$B8 string_table <table>`

- `table`: a 24 bit address in the SNES version, a single byte in the PC version.

Sets the current string table. 

---

### `$BB textbox <string index> position=auto`

- `string index` byte: the string index to display
- `position`: where to show the textbox

Displays a textbox with dialogue. The textbox is positioned automatically.

---

### `$C0 choice <string index> <choice lines> position=auto`

- `string index` byte: the string index to display
- `choice lines` byte: 2 bits where the choice lines end, and 2 more bits where they start
- `position`: where to show the textbox

Displays a textbox with dialogue and a choice on the last page. The textbox is positioned automatically.

---

### `$C1 textbox <string index> top`

- `string index` byte: the string index to display
- `position`: where to show the textbox

Displays a textbox with dialogue. The textbox is positioned at the top.

---

### `$C2 textbox <string index> bottom`

- `string index` byte: the string index to display
- `position`: where to show the textbox

Displays a textbox with dialogue. The textbox is positioned at the bottom.

---

### `$C3 choice <string index> <choice lines> top`

- `string index` byte: the string index to display
- `choice lines` byte: 2 bits where the choice lines end, and 2 more bits where they start
- `position`: where to show the textbox

Displays a textbox with dialogue and a choice on the last page. The textbox is positioned at the top.

---

### `$C4 choice <string index> <choice lines> bottom`

- `string index` byte: the string index to display
- `choice lines` byte: 2 bits where the choice lines end, and 2 more bits where they start
- `position`: where to show the textbox

Displays a textbox with dialogue and a choice on the last page. The textbox is positioned at the bottom.

---

### `$C8 ui <ui>`

- `ui` byte: what UI dialog to display.
    - If `$00`, the character switch UI will open.
  - If `$01`, the load game UI will open. Bit `$40` has an unknown special meaning.
  - If `$02`, the save game UI will open. Bit `$40` has an unknown special meaning.
  - If only bit `$80` is set, it will open the shop UI for that shop index.
  - If bits `$C0` are set, it will open the rename UI for that given player character.

Opens a special UI dialog.
