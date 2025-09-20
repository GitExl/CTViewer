use std::f64::consts::PI;
use std::{fmt, ops};
use crate::util::vec2df64::Vec2Df64;

#[derive(Clone, Copy)]
pub struct Vec2Di32 {
    pub x: i32,
    pub y: i32,
}

impl Vec2Di32 {
    pub fn default() -> Vec2Di32 {
        Vec2Di32 {
            x: 0,
            y: 0,
        }
    }

    pub fn new(x: i32, y: i32) -> Vec2Di32 {
        Vec2Di32 {
            x, y,
        }
    }

    pub fn interpolate(v1: Vec2Di32, v2: Vec2Di32, alpha: f64) -> Vec2Di32 {
        Vec2Di32 {
            x: ((v1.x + (v2.x - v1.x)) as f64 * alpha) as i32,
            y: ((v1.y + (v2.y - v1.y)) as f64 * alpha) as i32,
        }
    }

    pub fn angle_deg_between(v1: Vec2Di32, v2: Vec2Di32) -> f64 {
        let diff_x = (v2.x - v1.x) as f64;
        let diff_y = (v2.y - v1.y) as f64;
        diff_y.atan2(diff_x) * 180.0 / PI
    }

    pub fn angle_rad_between(v1: Vec2Di32, v2: Vec2Di32) -> f64 {
        let diff_x = (v2.x - v1.x) as f64;
        let diff_y = (v2.y - v1.y) as f64;
        diff_y.atan2(diff_x)
    }

    pub fn as_vec2d_f64(&self) -> Vec2Df64 {
        Vec2Df64::new(self.x as f64, self.y as f64)
    }
}

impl fmt::Display for Vec2Di32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} x {}", self.x, self.y)
    }
}

impl ops::Add<Vec2Di32> for Vec2Di32 {
    type Output = Vec2Di32;

    fn add(self, _rhs: Vec2Di32) -> Vec2Di32 {
        Vec2Di32 {
            x: self.x + _rhs.x,
            y: self.y + _rhs.y,
        }
    }
}

impl ops::Add<i32> for Vec2Di32 {
    type Output = Vec2Di32;

    fn add(self, _rhs: i32) -> Vec2Di32 {
        Vec2Di32 {
            x: self.x + _rhs,
            y: self.y + _rhs,
        }
    }
}

impl ops::Sub<Vec2Di32> for Vec2Di32 {
    type Output = Vec2Di32;

    fn sub(self, _rhs: Vec2Di32) -> Vec2Di32 {
        Vec2Di32 {
            x: self.x - _rhs.x,
            y: self.y - _rhs.y,
        }
    }
}

impl ops::Sub<i32> for Vec2Di32 {
    type Output = Vec2Di32;

    fn sub(self, _rhs: i32) -> Vec2Di32 {
        Vec2Di32 {
            x: self.x - _rhs,
            y: self.y - _rhs,
        }
    }
}

impl ops::Div<Vec2Di32> for Vec2Di32 {
    type Output = Vec2Di32;

    fn div(self, _rhs: Vec2Di32) -> Vec2Di32 {
        Vec2Di32 {
            x: self.x / _rhs.x,
            y: self.y / _rhs.y,
        }
    }
}

impl ops::Div<i32> for Vec2Di32 {
    type Output = Vec2Di32;

    fn div(self, _rhs: i32) -> Vec2Di32 {
        Vec2Di32 {
            x: self.x / _rhs,
            y: self.y / _rhs,
        }
    }
}

impl ops::Mul<Vec2Di32> for Vec2Di32 {
    type Output = Vec2Di32;

    fn mul(self, _rhs: Vec2Di32) -> Vec2Di32 {
        Vec2Di32 {
            x: self.x * _rhs.x,
            y: self.y * _rhs.y,
        }
    }
}

impl ops::Mul<i32> for Vec2Di32 {
    type Output = Vec2Di32;

    fn mul(self, _rhs: i32) -> Vec2Di32 {
        Vec2Di32 {
            x: self.x * _rhs,
            y: self.y * _rhs,
        }
    }
}
