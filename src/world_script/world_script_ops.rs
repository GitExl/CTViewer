use crate::destination::Destination;
use crate::memory::{DataDest, DataSource};
use crate::shared_op::{BitMathOp, ByteMathOp, CompareOp};
use crate::world_script::function_dispatch::WorldActorFunction;
use crate::world_script::task_dispatch::WorldActorTask;

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
    WaitAndAnimate {
        steps: u8,
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
        function: WorldActorFunction,
        address: u32,
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
        function: WorldActorFunction,
        address: u32,
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
        task: WorldActorTask,
        address: u32,
    },
    LinkSpecial {
        task: WorldActorTask,
        address: u32,
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
        anim_index: usize,
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
        animation1: usize,
        animation2: usize,
    },
    MoveToY {
        steps: u16,
        animation1: usize,
        animation2: usize,
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
    CopyToVram {
        source_address: u64,
        vram_dest_address: u16,
        byte_count: u16,
    },
    Wait {
        steps: u8,
    },
    ExitOpen {
        address: u16,
    },
}
