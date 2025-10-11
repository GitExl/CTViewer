use crate::actor::{ActorFlags, DrawMode};
use crate::destination::Destination;
use crate::scene::textbox::TextBoxPosition;
use crate::scene_script::decoder::ops_char_load::CharacterType;
use crate::scene_script::decoder::ops_textbox::{TextBoxInput, DialogueSpecialType};
use crate::scene_script::decoder::ops_jump::CompareOp;
use crate::scene_script::decoder::ops_math::{BitMathOp, ByteMathOp};
use crate::scene_script::decoder::ops_palette::{ColorMathMode, SubPalette};
use crate::scene_script::scene_script_decoder::{ActorRef, BattleFlags, CopyTilesFlags, ScrollLayerFlags, SpecialEffect};
use crate::memory::{DataDest, DataSource};
use crate::sprites::sprite_renderer::SpritePriority;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Op {
    NOP,

    // Script execution.
    Return,
    Yield {
        forever: bool,
    },
    SetScriptDelay {
        /// Delay in ticks between script processing.
        delay: u32,
    },
    Wait {
        ticks: u32,
    },
    Control {
        forever: bool,
    },
    SetScriptProcessing {
        actor: ActorRef,
        enabled: bool,
    },

    // Function calls.
    Call {
        actor: ActorRef,
        priority: usize,
        function: usize,
    },
    CallWaitCompletion {
        actor: ActorRef,
        priority: usize,
        function: usize,
    },
    CallWaitReturn {
        actor: ActorRef,
        priority: usize,
        function: usize,
    },

    // Actor properties.
    ActorUpdateFlags {
        actor: ActorRef,
        set: ActorFlags,
        remove: ActorFlags,
    },
    ActorSetSpeed {
        actor: ActorRef,
        speed: DataSource,
    },
    ActorCoordinatesGet {
        actor: ActorRef,
        tile_x: DataDest,
        tile_y: DataDest,
    },
    ActorCoordinatesSet {
        actor: ActorRef,
        tile_x: DataSource,
        tile_y: DataSource,
    },
    ActorCoordinatesSetPrecise {
        actor: ActorRef,
        x: DataSource,
        y: DataSource,
    },
    ActorFacingGet {
        actor: ActorRef,
        source: DataSource,
    },
    ActorSetSpritePriority {
        actor: ActorRef,
        top: SpritePriority,
        bottom: SpritePriority,
        set_and_lock: bool,
        unknown_bits: u8,
    },
    ActorSetResult8 {
        actor: ActorRef,
        result: DataSource,
    },
    ActorSetResult16 {
        actor: ActorRef,
        result: DataSource,
    },
    ActorRemove {
        actor: ActorRef,
    },
    ActorSetDrawMode {
        actor: ActorRef,
        draw_mode: DrawMode,
    },

    // Actor movement.
    ActorMoveToTile {
        x: DataSource,
        y: DataSource,
        steps: Option<DataSource>,
        update_facing: bool,
        animated: bool,
    },
    ActorMoveToActor {
        to_actor: ActorRef,
        script_cycle_count: Option<u32>,
        update_facing: bool,
        animated: bool,
        into_battle_range: bool,
        forever: bool,
    },
    ActorMoveAtAngle {
        angle: DataSource,
        steps: DataSource,
        update_facing: bool,
        animated: bool,
    },
    MovePartyTo {
        pc0_x: i32,
        pc0_y: i32,
        pc1_x: i32,
        pc1_y: i32,
        pc2_x: i32,
        pc2_y: i32,
    },
    ActorJump {
        actor: ActorRef,
        x: i32,
        y: i32,
        height: u32,
    },
    ActorJumpUnknown {
        actor: ActorRef,
        move_x: i32,
        move_y: i32,
        steps: u32,
        unknown: u32,
    },

    ActorHeal {
        actor: ActorRef,
        hp: bool,
        mp: bool,
    },

    // Actor facing.
    ActorFacingSet {
        actor: ActorRef,
        facing: DataSource,
    },
    ActorSetFacingTowards {
        actor: ActorRef,
        to: ActorRef,
    },

    // Animation.
    Animation {
        actor: ActorRef,
        animation: DataSource,
    },
    AnimationLoopCount {
        actor: ActorRef,
        animation: DataSource,
        loops: DataSource,
    },
    AnimationReset {
        actor: ActorRef,
    },
    AnimationStaticFrame {
        actor: ActorRef,
        frame: DataSource,
    },

    // Code jumps.
    Jump {
        offset: i64,
    },
    JumpConditional8 {
        lhs: DataSource,
        cmp: CompareOp,
        rhs: DataSource,
        offset: i64,
    },
    JumpConditional16 {
        lhs: DataSource,
        cmp: CompareOp,
        rhs: DataSource,
        offset: i64,
    },
    JumpConditionalDrawMode {
        actor: ActorRef,
        draw_mode: DrawMode,
        offset: i64,
    },
    JumpConditionalBattleRange {
        actor: ActorRef,
        offset: i64,
    },

    // Data copy.
    Copy8 {
        dest: DataDest,
        source: DataSource,
    },
    Copy16 {
        dest: DataDest,
        source: DataSource,
    },
    CopyBytes {
        dest: DataDest,
        bytes: [u8; 64],
        length: usize,
    },

    // Color math.
    ColorMath {
        mode: ColorMathMode,
        r: bool,
        g: bool,
        b: bool,
        color_start: u8,
        color_count: u8,
        intensity_start: f64,
        intensity_end: f64,
        duration: f64,
    },

    // Palettes.
    PaletteSetImmediate {
        sub_palette: SubPalette,
        color_index: usize,
        data: [u8; 64],
        length: usize,
    },
    PaletteSet {
        palette: usize,
    },
    PaletteRestore,

    // Random value.
    Random {
        dest: DataDest,
    },

    // Character loading.
    LoadCharacter {
        char_type: CharacterType,
        index: usize,
        must_be_in_party: bool,
        is_static: bool,
        battle_index: usize,
    },

    // Math.
    ByteMath8 {
        dest: DataDest,
        lhs: DataSource,
        op: ByteMathOp,
        rhs: DataSource,
    },
    ByteMath16 {
        dest: DataDest,
        lhs: DataSource,
        op: ByteMathOp,
        rhs: DataSource,
    },
    BitMath {
        dest: DataDest,
        lhs: DataSource,
        op: BitMathOp,
        rhs: DataSource,
    },

    // Textboxes.
    TextSetTable {
        address: usize,
    },
    TextBoxShow {
        index: usize,
        position: TextBoxPosition,
        input: TextBoxInput,
    },

    // Special dialogues.
    DialogueSpecial {
        dialogue_type: DialogueSpecialType,
    },

    // Inventory.
    ItemGive {
        actor: ActorRef,
        item: DataSource,
        category: usize,
    },
    ItemTake {
        actor: ActorRef,
        item: DataSource,
        category: usize,
    },
    GoldGive {
        actor: ActorRef,
        amount: DataSource,
    },
    GoldTake {
        actor: ActorRef,
        amount: DataSource,
    },
    ItemGetAmount {
        item: usize,
        category: usize,
        dest: DataSource,
    },

    // Party management.
    PartyMemberMakeActive {
        pc: usize,
    },
    PartyMemberAddToReserve {
        pc: usize,
    },
    PartyMemberRemove {
        pc: usize,
    },
    PartyMemberRemoveFromActive {
        pc: usize,
    },
    PartyMemberToReserve {
        pc: usize,
    },
    PartyMemberEquip {
        pc: usize,
        item: usize,
    },
    PartyFollow,
    PartyExploreMode {
        value: u8,
    },

    // Change location.
    ChangeLocation {
        destination: Destination,
        instant: bool,
        queue_different_unknown: bool,
    },
    ChangeLocationFromMemory {
        byte1: DataSource,
        byte2: DataSource,
        byte3: DataSource,
        byte4: DataSource,
    },

    // Start a battle.
    Battle {
        flags: BattleFlags,
    },

    // Copy tiles from somewhere on the map.
    CopyTiles {
        left: u32,
        top: u32,
        right: u32,
        bottom: u32,
        dest_x: u32,
        dest_y: u32,
        flags: CopyTilesFlags,
        delayed: bool,
    },

    // Scroll map layers.
    ScrollLayers {
        x: i32,
        y: i32,
        flags: ScrollLayerFlags,
        duration: u32,
    },
    MoveCameraTo {
        x: i32,
        y: i32,
    },

    // Sound and music.
    SoundPlay {
        sound: usize,
        panning: f64,
    },
    MusicPlay {
        music: usize,
        interrupt: bool,
    },
    MusicVolumeSlide {
        duration: f64,
        volume: f64,
    },
    MusicTempoSlide {
        duration: f64,
        tempo: u8,
    },
    SoundVolumeSlide {
        left: f64,
        right: f64,
        duration: f64,
    },
    SoundWaitEnd,
    MusicWaitEnd,

    // Screen effects.
    ScreenFade {
        target: f64,
        delay: usize,
    },
    ScreenColorMath {
        r: u8,
        g: u8,
        b: u8,
        intensity: f64,
        mode: ColorMathMode,
        duration: f64,
    },
    ScreenWaitForFade,
    ScreenWaitForColorMath,
    ScreenShake {
        enabled: bool,
    },
    ScreenColorMathGeometry {
        unknown: u8,

        x1_src: u8,
        x1_dest: u8,
        y1_src: u8,
        y1_dest: u8,

        x2_src: u8,
        x2_dest: u8,
        y2_src: u8,
        y2_dest: u8,

        x3_src: u8,
        x3_dest: u8,
        y3_src: u8,
        y3_dest: u8,

        x4_src: u8,
        x4_dest: u8,
        y4_src: u8,
        y4_dest: u8,
    },

    // Specials.
    SpecialScene {
        scene: usize,
        flags: u8,
    },
    SpecialOpenPortal {
        value1: u8,
        value2: u8,
        value3: u8,
    },
    SpecialEffect(SpecialEffect),

    // Unknown.
    Unknown {
        code: u8,
        data: [u8; 4],
    },
}
