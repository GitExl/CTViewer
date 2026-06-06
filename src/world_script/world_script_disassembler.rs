use std::collections::{HashMap, HashSet};
use std::io::Cursor;
use crate::GameMode;
use crate::music_list::get_music_title;
use crate::sound_list::get_sound_name;
use crate::world_script::world_action_funcs::action_func_as_string;
use crate::world_script::world_script_decoder::op_decode;
use crate::world_script::world_script_ops::Op;
use crate::world_script::world_animation_script::get_animation_description;

pub struct WorldScriptDisassembler {
    data: Cursor<Vec<u8>>,
    ops: Vec<Op>,
    mode: GameMode,
    labels: HashMap<u64, HashSet<String>>,
}

impl WorldScriptDisassembler {
    pub fn new(data: &Vec<u8>, mode: GameMode) -> WorldScriptDisassembler {
        WorldScriptDisassembler {
            data: Cursor::new(data.clone()),
            ops: vec![],
            mode,
            labels: HashMap::new(),
        }
    }

    pub fn disassemble(&mut self) {
        let data_len = self.data.get_ref().len() as u64;

        let mut op_address = 0;
        self.data.set_position(0);
        while op_address < data_len {

            // Decode.
            let op = op_decode(&mut self.data, self.mode);
            if let Some(op) = op {
                self.ops.push(op);

                // Generate labels.
                match op {
                    Op::AddActor { address, .. } => self.add_label(address, format!("actor_{:04X}", address)),
                    Op::AddActorSpecial { address, .. } => self.add_label(address, format!("actor_special_{:04X}", address)),
                    Op::Bind { address, .. } => self.add_label(address, format!("pc_{:04X}", address)),
                    Op::DecrementAndJumpIfNonZero { offset, .. } => self.add_label((op_address as i64 + offset) as u64, format!("jpnz_{:04X}", op_address as i64 + offset)),
                    Op::GoTo { address } => self.add_label(address, format!("jp_{:04X}", address)),
                    Op::JumpConditional { offset, .. } => self.add_label((op_address as i64 + offset) as u64, format!("jp_{:04X}", op_address as i64 + offset)),
                    Op::GoSub { address } => self.add_label(address, format!("sub_{:04X}", address)),
                    _ => {}
                };
            }

            op_address = self.data.position();
        }
    }

    fn add_label(&mut self, address: u64, label: String) {
        if !self.labels.contains_key(&address) {
            self.labels.insert(address, HashSet::new());
        }
        self.labels.get_mut(&address).unwrap().insert(label);
    }

