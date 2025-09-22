use bitflags::bitflags;
use crate::sprites::sprite_renderer::SpritePriority;
use crate::util::vec2df64::Vec2Df64;

bitflags! {
    #[derive(Clone, Copy, Default)]
    pub struct SceneTileFlags: u32 {
        const L1_TILE_ADD = 0x001;
        const L2_TILE_ADD = 0x002;
        const DOOR_TRIGGER = 0x004;
        const UNKNOWN_1 = 0x008;
        const UNKNOWN_2 = 0x010;
        const NPC_COLLISION_BATTLE = 0x080;
        const NPC_COLLISION = 0x100;
        const COLLISION_IGNORE_Z = 0x200;
        const COLLISION_INVERTED = 0x400;
        const Z_NEUTRAL = 0x800;
        const RLE_COMPRESSED = 0x1000;
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub enum SceneTileCollision {
    #[default]
    None,

    Full,

    Corner45NW,
    Corner45NE,
    Corner45SW,
    Corner45SE,

    Corner30NW,
    Corner30NE,
    Corner30SW,
    Corner30SE,

    Corner22NW,
    Corner22NE,
    Corner22SW,
    Corner22SE,

    Corner75NW,
    Corner75NE,
    Corner75SW,
    Corner75SE,

    Corner75NWDup,
    Corner75NEDup,
    Corner75SWDup,
    Corner75SEDup,

    StairsSWNE,
    StairsSENW,

    LeftHalf,
    TopHalf,

    SW,
    SE,
    NE,
    NW,

    Ladder,

    Invalid,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum SceneMoveDirection {
    #[default]
    North,
    South,
    West,
    East,
}

#[derive(Clone, Copy, Default)]
pub struct SceneTileProps {
    pub flags: SceneTileFlags,
    pub collision: SceneTileCollision,
    pub sprite_priority_top: SpritePriority,
    pub sprite_priority_bottom: SpritePriority,
    pub z_plane: u32,           // transition (solid), 1, 2, transition (walkable)
    pub move_direction: SceneMoveDirection,
    pub move_speed: u32,
}

pub struct ScenePropLayer {
    pub width: u32,
    pub height: u32,
    pub props: Vec<SceneTileProps>,
}

pub struct SceneMap {
    pub index: usize,
    pub props: ScenePropLayer,
}

impl SceneMap {
    pub fn get_props_at_pixel(&self, pos: Vec2Df64) -> Option<&SceneTileProps> {
        let tile_pos = (pos / 16.0).as_vec2d_i32();
        let index = (tile_pos.y * self.props.width as i32 + tile_pos.x) as usize;
        self.props.props.get(index)
    }

    pub fn dump(&self) {
        println!("Scene map {}", self.index);
        println!("  {} x {} tiles", self.props.width, self.props.height);
        println!();
    }
}
