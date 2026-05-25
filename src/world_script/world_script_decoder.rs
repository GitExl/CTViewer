use std::io::{Cursor};
use byteorder::{LittleEndian, ReadBytesExt};
use crate::shared_op::{BitMathOp, ByteMathOp, CompareOp};
use crate::memory::{DataDest, DataSource};
use crate::world_script::ops::Op;
use crate::world_script::world_script::WorldScriptMode;

/// Opcodes.
pub fn op_decode(data: &mut Cursor<Vec<u8>>, mode: WorldScriptMode) -> Option<Op> {
    let op_byte = match data.read_u8() {
        Ok(op_byte) => op_byte,
        Err(_) => {
            println!("Script execution past end of data at 0x{:04X}.", data.position());
            return None;
        }
    };

    let op = match op_byte {

        // "initialize"
        0x00 => Op::InitMemory,
        // "colofs"
        0x01 => Op::SetPalette {
            index: data.read_u8().unwrap(),
        },
        // "priset"
        0x02 => Op::SetPriority {
            priority: data.read_u8().unwrap(),
        },
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
        // "pal", source address?, number of colors?, unknown?
        0x04 => Op::Unknown04 {
            address: read_24_bit_address(data),
            i0: data.read_u8().unwrap(),
            i1: data.read_u8().unwrap(),
        },
        // "mapjump"
        0x05 => {
            match mode {
                WorldScriptMode::Snes => Op::ChangeLocation {
                    location: data.read_u16::<LittleEndian>().unwrap(),
                    i0: 0,
                    x: data.read_u8().unwrap(),
                    y: data.read_u8().unwrap(),
                },
                WorldScriptMode::Pc => Op::ChangeLocation {
                    location: data.read_u16::<LittleEndian>().unwrap(),
                    i0: data.read_u8().unwrap(),
                    x: data.read_u8().unwrap(),
                    y: data.read_u8().unwrap(),
                },
            }
        },
        // "putmap"
        0x07 => Op::SetTile {
            layer: data.read_u8().unwrap(),
            x: data.read_u8().unwrap(),
            y: data.read_u8().unwrap(),
            tile_index: data.read_u8().unwrap(),
        },
        // "bind"
        0x08 => Op::Bind {
            address: data.read_u16::<LittleEndian>().unwrap() - 0x400,
            pc: data.read_u8().unwrap(),
        },
        // "newevent"
        0x09 => Op::AddActor {
            address: data.read_u16::<LittleEndian>().unwrap() - 0x400,
            i0: data.read_u8().unwrap(),
        },

        // "clr"
        0x0A => Op::Copy8 {
            lhs: DataDest::WorldLocal(data.read_u8().unwrap() as usize),
            rhs: DataSource::Immediate(0),
        },
        // "incr"
        0x0B => Op::ByteMath {
            lhs: DataDest::WorldLocal(data.read_u8().unwrap() as usize),
            rhs: DataSource::Immediate(1),
            op: ByteMathOp::Add,
        },
        // "decr"
        0x0C => Op::ByteMath {
            lhs: DataDest::WorldLocal(data.read_u8().unwrap() as usize),
            rhs: DataSource::Immediate(1),
            op: ByteMathOp::Subtract,
        },
        // "setr"
        0x0D => Op::Copy8 {
            lhs: DataDest::WorldLocal(data.read_u8().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
        },
        // "bitsetr"
        0x0E => Op::BitMath {
            lhs: DataDest::WorldLocal(data.read_u8().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            op: BitMathOp::Or,
        },
        // "bitclr"
        0x0F => Op::BitMath {
            lhs: DataDest::WorldLocal(data.read_u8().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            op: BitMathOp::Xor,
        },

        // "memclr"
        0x10 => Op::Copy8 {
            lhs: DataDest::Memory(0x7E000 + data.read_u16::<LittleEndian>().unwrap() as usize),
            rhs: DataSource::Immediate(0),
        },
        // "meminc"
        0x11 => Op::ByteMath {
            lhs: DataDest::Memory(0x7E000 + data.read_u16::<LittleEndian>().unwrap() as usize),
            rhs: DataSource::Immediate(1),
            op: ByteMathOp::Add,
        },
        // "memdec"
        0x12 => Op::ByteMath {
            lhs: DataDest::Memory(0x7E000 + data.read_u16::<LittleEndian>().unwrap() as usize),
            rhs: DataSource::Immediate(1),
            op: ByteMathOp::Subtract,
        },
        // "memset"
        0x13 => Op::Copy8 {
            lhs: DataDest::Memory(0x7E000 + data.read_u16::<LittleEndian>().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
        },
        // "membitset"
        0x14 => Op::BitMath {
            lhs: DataDest::Memory(0x7E000 + data.read_u16::<LittleEndian>().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            op: BitMathOp::Or,
        },
        // "membitclr"
        0x15 => Op::BitMath {
            lhs: DataDest::Memory(0x7E000 + data.read_u16::<LittleEndian>().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            op: BitMathOp::Xor,
        },

        // "trnlg"
        0x16 => Op::Copy8 {
            lhs: DataDest::Memory(0x7E000 + data.read_u16::<LittleEndian>().unwrap() as usize),
            rhs: DataSource::WorldLocal(data.read_u8().unwrap() as usize),
        },
        // "trngl"
        0x17 => Op::Copy8 {
            lhs: DataDest::WorldLocal(data.read_u8().unwrap() as usize),
            rhs: DataSource::Memory(0x7E000 + data.read_u16::<LittleEndian>().unwrap() as usize),
        },

        // "addr"
        0x46 => Op::ByteMath {
            lhs: DataDest::WorldLocal(data.read_u8().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            op: ByteMathOp::Add,
        },
        // "subr"
        0x47 => Op::ByteMath {
            lhs: DataDest::WorldLocal(data.read_u8().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            op: ByteMathOp::Subtract,
        },
        // "memadd"
        0x48 => Op::ByteMath {
            lhs: DataDest::Memory(0x7E000 + data.read_u16::<LittleEndian>().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            op: ByteMathOp::Add,
        },
        // "memsub"
        0x49 => Op::ByteMath {
            lhs: DataDest::Memory(0x7E000 + data.read_u16::<LittleEndian>().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            op: ByteMathOp::Subtract,
        },

        // "jp"
        0x1A => Op::Jump {
            address: data.read_u16::<LittleEndian>().unwrap() as usize - 0x400,
        },
        // "jdjnz"
        0x1B => Op::DecrementAndJumpIfNonZero {
            address: DataDest::WorldLocal(data.read_u8().unwrap() as usize),
            offset: data.read_i8().unwrap() as isize,
        },

        // "jz", unused
        0x1C => Op::JumpConditional {
            lhs: DataSource::WorldLocal(data.read_u8().unwrap() as usize),
            rhs: DataSource::Immediate(0),
            cmp: CompareOp::Eq,
            offset: data.read_i8().unwrap() as isize,
        },
        // "jnz", unused
        0x1D => Op::JumpConditional {
            lhs: DataSource::WorldLocal(data.read_u8().unwrap() as usize),
            rhs: DataSource::Immediate(0),
            cmp: CompareOp::NotEq,
            offset: data.read_i8().unwrap() as isize,
        },
        // "jcpnz"
        0x1E => Op::JumpConditional {
            lhs: DataSource::WorldLocal(data.read_u8().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            cmp: CompareOp::NotEq,
            offset: data.read_i8().unwrap() as isize,
        },
        // "jcpz"
        0x1F => Op::JumpConditional {
            lhs: DataSource::WorldLocal(data.read_u8().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            cmp: CompareOp::Eq,
            offset: data.read_i8().unwrap() as isize,
        },
        // "jandnz"
        0x20 => Op::JumpConditional {
            lhs: DataSource::WorldLocal(data.read_u8().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            cmp: CompareOp::And,
            offset: data.read_i8().unwrap() as isize,
        },
        // "jandz"
        0x21 => Op::JumpConditional {
            lhs: DataSource::WorldLocal(data.read_u8().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            cmp: CompareOp::Or,
            offset: data.read_i8().unwrap() as isize,
        },
        // "jz_g"
        0x22 => Op::JumpConditional {
            lhs: DataSource::Memory(0x7E000 + data.read_u16::<LittleEndian>().unwrap() as usize),
            rhs: DataSource::Immediate(0),
            cmp: CompareOp::Eq,
            offset: data.read_i8().unwrap() as isize,
        },
        // "jnz_g"
        0x23 => Op::JumpConditional {
            lhs: DataSource::Memory(0x7E000 + data.read_u16::<LittleEndian>().unwrap() as usize),
            rhs: DataSource::Immediate(0),
            cmp: CompareOp::NotEq,
            offset: data.read_i8().unwrap() as isize,
        },
        // "jcpnz_g"
        0x24 => Op::JumpConditional {
            lhs: DataSource::Memory(0x7E000 + data.read_u16::<LittleEndian>().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            cmp: CompareOp::NotEq,
            offset: data.read_i8().unwrap() as isize,
        },
        // "jcpz_g"
        0x25 => Op::JumpConditional {
            lhs: DataSource::Memory(0x7E000 + data.read_u16::<LittleEndian>().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            cmp: CompareOp::Eq,
            offset: data.read_i8().unwrap() as isize,
        },
        // "jandnz_g"
        0x26 => Op::JumpConditional {
            lhs: DataSource::Memory(0x7E000 + data.read_u16::<LittleEndian>().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            cmp: CompareOp::And,
            offset: data.read_i8().unwrap() as isize,
        },
        // "jandz_g"
        0x27 => Op::JumpConditional {
            lhs: DataSource::Memory(0x7E000 + data.read_u16::<LittleEndian>().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            cmp: CompareOp::Or,
            offset: data.read_i8().unwrap() as isize,
        },
        // "jcpcc"
        0x4C => Op::JumpConditional {
            lhs: DataSource::Memory(0x7E000 + data.read_u16::<LittleEndian>().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            cmp: CompareOp::Lt,
            offset: data.read_i8().unwrap() as isize,
        },
        // "jcpcs"
        0x4D => Op::JumpConditional {
            lhs: DataSource::Memory(0x7E000 + data.read_u16::<LittleEndian>().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            cmp: CompareOp::GtEq,
            offset: data.read_i8().unwrap() as isize,
        },

        // "fadeout"
        0x28 => Op::FadeOut {
            i0: data.read_u8().unwrap(),
        },
        // "fadein"
        0x29 => Op::FadeIn {
            i0: data.read_u8().unwrap(),
        },

        // "pos"
        0x2C => Op::SetCoordinates {
            x: data.read_u16::<LittleEndian>().unwrap(),
            y: data.read_u16::<LittleEndian>().unwrap(),
        },

        // "vecx"
        0x2E => Op::SpeedX {
            a: data.read_i16::<LittleEndian>().unwrap(),
            b: data.read_i16::<LittleEndian>().unwrap(),
        },
        // "vecy"
        0x2F => Op::SpeedY {
            a: data.read_i16::<LittleEndian>().unwrap(),
            b: data.read_i16::<LittleEndian>().unwrap(),
        },
        // "anmseq"
        0x30 => Op::SetAnimation {
            anim_index: data.read_u8().unwrap(),
        },
        // "move"
        0x31 => Op::Move {
            steps: data.read_u8().unwrap(),
        },
        // "scroll"
        0x32 => Op::Scroll {
            time: data.read_u8().unwrap(),
        },
        // "bganm"
        0x33 => Op::Unknown33 {
            i0: data.read_u8().unwrap(),
            i1: data.read_u16::<LittleEndian>().unwrap(),
            i2: data.read_u16::<LittleEndian>().unwrap(),
            i3: data.read_u16::<LittleEndian>().unwrap(),
        },
        // "func"
        0x34 => Op::Func {
            address: data.read_u16::<LittleEndian>().unwrap(),
        },
        // "link"
        0x35 => Op::Link {
            x: data.read_u8().unwrap(),
            y: data.read_u8().unwrap(),
        },
        // "call"
        0x36 => Op::Call {
            address: data.read_u16::<LittleEndian>().unwrap() - 0x400,
        },
        // "return"
        0x37 => Op::Return,
        // "wait"
        0x38 => Op::Wait {
            i0: data.read_u8().unwrap(),
        },
        // "anmwait"
        0x39 => Op::AnimWait {
            speed: data.read_u8().unwrap(),
        },

        // 0x3A unused, "timer"

        // "effect1"
        0x3B => Op::PlaySound1 {
            sound: data.read_u8().unwrap(),
            position: data.read_i8().unwrap(),
        },
        // "effect2"
        0x3C => Op::PlaySound2 {
            sound: data.read_u8().unwrap(),
            position: data.read_i8().unwrap(),
        },
        // "sound"
        0x3D => Op::PlayMusic {
            music_index: data.read_u8().unwrap(),
        },

        // "initscreen"
        0x3E => Op::InitLayers {
            i0: data.read_u8().unwrap(),
        },

        // "tpxmove"
        0x3F => Op::TpMoveX {
            i0: data.read_u16::<LittleEndian>().unwrap(),
            i1: data.read_u16::<LittleEndian>().unwrap(),
        },
        // "tpymove"
        0x40 => Op::TpMoveY {
            i0: data.read_u16::<LittleEndian>().unwrap(),
            i1: data.read_u16::<LittleEndian>().unwrap(),
        },

        // 0x41 unused, "trigger"
        // 0x42 unused, "slink"

        // "s_newevent"
        0x43 => Op::AddActorS {
            address: data.read_u16::<LittleEndian>().unwrap() - 0x400,
            i0: data.read_u8().unwrap(),
        },

        // "wake"
        0x44 => Op::Wake {
            address: data.read_u16::<LittleEndian>().unwrap(),
        },
        // "sleep"
        0x45 => Op::Sleep {
            address: data.read_u16::<LittleEndian>().unwrap(),
        },

        // "s_sound"
        0x4A => Op::PlayMusicS {
            music_index: data.read_u8().unwrap(),
        },
        // "musiccmd"
        0x4B => Op::MusicCommand {
            i0: data.read_u8().unwrap(),
            i1: data.read_u8().unwrap(),
            i2: data.read_u8().unwrap(),
            i3: data.read_u8().unwrap(),
        },

        // "func2"
        0x4E => Op::CallFar {
            address: read_24_bit_address(data),
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
        // TODO: are the arguments correct?
        0x50 => Op::SetTileR {
            layer: data.read_u8().unwrap(),
            x: data.read_u8().unwrap(),
            y: data.read_u8().unwrap(),
            tile_index: data.read_u8().unwrap(),
        },
        // "scrollr"
        0x51 => Op::ScrollR {
            i0: data.read_u8().unwrap(),
            i1: data.read_u8().unwrap(),
        },

        // "taskend"
        0x52 => Op::End,

        // DS/PC extra ops.
        // "moveEX"
        // TODO: is the argument count correct? One path seems to have variable number of arguments.
        0x53 => Op::MoveExtended {
            i0: data.read_u8().unwrap(),
            i1: data.read_u8().unwrap(),
            i2: data.read_u8().unwrap(),
        },
        // "palEX"
        // TODO: is the argument count correct?
        0x54 => Op::PaletteExtended {
            i0: data.read_u8().unwrap(),
            i1: data.read_u8().unwrap(),
            i2: data.read_u8().unwrap(),
            i3: data.read_u8().unwrap(),
        },

        _ => {
            println!("Decoding unimplemented opcode 0x{:02X} as NOP", op_byte);
            Op::NOP
        },
    };

    Some(op)
}

fn read_24_bit_address(data: &mut Cursor<Vec<u8>>) -> usize {
    data.read_u8().unwrap() as usize |
        (data.read_u8().unwrap() as usize) << 8 |
        (data.read_u8().unwrap() as usize) << 16
}
