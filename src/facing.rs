#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub enum Facing {
    Up,
    #[default]
    Down,
    Left,
    Right,
}

impl Facing {
    pub fn to_index(&self) -> usize {
        match self {
            Facing::Up => 0,
            Facing::Down => 1,
            Facing::Left => 2,
            Facing::Right => 3,
        }
    }

    pub fn from_index(index: usize) -> Facing {
        match index {
            0 => Facing::Up,
            1 => Facing::Down,
            2 => Facing::Left,
            3 => Facing::Right,
            _ => Facing::default(),
        }
    }

    pub fn from_angle(angle: f64) -> Facing {
        match (angle / 90.0).floor() as u32 {
            0 => Facing::Down,
            1 => Facing::Left,
            2 => Facing::Up,
            3 => Facing::Right,
            _ => Facing::Up,
        }
    }
}
