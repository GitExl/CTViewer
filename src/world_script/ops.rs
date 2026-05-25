use crate::memory::{DataDest, DataSource};
use crate::shared_op::{BitMathOp, ByteMathOp, CompareOp};

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Op {
    NOP,

    AddActor {
        address: u16,
        i0: u8,
    },
    AddActorS {
        address: u16,
        i0: u8,
    },
    AnimWait {
        speed: u8,
    },
    Bind {
        address: u16,
        pc: u8,
    },
    BitMath {
        lhs: DataDest,
        rhs: DataSource,
        op: BitMathOp,
    },
    ByteMath {
        lhs: DataDest,
        rhs: DataSource,
        op: ByteMathOp,
    },
    Call {
        address: u16,
    },
    CallFar {
        address: usize,
    },
    ChangeLocation {
        location: u16,
        i0: u8,
        x: u8,
        y: u8,
    },
    Copy8 {
        lhs: DataDest,
        rhs: DataSource,
    },
    CopyTiles {
        source_layer: u8,
        source_x: u8,
        source_y: u8,
        dest_layer: u8,
        dest_x: u8,
        dest_y: u8,
        width: u8,
        height: u8,
    },
    DecrementAndJumpIfNonZero {
        address: DataDest,
        offset: isize,
    },
    End,
    FadeIn {
        i0: u8,
    },
    FadeOut {
        i0: u8,
    },
    Func {
        address: u16,
    },
    InitLayers {
        i0: u8,
    },
    InitMemory,
    Jump {
        address: usize,
    },
    JumpConditional {
        lhs: DataSource,
        rhs: DataSource,
        cmp: CompareOp,
        offset: isize,
    },
    Link {
        x: u8,
        y: u8,
    },
    Move {
        steps: u8,
    },
    MoveExtended {
        i0: u8,
        i1: u8,
        i2: u8,
    },
    MusicCommand {
        i0: u8,
        i1: u8,
        i2: u8,
        i3: u8,
    },
    PaletteExtended {
        i0: u8,
        i1: u8,
        i2: u8,
        i3: u8,
    },
    PlayMusic {
        music_index: u8,
    },
    PlayMusicS {
        music_index: u8,
    },
    PlaySound1 {
        sound: u8,
        position: i8,
    },
    PlaySound2 {
        sound: u8,
        position: i8,
    },
    Return,
    Scroll {
        time: u8,
    },
    ScrollR {
        i0: u8,
        i1: u8,
    },
    SetAnimation {
        anim_index: u8,
    },
    SetCoordinates {
        x: u16,
        y: u16,
    },
    SetPalette {
        index: u8,
    },
    SetPriority {
        priority: u8,
    },
    SetTile {
        layer: u8,
        x: u8,
        y: u8,
        tile_index: u8,
    },
    SetTileR {
        layer: u8,
        x: u8,
        y: u8,
        tile_index: u8,
    },
    Sleep {
        address: u16,
    },
    SpeedX {
        a: i16,
        b: i16,
    },
    SpeedY {
        a: i16,
        b: i16,
    },
    TpMoveX {
        i0: u16,
        i1: u16,
    },
    TpMoveY {
        i0: u16,
        i1: u16,
    },
    Unknown03 {
        i0: u8,
        i1: u8,
        i2: u8,
        i3: u8,
        i4: u8,
        i5: u8,
        i6: u8,
        i7: u8,
        i8: u8,
    },
    Unknown04 {
        address: usize,
        i0: u8,
        i1: u8,
    },
    Unknown33 {
        i0: u8,
        i1: u16,
        i2: u16,
        i3: u16,
    },
    Wait {
        i0: u8,
    },
    Wake {
        address: u16,
    },
}