    pub fn dump(&mut self) {
        let data_len = self.data.get_ref().len() as u64;
        let mut op_address = 0;

        self.data.set_position(0);
        while op_address < data_len {

            // Output generated labels.
            if self.labels.contains_key(&op_address) {
                println!();
                for label in self.labels[&op_address].iter() {
                    println!("{:04X} {}:", op_address, label);
                }
            }

            // Output text for op.
            let op = op_decode(&mut self.data, self.mode);
            if let Some(op) = op {
                let statement = match op {
                    Op::AddActor { address, unused } => format!("add_actor actor_{:04X}, {}", address, unused),
                    Op::AddActorSpecial { address, i0 } => format!("add_special_actor actor_special_{:04X}, {}", address, i0),
                    Op::WaitAndAnimate { steps: delay } => format!("wait_animate {}", delay),
                    Op::Bind { address, pc } => format!("bind_pc pc_{:04X}, {}", address, pc_index(pc)),
                    Op::BitMath { dest, lhs, op, rhs } => format!("{} = {} {} {}", dest.as_string(), lhs.as_string(), op.as_string(), rhs.as_string()),
                    Op::ByteMath { dest, lhs, op, rhs } => format!("{} = {} {} {}", dest.as_string(), lhs.as_string(), op.as_string(), rhs.as_string()),
                    Op::GoSub { address } => format!("gosub sub_{:04X}", address),
                    Op::CallFunctionFar { address } => format!("function_far 0x{:06X}", address),
                    Op::ChangeLocation { destination } => format!("location {}", destination.as_string()),
                    Op::Copy8 { lhs, rhs } => format!("{} = {}", lhs.as_string(), rhs.as_string()),
                    Op::CopyTiles { source_layer, source_x, source_y, dest_layer, dest_x, dest_y, width, height } => {
                        format!("copy_tiles {}, ({}, {}), {}, ({}, {}), ({}, {})", source_layer, source_x, source_y, dest_layer, dest_x, dest_y, width, height)
                    }
                    Op::DecrementAndJumpIfNonZero { src, offset, .. } => {
                        format!("if --{} != 0 goto jpnz_{:04X}", src.as_string(), op_address as i64 + offset)
                    },
                    Op::End => String::from("end"),
                    Op::FadeIn { delay: mode } => format!("fade_in {}", mode),
                    Op::FadeOut { delay: mode } => format!("fade_out {}", mode),
                    Op::CallFunction { address } => format!("function 0xC2{:04X}    // Function: {}", address, action_func_as_string(address)),
                    Op::InitBackgroundLayer { layer } => format!("init_bg_layer {}", layer),
                    Op::InitMemory => String::from("init_memory"),
                    Op::GoTo { address } => format!("goto jp_{:04X}", address),
                    Op::JumpConditional { lhs, cmp, rhs, offset } => {
                        format!("if {} {} {} goto jp_{:04X}", lhs.as_string(), cmp.as_string(), rhs.as_string(), op_address as i64 + offset)
                    },
                    Op::Link { address } => format!("link 0xC2{:04X}    // Action: {}", address, action_func_as_string(address)),
                    Op::LinkSpecial { address } => format!("link_special 0xC2{:04X}    // Action: {}", address, action_func_as_string(address)),
                    Op::MosaicIn { mode } => format!("mosaic_in {}", mode),
                    Op::MosaicOut { mode } => format!("mosaic_out {}", mode),
                    Op::Move { steps } => format!("move {}", steps),
                    Op::MoveExtended { i0, i1, i2 } => format!("move_ext {}, {}, {}", i0, i1, i2),
                    Op::MusicCommand { music_index, flags1, flags2, extra } => {
                        format!("music_cmd {}, {}, {}, {}    // Music: {}", music_index, flags1, flags2, extra, get_music_title(music_index))
                    },
                    Op::PaletteExtended { i0, i1, i2, i3 } => format!("palette_ext {}, {}, {}, {}", i0, i1, i2, i3),
                    Op::PlayMusic { music_index } => format!("play_music_keep {}    // Music: {}", music_index, get_music_title(music_index)),
                    Op::PlayMusicS { music_index } => format!("play_music {}    // Music: {}", music_index, get_music_title(music_index)),
                    Op::PlaySound1 { sound, position } => format!("play_sound1 {}, {}    // Sound: {}", sound, position, get_sound_name(sound)),
                    Op::PlaySound2 { sound, position } => format!("play_sound2 {}, {}    // Sound: {}", sound, position, get_sound_name(sound)),
                    Op::Return => String::from("return"),
                    Op::Scroll { steps } => format!("scroll {}", steps),
                    Op::ScrollLayer { layer, steps } => format!("scroll_layer {}, {}", layer, steps),
                    Op::SetAnimation { anim_index } => format!("set_animation {}    // Animation: {}", anim_index, get_animation_description(anim_index)),
                    Op::SetPosition { x, y } => format!("set_pos {}, {}", x, y),
                    Op::SetPalette { index } => format!("set_palette {}", index),
                    Op::SetPriority { priority } => format!("set_priority {}", priority),
                    Op::SetTile { layer, x, y, tile_index } => format!("set_tile {}, ({}, {}), {}", layer, x, y, tile_index),
                    Op::SetTileR { layer, x, y, tile_index } => format!("set_tile_r {}, ({}, {}), {}", layer, x, y, tile_index),
                    Op::ExitClose { address } => format!("exit_close 0x{:04X}", address),
                    Op::VectorX { magnitude } => format!("vector_x {:.03}", (magnitude as f64) / 65536.0),
                    Op::VectorY { magnitude } => format!("vector_y {:.03}", (magnitude as f64) / 65536.0),
                    Op::Timer { value } => format!("timer {}", value),
                    Op::MoveToX { steps, animation1, animation2 } => {
                        format!("move_to_x {}, {}, {}    // Move left anim: {}, move right anim: {}", steps, animation1, animation2, get_animation_description(animation1), get_animation_description(animation2))
                    },
                    Op::MoveToY { steps, animation1, animation2 } => {
                        format!("move_to_y {}, {}, {}    // Move left anim: {}, move right anim: {}", steps, animation1, animation2, get_animation_description(animation1), get_animation_description(animation2))
                    },
                    Op::Unknown03 { i0, i1, i2, i3, i4, i5, i6, i7, i8 } => {
                        format!("unknown03 {}, {}, {}, {}, {}, {}, {}, {}, {}", i0, i1, i2, i3, i4, i5, i6, i7, i8)
                    },
                    Op::PaletteLoad { address, palette_index: count, mode } => format!("palette_load 0x{:04X}, {}, {}", address, count, mode),
                    Op::BgAnimate { i0, i1, i2, i3 } => format!("bg_anim {}, {}, {}, {}", i0, i1, i2, i3),
                    Op::Wait { steps } => format!("wait {}", steps),
                    Op::ExitOpen { address } => format!("exit_open 0x{:04X}", address),
                };

                println!("{:04X}   {}", op_address, statement);
            }

            op_address = self.data.position();
        }
    }
}

fn pc_index(index: u8) -> String {
    format!("PC{:02}", index)
}
