use crate::actor::ActorFlags;
use crate::scene_script::ops_call::WaitMode;
use crate::scene_script::ops_char_load::CharacterType;
use crate::scene_script::ops_dialogue::{DialogueInput, DialoguePosition, DialogueSpecialType};
use crate::scene_script::ops_jump::CompareOp;
use crate::scene_script::ops_math::{BitMathOp, ByteMathOp};
use crate::scene_script::ops_palette::{ColorMathMode, SubPalette};
use crate::scene_script::scene_script_decoder::{ActorRef, BattleFlags, CopyTilesFlags, ScrollLayerFlags, SpecialEffect};
use crate::scene_script::scene_script_memory::{DataDest, DataSource};
use crate::sprites::sprite_renderer::SpritePriority;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Op {
    NOP,

    // Script execution.
    Return,
    Yield {
        forever: bool,
    },
    SetScriptSpeed {
        speed: u32,
    },
    Wait {
        duration: usize,
    },
    Control {
        forever: bool,
    },

    // Function calls.
    Call {
        actor: ActorRef,
        priority: usize,
        function: usize,
        wait_mode: WaitMode,
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
        x: DataSource,
        y: DataSource,
    },
    ActorCoordinatesSet {
        actor: ActorRef,
        x: DataSource,
        y: DataSource,
    },
    ActorCoordinatesSetPrecise {
        actor: ActorRef,
        x: DataSource,
        y: DataSource,
    },
    ActorDirectionGet {
        actor: ActorRef,
        source: DataSource,
    },
    ActorMoveJump {
        actor: ActorRef,
        x: i32,
        y: i32,
        height: u32,
    },
    ActorSetSpritePriority {
        actor: ActorRef,
        top: SpritePriority,
        bottom: SpritePriority,
        mode_set: bool,
        unknown_bits: u8,
    },
    ActorSetResult {
        actor: ActorRef,
        result: DataSource,
    },

    // Actor movement.
    ActorMoveTo {
        actor: ActorRef,
        x: DataSource,
        y: DataSource,
        distance: DataSource,
        update_direction: bool,
        animated: bool,
    },
    ActorMoveToActor {
        actor: ActorRef,
        to_actor: ActorRef,
        distance: DataSource,
        update_direction: bool,
        animated: bool,
        distant: bool,
        forever: bool,
    },
    ActorMoveAtAngle {
        actor: ActorRef,
        angle: DataSource,
        distance: DataSource,
        update_direction: bool,
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
    ActorHeal {
        actor: ActorRef,
        hp: bool,
        mp: bool,
    },

    // Actor direction.
    ActorSetDirection {
        actor: ActorRef,
        direction: DataSource,
    },
    ActorSetDirectionTowards {
        actor: ActorRef,
        to: ActorRef,
    },

    // Sprite.
    ActorSetSpriteFrame {
        actor: ActorRef,
        frame: DataSource,
    },

    // Animation.
    // 0 loops means do not loop, only play once.
    // 0xFFFFFFFF loops means loop forever.
    Animate {
        actor: ActorRef,
        animation: DataSource,
        wait: bool,
        run: bool,
        loops: DataSource,
    },
    AnimationLimit {
        limit: u8,
    },

    // Code jumps.
    Jump {
        offset: isize,
    },
    JumpConditional {
        lhs: DataSource,
        cmp: CompareOp,
        width: usize,
        rhs: DataSource,
        offset: isize,
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
        bytes: [u8; 32],
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
        data: [u8; 32],
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
    ByteMath {
        dest: DataDest,
        lhs: DataSource,
        op: ByteMathOp,
        rhs: DataSource,
        byte_count: usize,
    },
    BitMath {
        dest: DataDest,
        lhs: DataSource,
        op: BitMathOp,
        rhs: DataSource,
    },

    // Dialogue.
    DialogueSetTable {
        address: usize,
    },
    DialogueShow {
        index: usize,
        position: DialoguePosition,
        input: DialogueInput,
    },
    DialogueSpecial {
        dialogue_type: DialogueSpecialType,
    },

    // Inventory.
    ItemGive {
        actor: ActorRef,
        item: DataSource,
    },
    ItemTake {
        actor: ActorRef,
        item: DataSource,
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
        index_direction: DataSource,
        x: DataSource,
        y: DataSource,
        variant: u8,
    },

    // Start a battle.
    Battle {
        flags: BattleFlags,
    },

    // Copy tiles from somewhere on the map.
    CopyTiles {
        left: u8,
        top: u8,
        right: u8,
        bottom: u8,
        x: u8,
        y: u8,
        flags: CopyTilesFlags,
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
    ScreenDarken {
        duration: f64,
    },
    ScreenColorMath {
        r: u8,
        g: u8,
        b: u8,
        intensity: f64,
        mode: ColorMathMode,
        duration: f64,
    },
    ScreenFadeOut,
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
