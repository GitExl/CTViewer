use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use crate::destination::Destination;
use crate::GameMode;
use crate::shared_op::{BitMathOp, ByteMathOp, CompareOp};
use crate::memory::{DataDest, DataSource};
use crate::util::data_read::read_24_bit_address;
use crate::world_script::world_script_ops::Op;

/// Opcodes.
pub fn op_decode(data: &mut Cursor<Vec<u8>>, mode: GameMode) -> Option<Op> {
    let op_byte = match data.read_u8() {
        Ok(op_byte) => op_byte,
        Err(_) => {
            println!("Script execution past end of data at 0x{:04X}.", data.position());
            return None;
        }
    };

    let op = match op_byte {

        // Misc. ops.
        // "initialize"
        0x00 => Op::InitMemory,
        // "grp"
        0x03 => Op::Unknown03 {
            i0: data.read_u8().unwrap(),
            i1: data.read_u8().unwrap(),
            i2: data.read_u8().unwrap(),
            i3: data.read_u8().unwrap(),
            i4: data.read_u8().unwrap(),
            i5: data.read_u8().unwrap(),
            i6: data.read_u8().unwrap(),
            i7: data.read_u8().unwrap(),
            i8: data.read_u8().unwrap(),
        },
        // "pal", source data address, number of colors, mode
        0x04 => Op::PaletteLoad {
            address: read_24_bit_address(data),
            palette_index: data.read_u8().unwrap(),
            mode: data.read_u8().unwrap(),
        },
        // "mapjump"
        0x05 => Op::ChangeLocation {
            destination: Destination::from_cursor(data, mode),
        },
        // "initscreen"
        0x3E => Op::InitBackgroundLayer {
            layer: data.read_u8().unwrap(),
        },

        // Memory/math.
        // "clr"
        0x0A => Op::Copy8 {
            lhs: DataDest::for_world_actor_memory(data.read_u8().unwrap() as usize),
            rhs: DataSource::Immediate(0),
        },
        // "incr"
        0x0B => Op::ByteMath {
            lhs: DataDest::for_world_actor_memory(data.read_u8().unwrap() as usize),
            rhs: DataSource::Immediate(1),
            op: ByteMathOp::Add,
        },
        // "decr"
        0x0C => Op::ByteMath {
            lhs: DataDest::for_world_actor_memory(data.read_u8().unwrap() as usize),
            rhs: DataSource::Immediate(1),
            op: ByteMathOp::Subtract,
        },
        // "setr"
        0x0D => Op::Copy8 {
            lhs: DataDest::for_world_actor_memory(data.read_u8().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
        },
        // "bitsetr"
        0x0E => Op::BitMath {
            lhs: DataDest::for_world_actor_memory(data.read_u8().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            op: BitMathOp::Or,
        },
        // "bitclr"
        0x0F => Op::BitMath {
            lhs: DataDest::for_world_actor_memory(data.read_u8().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            op: BitMathOp::Xor,
        },
        // "memclr"
        0x10 => Op::Copy8 {
            lhs: DataDest::for_system_memory(data.read_u16::<LittleEndian>().unwrap() as usize),
            rhs: DataSource::Immediate(0),
        },
        // "meminc"
        0x11 => Op::ByteMath {
            lhs: DataDest::for_system_memory(data.read_u16::<LittleEndian>().unwrap() as usize),
            rhs: DataSource::Immediate(1),
            op: ByteMathOp::Add,
        },
        // "memdec"
        0x12 => Op::ByteMath {
            lhs: DataDest::for_system_memory(data.read_u16::<LittleEndian>().unwrap() as usize),
            rhs: DataSource::Immediate(1),
            op: ByteMathOp::Subtract,
        },
        // "memset"
        0x13 => Op::Copy8 {
            lhs: DataDest::for_system_memory(data.read_u16::<LittleEndian>().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
        },
        // "membitset"
        0x14 => Op::BitMath {
            lhs: DataDest::for_system_memory(data.read_u16::<LittleEndian>().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            op: BitMathOp::Or,
        },
        // "membitclr"
        0x15 => Op::BitMath {
            lhs: DataDest::for_system_memory(data.read_u16::<LittleEndian>().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            op: BitMathOp::Xor,
        },
        // "trnlg"
        0x16 => Op::Copy8 {
            lhs: DataDest::for_system_memory(data.read_u16::<LittleEndian>().unwrap() as usize),
            rhs: DataSource::for_world_actor_memory(data.read_u8().unwrap() as usize),
        },
        // "trngl"
        0x17 => Op::Copy8 {
            lhs: DataDest::for_world_actor_memory(data.read_u8().unwrap() as usize),
            rhs: DataSource::for_system_memory(data.read_u16::<LittleEndian>().unwrap() as usize),
        },

        // "trnr"
        0x18 => Op::Copy8 {
            lhs: DataDest::for_world_actor_memory(data.read_u8().unwrap() as usize),
            rhs: DataSource::for_world_actor_memory(data.read_u8().unwrap() as usize),
        },
        // "trnmem"
        0x19 => Op::Copy8 {
            lhs: DataDest::for_system_memory(data.read_u16::<LittleEndian>().unwrap() as usize),
            rhs: DataSource::for_system_memory(data.read_u16::<LittleEndian>().unwrap() as usize),
        },
        // "addr"
        0x46 => Op::ByteMath {
            lhs: DataDest::for_world_actor_memory(data.read_u8().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            op: ByteMathOp::Add,
        },
        // "subr"
        0x47 => Op::ByteMath {
            lhs: DataDest::for_world_actor_memory(data.read_u8().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            op: ByteMathOp::Subtract,
        },
        // "memadd"
        0x48 => Op::ByteMath {
            lhs: DataDest::for_system_memory(data.read_u16::<LittleEndian>().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            op: ByteMathOp::Add,
        },
        // "memsub"
        0x49 => Op::ByteMath {
            lhs: DataDest::for_system_memory(data.read_u16::<LittleEndian>().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            op: ByteMathOp::Subtract,
        },

        // Jumps.
        // "jp"
        0x1A => Op::GoTo {
            address: data.read_u16::<LittleEndian>().unwrap() as usize - 0x400,
        },
        // "jdjnz"
        0x1B => Op::DecrementAndJumpIfNonZero {
            address: DataDest::for_world_actor_memory(data.read_u8().unwrap() as usize),
            offset: data.read_i8().unwrap() as isize,
        },
        // "jz"
        0x1C => Op::JumpConditional {
            lhs: DataSource::for_world_actor_memory(data.read_u8().unwrap() as usize),
            rhs: DataSource::Immediate(0),
            cmp: CompareOp::Eq,
            offset: data.read_i8().unwrap() as isize,
        },
        // "jnz"
        0x1D => Op::JumpConditional {
            lhs: DataSource::for_world_actor_memory(data.read_u8().unwrap() as usize),
            rhs: DataSource::Immediate(0),
            cmp: CompareOp::NotEq,
            offset: data.read_i8().unwrap() as isize,
        },
        // "jcpnz"
        0x1E => Op::JumpConditional {
            lhs: DataSource::for_world_actor_memory(data.read_u8().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            cmp: CompareOp::NotEq,
            offset: data.read_i8().unwrap() as isize,
        },
        // "jcpz"
        0x1F => Op::JumpConditional {
            lhs: DataSource::for_world_actor_memory(data.read_u8().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            cmp: CompareOp::Eq,
            offset: data.read_i8().unwrap() as isize,
        },
        // "jandnz"
        0x20 => Op::JumpConditional {
            lhs: DataSource::for_world_actor_memory(data.read_u8().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            cmp: CompareOp::And,
            offset: data.read_i8().unwrap() as isize,
        },
        // "jandz"
        0x21 => Op::JumpConditional {
            lhs: DataSource::for_world_actor_memory(data.read_u8().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            cmp: CompareOp::Or,
            offset: data.read_i8().unwrap() as isize,
        },
        // "jz_g"
        0x22 => Op::JumpConditional {
            lhs: DataSource::for_system_memory(data.read_u16::<LittleEndian>().unwrap() as usize),
            rhs: DataSource::Immediate(0),
            cmp: CompareOp::Eq,
            offset: data.read_i8().unwrap() as isize,
        },
        // "jnz_g"
        0x23 => Op::JumpConditional {
            lhs: DataSource::for_system_memory(data.read_u16::<LittleEndian>().unwrap() as usize),
            rhs: DataSource::Immediate(0),
            cmp: CompareOp::NotEq,
            offset: data.read_i8().unwrap() as isize,
        },
        // "jcpnz_g"
        0x24 => Op::JumpConditional {
            lhs: DataSource::for_system_memory(data.read_u16::<LittleEndian>().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            cmp: CompareOp::NotEq,
            offset: data.read_i8().unwrap() as isize,
        },
        // "jcpz_g"
        0x25 => Op::JumpConditional {
            lhs: DataSource::for_system_memory(data.read_u16::<LittleEndian>().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            cmp: CompareOp::Eq,
            offset: data.read_i8().unwrap() as isize,
        },
        // "jandnz_g"
        0x26 => Op::JumpConditional {
            lhs: DataSource::for_system_memory(data.read_u16::<LittleEndian>().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            cmp: CompareOp::And,
            offset: data.read_i8().unwrap() as isize,
        },
        // "jandz_g"
        0x27 => Op::JumpConditional {
            lhs: DataSource::for_system_memory(data.read_u16::<LittleEndian>().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            cmp: CompareOp::Or,
            offset: data.read_i8().unwrap() as isize,
        },
        // "jcpcc"
        0x4C => Op::JumpConditional {
            lhs: DataSource::for_system_memory(data.read_u16::<LittleEndian>().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            cmp: CompareOp::Lt,
            offset: data.read_i8().unwrap() as isize,
        },
        // "jcpcs"
        0x4D => Op::JumpConditional {
            lhs: DataSource::for_system_memory(data.read_u16::<LittleEndian>().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            cmp: CompareOp::GtEq,
            offset: data.read_i8().unwrap() as isize,
        },

        // Reveal/show.
        // "fadeout"
        0x28 => Op::FadeOut {
            mode: data.read_u8().unwrap(),
        },
        // "fadein"
        0x29 => Op::FadeIn {
            mode: data.read_u8().unwrap(),
        },
        // "mozin"
        0x2A => Op::MosaicIn {
            mode: data.read_u8().unwrap(),
        },
        // "mozout"
        0x2B => Op::MosaicOut {
            mode: data.read_u8().unwrap(),
        },

        // Sprite/appearance.
        // "colofs"
        0x01 => Op::SetPalette {
            index: data.read_u8().unwrap(),
        },
        // "priset"
        0x02 => Op::SetPriority {
            priority: data.read_u8().unwrap(),
        },
        // "anmseq"
        0x30 => Op::SetAnimation {
            anim_index: data.read_u8().unwrap(),
        },
        // "anmwait"
        0x39 => Op::WaitThenAnimate {
            delay: data.read_u8().unwrap(),
        },

        // Map changes.
        // "putmap"
        0x07 => Op::SetTile {
            layer: data.read_u8().unwrap(),
            x: data.read_u8().unwrap(),
            y: data.read_u8().unwrap(),
            tile_index: data.read_u8().unwrap(),
        },
        // "bganm"
        0x33 => Op::BgAnimate {
            i0: data.read_u8().unwrap(),
            i1: data.read_u16::<LittleEndian>().unwrap(),
            i2: data.read_u16::<LittleEndian>().unwrap(),
            i3: data.read_u16::<LittleEndian>().unwrap(),
        },
        // "copymap"
        0x4F => Op::CopyTiles {
            source_layer: data.read_u8().unwrap(),
            source_x: data.read_u8().unwrap(),
            source_y: data.read_u8().unwrap(),
            dest_layer: data.read_u8().unwrap(),
            dest_x: data.read_u8().unwrap(),
            dest_y: data.read_u8().unwrap(),
            width: data.read_u8().unwrap(),
            height: data.read_u8().unwrap(),
        },
        // "putmapr"
        0x50 => Op::SetTileR {
            layer: data.read_u8().unwrap(),
            x: data.read_u8().unwrap(),
            y: data.read_u8().unwrap(),
            tile_index: data.read_u8().unwrap(),
        },

        // Movement/position.
        // "pos"
        0x2C => Op::SetPosition {
            x: data.read_u16::<LittleEndian>().unwrap(),
            y: data.read_u16::<LittleEndian>().unwrap(),
        },
        // "vecx"
        0x2E => Op::VectorX {
            magnitude: data.read_i32::<LittleEndian>().unwrap(),
        },
        // "vecy"
        0x2F => Op::VectorY {
            magnitude: data.read_i32::<LittleEndian>().unwrap(),
        },
        // "move"
        0x31 => Op::Move {
            steps: data.read_u8().unwrap(),
        },
        // "scroll"
        0x32 => Op::Scroll {
            steps: data.read_u8().unwrap(),
        },
        // "scrollr
        0x51 => Op::ScrollLayer {
            layer: data.read_u8().unwrap(),
            steps: data.read_u8().unwrap(),
        },
        // "tpxmove"
        0x3F => Op::TpMoveX {
            steps: data.read_u16::<LittleEndian>().unwrap(),
            animation1: data.read_u8().unwrap(),
            animation2: data.read_u8().unwrap(),
        },
        // "tpymove"
        0x40 => Op::TpMoveY {
            steps: data.read_u16::<LittleEndian>().unwrap(),
            animation1: data.read_u8().unwrap(),
            animation2: data.read_u8().unwrap(),
        },

        // Function calls/new objects.
        // "bind"
        0x08 => Op::Bind {
            address: data.read_u16::<LittleEndian>().unwrap() - 0x400,
            pc: data.read_u8().unwrap(),
        },
        // "newevent"
        0x09 => Op::AddActor {
            address: data.read_u16::<LittleEndian>().unwrap() - 0x400,
            unused: data.read_u8().unwrap(),
        },
        // "func"
        0x34 => Op::CallFunction {
            address: data.read_u16::<LittleEndian>().unwrap(),
        },
        // "link"
        0x35 => Op::Link {
            address: data.read_u16::<LittleEndian>().unwrap(),
        },
        // "call"
        0x36 => Op::GoSub {
            address: data.read_u16::<LittleEndian>().unwrap() - 0x400,
        },
        // "return"
        0x37 => Op::Return,
        // "slink"
        0x42 => Op::LinkSpecial {
            address: data.read_u16::<LittleEndian>().unwrap(),
        },
        // "s_newevent"
        0x43 => Op::AddActorSpecial {
            address: data.read_u16::<LittleEndian>().unwrap() - 0x400,
            i0: data.read_u8().unwrap(),
        },
        // "func2"
        0x4E => Op::CallFunctionFar {
            address: read_24_bit_address(data),
        },
        // "taskend"
        0x52 => Op::End,
        // "wait"
        0x38 => Op::Wait {
            steps: data.read_u8().unwrap(),
        },
        // "timer"
        0x3A => Op::Timer {
            value: data.read_u8().unwrap(),
        },

        // Sound / music.
        // "effect1"
        0x3B => Op::PlaySound1 {
            sound: data.read_u8().unwrap(),
            position: data.read_u8().unwrap(),
        },
        // "effect2"
        0x3C => Op::PlaySound2 {
            sound: data.read_u8().unwrap(),
            position: data.read_u8().unwrap(),
        },
        // "sound"
        0x3D => Op::PlayMusic {
            music_index: data.read_u8().unwrap(),
        },
        // "s_sound"
        0x4A => Op::PlayMusicS {
            music_index: data.read_u8().unwrap(),
        },
        // "musiccmd"
        0x4B => Op::MusicCommand {
            flags1: data.read_u8().unwrap(),
            music_index: data.read_u8().unwrap(),
            flags2: data.read_u8().unwrap(),
            extra: data.read_u8().unwrap(),
        },

        // Exits/scripted exits.
        // "wake"
        0x44 => Op::ExitOpen {
            address: data.read_u16::<LittleEndian>().unwrap(),
        },
        // "sleep"
        0x45 => Op::ExitClose {
            address: data.read_u16::<LittleEndian>().unwrap(),
        },

        // DS/PC specific ops.
        // "moveEX"
        0x53 => Op::MoveExtended {
            i0: data.read_u8().unwrap(),
            i1: data.read_u8().unwrap(),
            i2: data.read_u8().unwrap(),
        },
        // "palEX"
        0x54 => Op::PaletteExtended {
            i0: data.read_u8().unwrap(),
            i1: data.read_u8().unwrap(),
            i2: data.read_u8().unwrap(),
            i3: data.read_u8().unwrap(),
        },

        _ => {
            println!("Unimplemented opcode 0x{:02X}", op_byte);
            return None
        },
    };

    Some(op)
}
