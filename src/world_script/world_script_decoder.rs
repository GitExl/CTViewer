use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use crate::destination::Destination;
use crate::GameMode;
use crate::shared_op::{BitMathOp, ByteMathOp, CompareOp};
use crate::memory::{DataDest, DataSource};
use crate::util::data_read::read_24_bit_address;
use crate::world_script::function_dispatch::WorldActorFunction;
use crate::world_script::task_dispatch::WorldActorTask;
use crate::world_script::world_script_ops::Op;

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
        0x0B => {
            let lhs_address = data.read_u8().unwrap() as usize;
            Op::ByteMath {
                dest:  DataDest::for_world_actor_memory(lhs_address),
                lhs: DataSource::for_world_actor_memory(lhs_address),
                rhs: DataSource::Immediate(1),
                op: ByteMathOp::Add,
            }
        }
        // "decr"
        0x0C => {
            let lhs_address = data.read_u8().unwrap() as usize;
            Op::ByteMath {
                dest:  DataDest::for_world_actor_memory(lhs_address),
                lhs: DataSource::for_world_actor_memory(lhs_address),
                rhs: DataSource::Immediate(1),
                op: ByteMathOp::Subtract,
            }
        },
        // "setr"
        0x0D => Op::Copy8 {
            lhs: DataDest::for_world_actor_memory(data.read_u8().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
        },
        // "bitsetr"
        0x0E => {
            let lhs_address = data.read_u8().unwrap() as usize;
            let rhs_address = data.read_u8().unwrap() as i32;
            Op::BitMath {
                dest: DataDest::for_world_actor_memory(lhs_address),
                lhs: DataSource::for_world_actor_memory(lhs_address),
                rhs: DataSource::Immediate(rhs_address),
                op: BitMathOp::Or,
            }
        },
        // "bitclr"
        0x0F => {
            let lhs_address = data.read_u8().unwrap() as usize;
            let rhs_address = data.read_u8().unwrap() as i32;
            Op::BitMath {
                dest: DataDest::for_world_actor_memory(lhs_address),
                lhs: DataSource::for_world_actor_memory(lhs_address),
                rhs: DataSource::Immediate(rhs_address),
                op: BitMathOp::Xor,
            }
        },
        // "memclr"
        0x10 => Op::Copy8 {
            lhs: DataDest::for_system_memory(data.read_u16::<LittleEndian>().unwrap() as usize),
            rhs: DataSource::Immediate(0),
        },
        // "meminc"
        0x11 => {
            let lhs_address = data.read_u16::<LittleEndian>().unwrap() as usize;
            Op::ByteMath {
                dest:  DataDest::for_system_memory(lhs_address),
                lhs: DataSource::for_system_memory(lhs_address),
                rhs: DataSource::Immediate(1),
                op: ByteMathOp::Add,
            }
        },
        // "memdec"
        0x12 => {
            let lhs_address = data.read_u16::<LittleEndian>().unwrap() as usize;
            Op::ByteMath {
                dest:  DataDest::for_system_memory(lhs_address),
                lhs: DataSource::for_system_memory(lhs_address),
                rhs: DataSource::Immediate(1),
                op: ByteMathOp::Subtract,
            }
        },
        // "memset"
        0x13 => Op::Copy8 {
            lhs: DataDest::for_system_memory(data.read_u16::<LittleEndian>().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
        },
        // "membitset"
        0x14 => {
            let lhs_address = data.read_u16::<LittleEndian>().unwrap() as usize;
            let rhs_address = data.read_u8().unwrap() as i32;
            Op::BitMath {
                dest: DataDest::for_system_memory(lhs_address),
                lhs: DataSource::for_system_memory(lhs_address),
                rhs: DataSource::Immediate(rhs_address),
                op: BitMathOp::Or,
            }
        },
        // "membitclr"
        0x15 => {
            let lhs_address = data.read_u16::<LittleEndian>().unwrap() as usize;
            let rhs_address = data.read_u8().unwrap() as i32;
            Op::BitMath {
                dest: DataDest::for_system_memory(lhs_address),
                lhs: DataSource::for_system_memory(lhs_address),
                rhs: DataSource::Immediate(rhs_address),
                op: BitMathOp::Xor,
            }
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
        0x46 => {
            let lhs_address = data.read_u8().unwrap() as usize;
            Op::ByteMath {
                dest: DataDest::for_world_actor_memory(lhs_address),
                lhs: DataSource::for_world_actor_memory(lhs_address),
                rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
                op: ByteMathOp::Add,
            }
        },
        // "subr"
        0x47 => {
            let lhs_address = data.read_u8().unwrap() as usize;
            Op::ByteMath {
                dest: DataDest::for_world_actor_memory(lhs_address),
                lhs: DataSource::for_world_actor_memory(lhs_address),
                rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
                op: ByteMathOp::Subtract,
            }
        },
        // "memadd"
        0x48 => {
            let lhs_address = data.read_u16::<LittleEndian>().unwrap() as usize;
            Op::ByteMath {
                dest: DataDest::for_system_memory(lhs_address),
                lhs: DataSource::for_system_memory(lhs_address),
                rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
                op: ByteMathOp::Add,
            }
        },
        // "memsub"
        0x49 => {
            let lhs_address = data.read_u16::<LittleEndian>().unwrap() as usize;
            Op::ByteMath {
                dest: DataDest::for_system_memory(lhs_address),
                lhs: DataSource::for_system_memory(lhs_address),
                rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
                op: ByteMathOp::Subtract,
            }
        },

        // Jumps.
        // "jp"
        0x1A => Op::GoTo {
            address: data.read_u16::<LittleEndian>().unwrap() as u64 - 0x400 + 1,
        },
        // "jdjnz"
        0x1B => {
            let address = data.read_u8().unwrap() as usize;
            Op::DecrementAndJumpIfNonZero {
                src: DataSource::for_world_actor_memory(address),
                dest: DataDest::for_world_actor_memory(address),
                offset: data.read_i8().unwrap() as i64,
            }
        },
        // "jz"
        0x1C => Op::JumpConditional {
            lhs: DataSource::for_world_actor_memory(data.read_u8().unwrap() as usize),
            rhs: DataSource::Immediate(0),
            cmp: CompareOp::Eq,
            offset: data.read_i8().unwrap() as i64,
        },
        // "jnz"
        0x1D => Op::JumpConditional {
            lhs: DataSource::for_world_actor_memory(data.read_u8().unwrap() as usize),
            rhs: DataSource::Immediate(0),
            cmp: CompareOp::NotEq,
            offset: data.read_i8().unwrap() as i64,
        },
        // "jcpnz"
        0x1E => Op::JumpConditional {
            lhs: DataSource::for_world_actor_memory(data.read_u8().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            cmp: CompareOp::NotEq,
            offset: data.read_i8().unwrap() as i64,
        },
        // "jcpz"
        0x1F => Op::JumpConditional {
            lhs: DataSource::for_world_actor_memory(data.read_u8().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            cmp: CompareOp::Eq,
            offset: data.read_i8().unwrap() as i64,
        },
        // "jandnz"
        0x20 => Op::JumpConditional {
            lhs: DataSource::for_world_actor_memory(data.read_u8().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            cmp: CompareOp::And,
            offset: data.read_i8().unwrap() as i64,
        },
        // "jandz"
        0x21 => Op::JumpConditional {
            lhs: DataSource::for_world_actor_memory(data.read_u8().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            cmp: CompareOp::AndZero,
            offset: data.read_i8().unwrap() as i64,
        },
        // "jz_g"
        0x22 => Op::JumpConditional {
            lhs: DataSource::for_system_memory(data.read_u16::<LittleEndian>().unwrap() as usize),
            rhs: DataSource::Immediate(0),
            cmp: CompareOp::Eq,
            offset: data.read_i8().unwrap() as i64,
        },
        // "jnz_g"
        0x23 => Op::JumpConditional {
            lhs: DataSource::for_system_memory(data.read_u16::<LittleEndian>().unwrap() as usize),
            rhs: DataSource::Immediate(0),
            cmp: CompareOp::NotEq,
            offset: data.read_i8().unwrap() as i64,
        },
        // "jcpnz_g"
        0x24 => Op::JumpConditional {
            lhs: DataSource::for_system_memory(data.read_u16::<LittleEndian>().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            cmp: CompareOp::NotEq,
            offset: data.read_i8().unwrap() as i64,
        },
        // "jcpz_g"
        0x25 => Op::JumpConditional {
            lhs: DataSource::for_system_memory(data.read_u16::<LittleEndian>().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            cmp: CompareOp::Eq,
            offset: data.read_i8().unwrap() as i64,
        },
        // "jandnz_g"
        0x26 => Op::JumpConditional {
            lhs: DataSource::for_system_memory(data.read_u16::<LittleEndian>().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            cmp: CompareOp::And,
            offset: data.read_i8().unwrap() as i64,
        },
        // "jandz_g"
        0x27 => Op::JumpConditional {
            lhs: DataSource::for_system_memory(data.read_u16::<LittleEndian>().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            cmp: CompareOp::AndZero,
            offset: data.read_i8().unwrap() as i64,
        },
        // "jcpcc"
        0x4C => Op::JumpConditional {
            lhs: DataSource::for_system_memory(data.read_u16::<LittleEndian>().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            cmp: CompareOp::Lt,
            offset: data.read_i8().unwrap() as i64,
        },
        // "jcpcs"
        0x4D => Op::JumpConditional {
            lhs: DataSource::for_system_memory(data.read_u16::<LittleEndian>().unwrap() as usize),
            rhs: DataSource::Immediate(data.read_u8().unwrap() as i32),
            cmp: CompareOp::GtEq,
            offset: data.read_i8().unwrap() as i64,
        },

        // Reveal/show.
        // "fadeout"
        0x28 => Op::FadeOut {
            delay: data.read_u8().unwrap(),
        },
        // "fadein"
        0x29 => Op::FadeIn {
            delay: data.read_u8().unwrap(),
        },
        // "mozin"
        0x2A => Op::MosaicIn {
            mode: data.read_u16::<LittleEndian>().unwrap(),
        },
        // "mozout"
        0x2B => Op::MosaicOut {
            mode: data.read_u16::<LittleEndian>().unwrap(),
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
            anim_index: data.read_u8().unwrap() as usize,
        },
        // "anmwait"
        0x39 => Op::WaitAndAnimate {
            steps: data.read_u8().unwrap(),
        },

        // Map changes.
        // "putmap"
        0x07 => Op::SetTile {
            layer: data.read_u8().unwrap() as usize - 1,
            x: data.read_u8().unwrap() as usize,
            y: data.read_u8().unwrap() as usize,
            tile_index: data.read_u8().unwrap() as usize,
        },
        // "bganm"
        0x33 => {
            let bank = data.read_u8().unwrap() as u64;
            let local_address = data.read_u16::<LittleEndian>().unwrap() as u64;
            Op::CopyToVram {
                source_address: bank << 16 | local_address,
                vram_dest_address: data.read_u16::<LittleEndian>().unwrap(),
                byte_count: data.read_u16::<LittleEndian>().unwrap(),
            }
        },
        // "copymap"
        0x4F => Op::CopyTiles {
            source_layer: data.read_u8().unwrap() as usize - 1,
            source_x: data.read_u8().unwrap() as usize,
            source_y: data.read_u8().unwrap() as usize,
            dest_layer: data.read_u8().unwrap() as usize - 1,
            dest_x: data.read_u8().unwrap() as usize,
            dest_y: data.read_u8().unwrap() as usize,
            width: data.read_u8().unwrap() as usize,
            height: data.read_u8().unwrap() as usize,
        },
        // "putmapr"
        0x50 => Op::SetTileR {
            layer: data.read_u8().unwrap() - 1,
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
            layer: (data.read_u8().unwrap() - 1) as usize,
            steps: data.read_u8().unwrap(),
        },
        // "tpxmove"
        0x3F => Op::MoveToX {
            steps: data.read_u16::<LittleEndian>().unwrap(),
            animation1: data.read_u8().unwrap() as usize,
            animation2: data.read_u8().unwrap() as usize,
        },
        // "tpymove"
        0x40 => Op::MoveToY {
            steps: data.read_u16::<LittleEndian>().unwrap(),
            animation1: data.read_u8().unwrap() as usize,
            animation2: data.read_u8().unwrap() as usize,
        },

        // Function calls/new objects.
        // "bind"
        0x08 => Op::Bind {
            address: data.read_u16::<LittleEndian>().unwrap() as u64 - 0x400,
            pc: data.read_u8().unwrap(),
        },
        // "newevent"
        0x09 => Op::AddActor {
            address: data.read_u16::<LittleEndian>().unwrap() as u64 - 0x400,
            unused: data.read_u8().unwrap(),
        },
        // "func"
        0x34 => {
            let address = data.read_u16::<LittleEndian>().unwrap() as u32;
            Op::CallFunction {
                function: WorldActorFunction::from_address(address),
                address,
            }
        },
        // "link"
        0x35 => {
            let address = data.read_u16::<LittleEndian>().unwrap() as u32;
            Op::Link {
                task: WorldActorTask::from_address(address),
                address,
            }
        },
        // "call"
        0x36 => Op::GoSub {
            address: data.read_u16::<LittleEndian>().unwrap() as u64 - 0x400,
        },
        // "return"
        0x37 => Op::Return,
        // "slink"
        0x42 => {
            let address = data.read_u16::<LittleEndian>().unwrap() as u32;
            Op::LinkSpecial {
                task: WorldActorTask::from_address(address),
                address,
            }
        },
        // "s_newevent"
        0x43 => {
            Op::AddActorSpecial {
                address: data.read_u16::<LittleEndian>().unwrap() as u64 - 0x400,
                i0: data.read_u8().unwrap(),
            }
        },
        // "func2"
        0x4E => {
            let address = read_24_bit_address(data) as u32;
            Op::CallFunctionFar {
                function: WorldActorFunction::from_address(address),
                address,
            }
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
