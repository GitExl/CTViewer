use std::io::Cursor;
use byteorder::ReadBytesExt;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum CallWaitMode {
    NoWait,
    WaitForCompletion,
    WaitForReturn,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ActorRef {
    Actor { index: usize },
    PartyMember { index: usize },
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Op {
    NOP,
    Yield,
    Call {
        actor: ActorRef,
        priority: usize,
        function_index: usize,
        wait_mode: CallWaitMode,
    }
}

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
        0x02 => {
            let actor_index = data.read_u8().unwrap() as usize / 2;
            let bits = data.read_u8().unwrap();
            Op::Call {
                actor: ActorRef::Actor { index: actor_index },
                function_index: (bits & 0x0F) as usize,
                priority: (bits & 0xF0) as usize >> 4,
                wait_mode: CallWaitMode::NoWait,
            }
        },
        // Wait until the other actor completes a more urgent task, then call as in 0x02.
        0x03 => {
            let actor_index = data.read_u8().unwrap() as usize / 2;
            let bits = data.read_u8().unwrap();
            Op::Call {
                actor: ActorRef::Actor { index: actor_index },
                function_index: (bits & 0x0F) as usize,
                priority: (bits & 0xF0) as usize >> 4,
                wait_mode: CallWaitMode::WaitForCompletion,
            }
        },
        // Wait until the other actor completes a more urgent task, then call as in 0x02,
        // then wait until that function completes.
        0x04 => {
            let actor_index = data.read_u8().unwrap() as usize / 2;
            let bits = data.read_u8().unwrap();
            Op::Call {
                actor: ActorRef::Actor { index: actor_index },
                function_index: (bits & 0x0F) as usize,
                priority: (bits & 0xF0) as usize >> 4,
                wait_mode: CallWaitMode::WaitForReturn,
            }
        },

        // Same as 0x02, 0x03 and 0x04, but calls the actor of a specific party member.
        0x05 => {
            let party_member_index = data.read_u8().unwrap() as usize / 2;
            let bits = data.read_u8().unwrap();
            Op::Call {
                actor: ActorRef::PartyMember { index: party_member_index },
                function_index: (bits & 0x0F) as usize,
                priority: (bits & 0xF0) as usize >> 4,
                wait_mode: CallWaitMode::NoWait,
            }
        },
        0x06 => {
            let party_member_index = data.read_u8().unwrap() as usize / 2;
            let bits = data.read_u8().unwrap();
            Op::Call {
                actor: ActorRef::PartyMember { index: party_member_index },
                function_index: (bits & 0x0F) as usize,
                priority: (bits & 0xF0) as usize >> 4,
                wait_mode: CallWaitMode::WaitForCompletion,
            }
        },
        0x07 => {
            let party_member_index = data.read_u8().unwrap() as usize / 2;
            let bits = data.read_u8().unwrap();
            Op::Call {
                actor: ActorRef::PartyMember { index: party_member_index },
                function_index: (bits & 0x0F) as usize,
                priority: (bits & 0xF0) as usize >> 4,
                wait_mode: CallWaitMode::WaitForReturn,
            }
        },

        _ => {
            println!("Decoding unimplemented opcode 0x{:02X} as NOP", op_byte);
            Op::NOP
        }
    }
}
