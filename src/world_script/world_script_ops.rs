use crate::destination::Destination;
use crate::memory::{DataDest, DataSource};
use crate::shared_op::{BitMathOp, ByteMathOp, CompareOp};

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Op {
    AddActor {
        address: u64,
        unused: u8,
    },
    AddActorSpecial {
        address: u64,
        i0: u8,
    },
    WaitThenAnimate {
        delay: u8,
    },
    Bind {
        address: u64,
        pc: u8,
    },
    BitMath {
        dest: DataDest,
        lhs: DataSource,
        rhs: DataSource,
        op: BitMathOp,
    },
    ByteMath {
        dest: DataDest,
        lhs: DataSource,
        rhs: DataSource,
        op: ByteMathOp,
    },
    GoSub {
        address: u64,
    },
    CallFunctionFar {
        address: usize,
    },
    ChangeLocation {
        destination: Destination,
    },
    Copy8 {
        lhs: DataDest,
        rhs: DataSource,
    },
    CopyTiles {
        source_layer: usize,
        source_x: usize,
        source_y: usize,
        dest_layer: usize,
        dest_x: usize,
        dest_y: usize,
        width: usize,
        height: usize,
    },
    DecrementAndJumpIfNonZero {
        src: DataSource,
        dest: DataDest,
        offset: i64,
    },
    End,
    FadeIn {
        delay: u8,
    },
    FadeOut {
        delay: u8,
    },
    CallFunction {
        address: u64,
    },
    InitBackgroundLayer {
        layer: u8,
    },
    InitMemory,
    GoTo {
        address: u64,
    },
    JumpConditional {
        lhs: DataSource,
        rhs: DataSource,
        cmp: CompareOp,
        offset: i64,
    },
    Link {
        address: u64,
    },
    LinkSpecial {
        address: u64,
    },
    MosaicIn {
        mode: u16,
    },
    MosaicOut {
        mode: u16
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
        flags1: u8,
        music_index: u8,
        flags2: u8,
        extra: u8,
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
        position: u8,
    },
    PlaySound2 {
        sound: u8,
        position: u8,
    },
    Return,
    Scroll {
        steps: u8,
    },
    ScrollLayer {
        layer: usize,
        steps: u8,
    },
    SetAnimation {
        anim_index: u8,
    },
    SetPosition {
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
        layer: usize,
        x: usize,
        y: usize,
        tile_index: usize,
    },
    SetTileR {
        layer: u8,
        x: u8,
        y: u8,
        tile_index: u8,
    },
    ExitClose {
        address: u16,
    },
    VectorX {
        magnitude: i32,
    },
    VectorY {
        magnitude: i32,
    },
    Timer {
        value: u8,
    },
    MoveToX {
        steps: u16,
        animation1: u8,
        animation2: u8,
    },
    MoveToY {
        steps: u16,
        animation1: u8,
        animation2: u8,
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
    PaletteLoad {
        address: usize,
        palette_index: u8,
        mode: u8,
    },
    BgAnimate {
        i0: u8,
        i1: u16,
        i2: u16,
        i3: u16,
    },
    Wait {
        steps: u8,
    },
    ExitOpen {
        address: u16,
    },
}
