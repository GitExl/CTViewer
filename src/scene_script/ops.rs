use crate::actor::ActorFlags;
use crate::scene_script::ops_actor_props::SpritePriority;
use crate::scene_script::ops_call::WaitMode;
use crate::scene_script::ops_char_load::CharacterType;
use crate::scene_script::ops_dialogue::{DialogueInput, DialoguePosition, DialogueSpecialType};
use crate::scene_script::ops_jump::CompareOp;
use crate::scene_script::ops_math::{BitMathOp, ByteMathOp};
use crate::scene_script::ops_palette::{ColorMathMode, SubPalette};
use crate::scene_script::scene_script_decoder::{ActorRef, BattleFlags, CopyTilesFlags, DataRef, ScrollLayerFlags, SpecialEffect};

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
        speed: DataRef,
    },
    ActorCoordinatesGet {
        actor: ActorRef,
        x: DataRef,
        y: DataRef,
    },
    ActorCoordinatesSet {
        actor: ActorRef,
        x: DataRef,
        y: DataRef,
        precise: bool,
    },
    ActorDirectionGet {
        actor: ActorRef,
        source: DataRef,
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

    // Actor movement.
    ActorMoveTo {
        actor: ActorRef,
        x: DataRef,
        y: DataRef,
        distance: DataRef,
        update_direction: bool,
        animated: bool,
    },
    ActorMoveToActor {
        actor: ActorRef,
        to_actor: ActorRef,
        distance: DataRef,
        update_direction: bool,
        animated: bool,
        distant: bool,
        forever: bool,
    },
    ActorMoveAtAngle {
        actor: ActorRef,
        angle: DataRef,
        distance: DataRef,
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
        direction: DataRef,
    },
    ActorSetDirectionTowards {
        actor: ActorRef,
        to: ActorRef,
    },

    // Animation.
    // 0 loops means do not loop, only play once.
    // 0xFFFFFFFF loops means loop forever.
    Animate {
        actor: ActorRef,
        animation: DataRef,
        wait: bool,
        run: bool,
        loops: DataRef,
    },
    AnimationLimit {
        limit: u8,
    },

    // Code jumps.
    Jump {
        offset: isize,
    },
    JumpConditional {
        lhs: DataRef,
        rhs: DataRef,
        width: usize,
        cmp: CompareOp,
        offset: isize,
    },

    // Data copy.
    Copy {
        dest: DataRef,
        source: DataRef,
        width: usize,
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
    },
    PaletteSet {
        palette: usize,
    },
    PaletteRestore,

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
        rhs: DataRef,
        lhs: DataRef,
        byte_count: usize,
        op: ByteMathOp,
    },
    BitMath {
        rhs: DataRef,
        lhs: DataRef,
        op: BitMathOp,
    },

    // Dialogue.
    DialogueSetTable {
        address: DataRef,
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
        item: DataRef,
    },
    ItemTake {
        actor: ActorRef,
        item: DataRef,
    },
    GoldGive {
        actor: ActorRef,
        amount: DataRef,
    },
    GoldTake {
        actor: ActorRef,
        amount: DataRef,
    },
    ItemGetAmount {
        item: usize,
        dest: DataRef,
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
        index_direction: DataRef,
        x: DataRef,
        y: DataRef,
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
