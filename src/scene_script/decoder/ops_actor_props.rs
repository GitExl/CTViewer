use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use crate::actor::{ActorFlags, DrawMode};
use crate::scene_script::scene_script_ops::Op;
use crate::scene_script::scene_script_decoder::ActorRef;
use crate::memory::{DataDest, DataSource};
use crate::sprites::sprite_renderer::SpritePriority;

pub fn op_decode_actor_props(op: u8, data: &mut Cursor<Vec<u8>>) -> Op {
    match op {

        // Enable/disable function calls on this actor.
        // "lock"
        0x08 => Op::ActorUpdateFlags {
            actor: ActorRef::This,
            set: ActorFlags::CALLS_DISABLED,
            remove: ActorFlags::empty(),
        },
        // "unlock"
        0x09 => Op::ActorUpdateFlags {
            actor: ActorRef::This,
            set: ActorFlags::empty(),
            remove: ActorFlags::CALLS_DISABLED,
        },

        // Set actor result from 0x7F0200.
        // "switch"
        0x19 => Op::ActorSetResult8 {
            actor: ActorRef::This,
            result: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
        },
        // Set 16 bit actor result from 0x7F0000.
        // "gswitch"
        0x1C => Op::ActorSetResult16 {
            actor: ActorRef::This,
            result: DataSource::for_global_memory(data.read_u8().unwrap() as usize),
        },

        // Disable script processing and hide.
        // "kill"
        0x0A => Op::ActorRemove {
            actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
        },

        // Set drawing mode.
        // "dshow"
        0x7C => Op::ActorSetDrawMode {
            actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            draw_mode: DrawMode::Draw,
        },
        // "dhide"
        0x7D => Op::ActorSetDrawMode {
            actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            draw_mode: DrawMode::Hidden,
        },
        // "battlehide"
        // TODO: "removed" means only for battle?
        0x7E => Op::ActorSetDrawMode {
            actor: ActorRef::This,
            draw_mode: DrawMode::Removed,
        },
        // "show"
        0x90 => Op::ActorSetDrawMode {
            actor: ActorRef::This,
            draw_mode: DrawMode::Draw,
        },
        // "hide"
        0x91 => Op::ActorSetDrawMode {
            actor: ActorRef::This,
            draw_mode: DrawMode::Hidden,
        },

        // Sprite priority.
        // "pri"
        0x8E => {
            let bits = data.read_u8().unwrap();
            let set_and_lock = bits & 0x80 == 0;
            let top = bits & 0x3;
            let bottom = (bits & 0x30) >> 4;
            let unknown_bits = bits & 0x4C;

            Op::ActorSetSpritePriority {
                actor: ActorRef::This,
                top: SpritePriority::from_value(top),
                bottom: SpritePriority::from_value(bottom),
                set_and_lock,
                unknown_bits,
            }
        },

        // Set actor solidity.
        // "hitcheck"
        0x84 => {
            let bits = data.read_u8().unwrap();
            let mut flags_set = ActorFlags::empty();
            let mut flags_remove = ActorFlags::empty();

            flags_set.set(ActorFlags::SOLID, bits & 0x01 > 0);
            flags_set.set(ActorFlags::PUSHABLE, bits & 0x02 > 0);

            flags_remove.set(ActorFlags::SOLID, bits & 0x01 == 0);
            flags_remove.set(ActorFlags::PUSHABLE, bits & 0x02 == 0);

            Op::ActorUpdateFlags {
                actor: ActorRef::This,
                set: flags_set,
                remove: flags_remove,
            }
        },

        // Set actor collision properties.
        // "movecheck"
        0x0D => {
            let flags = data.read_u8().unwrap();
            let mut flags_set = ActorFlags::empty();
            let mut flags_remove = ActorFlags::empty();

            flags_set.set(ActorFlags::COLLISION_WITH_TILES, flags & 0x01 > 0);
            flags_set.set(ActorFlags::COLLISION_AVOID_PC, flags & 0x02 > 0);

            flags_remove.set(ActorFlags::COLLISION_WITH_TILES, flags & 0x01 == 0);
            flags_remove.set(ActorFlags::COLLISION_AVOID_PC, flags & 0x02 == 0);

            Op::ActorUpdateFlags {
                actor: ActorRef::This,
                set: flags_set,
                remove: flags_remove,
            }
        },

        // Set actor movement destination properties.
        // "moveadjust"
        0x0E => {
            let flags = data.read_u8().unwrap();
            let mut flags_set = ActorFlags::empty();
            let mut flags_remove = ActorFlags::empty();

            flags_set.set(ActorFlags::MOVE_ONTO_TILE, flags & 0x01 > 0);
            flags_set.set(ActorFlags::MOVE_ONTO_OBJECT, flags & 0x02 > 0);

            flags_remove.set(ActorFlags::MOVE_ONTO_TILE, flags & 0x01 == 0);
            flags_remove.set(ActorFlags::MOVE_ONTO_OBJECT, flags & 0x02 == 0);

            Op::ActorUpdateFlags {
                actor: ActorRef::This,
                set: flags_set,
                remove: flags_remove,
            }
        },

        // "mspeed"
        0x89 => Op::ActorSetSpeed {
            actor: ActorRef::This,
            speed: DataSource::Immediate(data.read_u8().unwrap() as i32),
        },
        // "vmspeed"
        0x8A => Op::ActorSetSpeed {
            actor: ActorRef::This,
            speed: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
        },

        // Coordinates from actor.
        // "where"
        0x21 => Op::ActorCoordinatesGet {
            actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            tile_x: DataDest::for_local_memory(data.read_u8().unwrap() as usize * 2),
            tile_y: DataDest::for_local_memory(data.read_u8().unwrap() as usize * 2),
        },

        // Coordinates from party member actor.
        // "pwhere"
        0x22 => Op::ActorCoordinatesGet {
            actor: ActorRef::ActivePartyIndex(data.read_u8().unwrap() as usize),
            tile_x: DataDest::for_local_memory(data.read_u8().unwrap() as usize * 2),
            tile_y: DataDest::for_local_memory(data.read_u8().unwrap() as usize * 2),
        },

        // Set coordinates.
        // "xy"
        0x8B => Op::ActorCoordinatesSet {
            actor: ActorRef::This,
            tile_x: DataSource::Immediate(data.read_u8().unwrap() as i32),
            tile_y: DataSource::Immediate(data.read_u8().unwrap() as i32),
        },
        // "vxy"
        0x8C => Op::ActorCoordinatesSet {
            actor: ActorRef::This,
            tile_x: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
            tile_y: DataSource::for_local_memory(data.read_u8().unwrap() as usize * 2),
        },
        // "dotxy"
        0x8D => Op::ActorCoordinatesSetPrecise {
            actor: ActorRef::This,
            x: DataSource::Immediate(data.read_u16::<LittleEndian>().unwrap() as i32 >> 4),
            y: DataSource::Immediate(data.read_u16::<LittleEndian>().unwrap() as i32 >> 4),
        },

        // "hpmpfull"
        0xF8 => Op::ActorHeal {
            actor: ActorRef::This,
            hp: true,
            mp: true,
        },
        // "hpfull"
        0xF9 => Op::ActorHeal {
            actor: ActorRef::This,
            hp: true,
            mp: false,
        },
        // "mpfull"
        0xFA => Op::ActorHeal {
            actor: ActorRef::This,
            hp: false,
            mp: true,
        },

        _ => panic!("Unknown actor property op."),
    }
}
