use std::io::Cursor;
use byteorder::ReadBytesExt;
use crate::scene_script::ops::Op;

pub fn op_decode_audio(op: u8, data: &mut Cursor<Vec<u8>>) -> Op {
    match op {
        0xE8 => Op::SoundPlay {
            sound: data.read_u8().unwrap() as usize,
            panning: 0.5,
        },
        0xEA => Op::MusicPlay {
            music: data.read_u8().unwrap() as usize,
            interrupt: true,
        },
        0xEB => Op::SoundVolumeSlide {
            left: data.read_u8().unwrap() as f64 * (1.0 / 255.0),
            right: data.read_u8().unwrap() as f64 * (1.0 / 255.0),
            duration: 0.0,
        },
        0xEC => {
            let mode = data.read_u8().unwrap();
            let data1 = data.read_u8().unwrap();
            let data2 = data.read_u8().unwrap();
            if mode == 0x11 {
                Op::MusicPlay {
                    music: data1 as usize,
                    interrupt: true,
                }
            } else if mode == 0x14 {
                Op::MusicPlay {
                    music: data1 as usize,
                    interrupt: false,
                }
            } else if mode == 0x18 || mode == 0x19 {
                Op::SoundPlay {
                    sound: data1 as usize,
                    panning: data2 as f64 * (1.0 / 255.0),
                }
            } else if mode == 0x82 {
                Op::MusicVolumeSlide {
                    duration: data1 as f64 * (1.0 / 60.0),
                    volume: data2 as f64 / (1.0 / 255.0),
                }
            } else if mode == 0x85 || mode == 0x86 {
                Op::MusicTempoSlide {
                    duration: data1 as f64 * (1.0 / 60.0),
                    tempo: data2,
                }
            } else if mode == 0xF0 {
                Op::MusicVolumeSlide {
                    duration: 0.0,
                    volume: 0.0,
                }
            } else if mode == 0xF2 {
                Op::SoundVolumeSlide {
                    left: 0.0,
                    right: 0.0,
                    duration: 0.0,
                }
            } else {
                Op::Unknown {
                    code: 0x83,
                    data: [data1, data2, 0, 0]
                }
            }
        },
        0xED => Op::SoundWaitEnd,
        0xEE => Op::MusicWaitEnd,

        _ => panic!("Unknown audio op."),
    }
}
