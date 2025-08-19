use std::io::Cursor;
use byteorder::ReadBytesExt;
use crate::scene::scene_script::{CallWaitMode, Op};

pub fn op_decode(data: &mut Cursor<Vec<u8>>) -> Op {
    let op_byte = data.read_u8().unwrap();

    match op_byte {

        // Yield to the function with the next higher priority number.
        // If there is none, simply yield.
        0x00 => Op::Yield,

        // Call function on actor.
        // If current function priority < priority
        //   Saves the current execution position
        //   Calls function in priority slot
        // If current function priority == priority
        //   Do nothing
        // If current function priority > priority
        //   Set the function exit address (?) to point to the function
        //   Only if the exit address is not defined yet.
        // Note that higher priority number == less urgent.
        0x01 => {
            let actor_index = data.read_u8().unwrap() as usize;
            let bits = data.read_u8().unwrap();
            Op::Call {
                actor_index,
                function_index: (bits & 0x0F) as usize,
                priority: (bits & 0xF0) as usize >> 4,
                wait_mode: CallWaitMode::NoWait,
            }
        },
        // Wait until the other actor completes a more urgent task, then call as in 0x01.
        0x02 => {
            let actor_index = data.read_u8().unwrap() as usize;
            let bits = data.read_u8().unwrap();
            Op::Call {
                actor_index,
                function_index: (bits & 0x0F) as usize,
                priority: (bits & 0xF0) as usize >> 4,
                wait_mode: CallWaitMode::WaitForCompletion,
            }
        },
        // Wait until the other actor completes a more urgent task, then call as in 0x01,
        // then wait until that function completes.
        0x03 => {
            let actor_index = data.read_u8().unwrap() as usize;
            let bits = data.read_u8().unwrap();
            Op::Call {
                actor_index,
                function_index: (bits & 0x0F) as usize,
                priority: (bits & 0xF0) as usize >> 4,
                wait_mode: CallWaitMode::WaitForReturn,
            }
        },

        _ => {
            println!("Unimplemented op: 0x{:02X}", op_byte);
            Op::NOP
        }
    }
}
