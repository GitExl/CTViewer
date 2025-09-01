use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use crate::actor::ActorFlags;
use crate::scene_script::ops::Op;
use crate::scene_script::scene_script_decoder::{ActorRef, DataSource};
use crate::sprites::sprite_renderer::SpritePriority;

pub fn op_decode_actor_props(op: u8, data: &mut Cursor<Vec<u8>>) -> Op {
    match op {

        // Enable/disable this actor being able to be touched.
        0x08 => Op::ActorUpdateFlags {
            actor: ActorRef::This,
            set: ActorFlags::TOUCHABLE,
            remove: ActorFlags::empty(),
        },
        0x09 => Op::ActorUpdateFlags {
            actor: ActorRef::This,
            set: ActorFlags::TOUCHABLE,
            remove: ActorFlags::empty(),
        },

        // Disable and hide another actor.
        0x0A => Op::ActorUpdateFlags {
            actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            set: ActorFlags::DISABLED,
            remove: ActorFlags::RENDERED,
        },

        // Disable/enable script execution.
        0x0B => Op::ActorUpdateFlags {
            actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            set: ActorFlags::DISABLED,
            remove: ActorFlags::empty(),
        },
        0x0C => Op::ActorUpdateFlags {
            actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            set: ActorFlags::empty(),
            remove: ActorFlags::DISABLED,
        },

        // Visibility/rendered.
        // Rendered, and not hidden.
        0x7C => Op::ActorUpdateFlags {
            actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            set: ActorFlags::RENDERED,
            remove: ActorFlags::HIDDEN,
        },
        // Not rendered, and not hidden.
        0x7D => Op::ActorUpdateFlags {
            actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            set: ActorFlags::empty(),
            remove: ActorFlags::RENDERED | ActorFlags::HIDDEN,
        },
        // Visible.
        0x90 => Op::ActorUpdateFlags {
            actor: ActorRef::This,
            set: ActorFlags::empty(),
            remove: ActorFlags::HIDDEN,
        },
        // Hidden.
        0x91 => Op::ActorUpdateFlags {
            actor: ActorRef::This,
            set: ActorFlags::HIDDEN,
            remove: ActorFlags::empty(),
        },
        // Hidden, but rendered.
        0x7E => Op::ActorUpdateFlags {
            actor: ActorRef::This,
            set: ActorFlags::RENDERED | ActorFlags::HIDDEN,
            remove: ActorFlags::empty(),
        },

        // Sprite priority.
        0x8E => {
            let bits = data.read_u8().unwrap();
            let mode_set = bits & 0x80 > 0;
            let bottom = bits & 0x3;
            let top = (bits & 0x30) >> 4;
            let unknown_bits = bits & 0x4C;

            Op::ActorSetSpritePriority {
                actor: ActorRef::This,
                top: SpritePriority::from_value(top),
                bottom: SpritePriority::from_value(bottom),
                mode_set,
                unknown_bits,
            }
        },

        // Set actor solidity.
        0x84 => {
            let bits = data.read_u8().unwrap();
            let mut flags_set = ActorFlags::empty();
            if bits & 0x01 > 0 {
                flags_set |= ActorFlags::SOLID;
            }
            if bits & 0x02 > 0 {
                flags_set |= ActorFlags::PUSHABLE;
            }

            Op::ActorUpdateFlags {
                actor: ActorRef::This,
                set: flags_set,
                remove: flags_set.complement(),
            }
        },

        // Set actor collision properties.
        0x0D => {
            let flags = data.read_u8().unwrap();
            let mut flags_set = ActorFlags::empty();
            let mut flags_remove = ActorFlags::empty();

            if flags & 0x01 > 0 {
                flags_set.set(ActorFlags::COLLISION_TILE, true);
            } else {
                flags_remove.set(ActorFlags::COLLISION_TILE, true);
            }
            if flags & 0x02 > 0 {
                flags_set.set(ActorFlags::COLLISION_PC, true);
            } else {
                flags_remove.set(ActorFlags::COLLISION_PC, true);
            }

            Op::ActorUpdateFlags {
                actor: ActorRef::This,
                set: flags_set,
                remove: flags_remove,
            }
        },

        // Set actor movement properties.
        0x0E => {
            let flags = data.read_u8().unwrap();
            let mut flags_set = ActorFlags::empty();
            let mut flags_remove = ActorFlags::empty();

            if flags & 0x01 > 0 {
                flags_set.set(ActorFlags::MOVE_ONTO_TILE, true);
            } else {
                flags_remove.set(ActorFlags::MOVE_ONTO_TILE, true);
            }
            if flags & 0x02 > 0 {
                flags_set.set(ActorFlags::MOVE_ONTO_OBJECT, true);
            } else {
                flags_remove.set(ActorFlags::MOVE_ONTO_OBJECT, true);
            }

            Op::ActorUpdateFlags {
                actor: ActorRef::This,
                set: flags_set,
                remove: flags_remove,
            }
        },

        0x89 => Op::ActorSetSpeed {
            actor: ActorRef::This,
            speed: DataSource::Immediate(data.read_u8().unwrap() as u32),
        },
        0x8A => Op::ActorSetSpeed {
            actor: ActorRef::This,
            speed: DataSource::LocalVar(data.read_u8().unwrap() as usize * 2),
        },

        // Coordinates from actor.
        0x21 => Op::ActorCoordinatesGet {
            actor: ActorRef::ScriptActor(data.read_u8().unwrap() as usize / 2),
            x: DataSource::LocalVar(data.read_u8().unwrap() as usize * 2),
            y: DataSource::LocalVar(data.read_u8().unwrap() as usize * 2),
        },

        // Coordinates from party member actor.
        0x22 => Op::ActorCoordinatesGet {
            actor: ActorRef::PartyMember(data.read_u8().unwrap() as usize),
            x: DataSource::LocalVar(data.read_u8().unwrap() as usize * 2),
            y: DataSource::LocalVar(data.read_u8().unwrap() as usize * 2),
        },

        // Set coordinates.
        0x8B => Op::ActorCoordinatesSet {
            actor: ActorRef::This,
            x: DataSource::Immediate(data.read_u8().unwrap() as u32),
            y: DataSource::Immediate(data.read_u8().unwrap() as u32),
            precise: false,
        },
        0x8C => Op::ActorCoordinatesSet {
            actor: ActorRef::This,
            x: DataSource::LocalVar(data.read_u8().unwrap() as usize * 2),
            y: DataSource::LocalVar(data.read_u8().unwrap() as usize * 2),
            precise: false,
        },
        0x8D => Op::ActorCoordinatesSet {
            actor: ActorRef::This,
            x: DataSource::Immediate(data.read_u16::<LittleEndian>().unwrap() as u32 >> 4),
            y: DataSource::Immediate(data.read_u16::<LittleEndian>().unwrap() as u32 >> 4),
            precise: true,
        },

        // Actor sprite.
        0xAC => Op::ActorSetSpriteFrame {
            actor: ActorRef::This,
            frame: DataSource::Immediate(data.read_u8().unwrap() as u32),
        },

        0xF8 => Op::ActorHeal {
            actor: ActorRef::This,
            hp: true,
            mp: true,
        },
        0xF9 => Op::ActorHeal {
            actor: ActorRef::This,
            hp: true,
            mp: false,
        },
        0xFA => Op::ActorHeal {
            actor: ActorRef::This,
            hp: false,
            mp: true,
        },

        _ => panic!("Unknown actor property op."),
    }
}
