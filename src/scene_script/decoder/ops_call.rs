use std::io::Cursor;
use byteorder::ReadBytesExt;
use crate::scene_script::ops::Op;
use crate::scene_script::scene_script_decoder::ActorRef;

pub fn op_decode_call(op: u8, data: &mut Cursor<Vec<u8>>) -> Op {
    match op {

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
                actor: ActorRef::ScriptActor(actor_index),
                function: (bits & 0x0F) as usize,
                priority: (bits & 0xF0) as usize >> 4,
            }
        },
        // Wait until the other actor completes a more urgent task, then call as in 0x02.
        0x03 => {
            let actor_index = data.read_u8().unwrap() as usize / 2;
            let bits = data.read_u8().unwrap();
            Op::CallWaitCompletion {
                actor: ActorRef::ScriptActor(actor_index),
                function: (bits & 0x0F) as usize,
                priority: (bits & 0xF0) as usize >> 4,
            }
        },
        // Wait until the other actor completes a more urgent task, then call as in 0x02,
        // then wait until that function completes.
        0x04 => {
            let actor_index = data.read_u8().unwrap() as usize / 2;
            let bits = data.read_u8().unwrap();
            Op::CallWaitReturn {
                actor: ActorRef::ScriptActor(actor_index),
                function: (bits & 0x0F) as usize,
                priority: (bits & 0xF0) as usize >> 4,
            }
        },

        // Same as 0x02, 0x03 and 0x04, but calls the actor of a specific party member.
        0x05 => {
            let party_member_index = data.read_u8().unwrap() as usize / 2;
            let bits = data.read_u8().unwrap();
            Op::Call {
                actor: ActorRef::PartyMember(party_member_index),
                function: (bits & 0x0F) as usize,
                priority: (bits & 0xF0) as usize >> 4,
            }
        },
        0x06 => {
            let party_member_index = data.read_u8().unwrap() as usize / 2;
            let bits = data.read_u8().unwrap();
            Op::CallWaitCompletion {
                actor: ActorRef::PartyMember(party_member_index),
                function: (bits & 0x0F) as usize,
                priority: (bits & 0xF0) as usize >> 4,
            }
        },
        0x07 => {
            let party_member_index = data.read_u8().unwrap() as usize / 2;
            let bits = data.read_u8().unwrap();
            Op::CallWaitReturn {
                actor: ActorRef::PartyMember(party_member_index),
                function: (bits & 0x0F) as usize,
                priority: (bits & 0xF0) as usize >> 4,
            }
        },

        _ => panic!("Unknown call op."),
    }
}
