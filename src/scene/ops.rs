use crate::actor::ActorFlags;
use crate::scene::ops_actor_props::SpritePriority;
use crate::scene::ops_call::WaitMode;
use crate::scene::ops_char_load::CharacterType;
use crate::scene::ops_jump::CompareOp;
use crate::scene::ops_math::{BitMathOp, ByteMathOp};
use crate::scene::scene_script_decoder::{ActorRef, ColorMathMode, DataRef, SubPalette};

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Op {
    NOP,

    // Script execution.
    Yield,
    SetScriptSpeed {
        speed: u8,
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
    },
    ActorMoveAtAngle {
        actor: ActorRef,
        angle: DataRef,
        distance: DataRef,
        update_direction: bool,
        animated: bool,
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

    // Unknown.
    Unknown {
        code: u8,
        data: [u8; 4],
    },
}
