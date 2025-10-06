use std::f64::consts::PI;
use std::{fmt, ops};
use crate::util::vec2di32::Vec2Di32;

#[derive(Clone, Copy)]
pub struct Vec2Df64 {
    pub x: f64,
    pub y: f64,
}

impl Vec2Df64 {
    pub fn default() -> Vec2Df64 {
        Vec2Df64 {
            x: 0.0,
            y: 0.0,
        }
    }

    pub fn new(x: f64, y: f64) -> Vec2Df64 {
        Vec2Df64 {
            x, y,
        }
    }

    pub fn floor(&self) -> Vec2Df64 {
        Vec2Df64 {
            x: self.x.floor(),
            y: self.y.floor(),
        }
    }

    pub fn ceil(&self) -> Vec2Df64 {
        Vec2Df64 {
            x: self.x.ceil(),
            y: self.y.ceil(),
        }
    }

    pub fn interpolate(prev: Vec2Df64, next: Vec2Df64, alpha: f64) -> Vec2Df64 {
        Vec2Df64 {
            x: prev.x + (next.x - prev.x) * alpha,
            y: prev.y + (next.y - prev.y) * alpha,
        }
    }

    pub fn angle_deg_between(v1: Vec2Df64, v2: Vec2Df64) -> f64 {
        let diff_x = f64::from(v2.x - v1.x);
        let diff_y = f64::from(v2.y - v1.y);
        diff_y.atan2(diff_x) * 180.0 / PI
    }

    pub fn angle_rad_between(v1: Vec2Df64, v2: Vec2Df64) -> f64 {
        let diff_x = f64::from(v2.x - v1.x);
        let diff_y = f64::from(v2.y - v1.y);
        diff_y.atan2(diff_x)
    }

    pub fn as_vec2d_i32(&self) -> Vec2Di32 {
        Vec2Di32::new(self.x as i32, self.y as i32)
    }

    pub fn signum(&self) -> Vec2Df64 {
        Vec2Df64::new(
            self.x.signum(),
            self.y.signum(),
        )
    }
}

impl fmt::Display for Vec2Df64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.1} x {:.1}", self.x, self.y)
    }
}

impl ops::Add<Vec2Df64> for Vec2Df64 {
    type Output = Vec2Df64;

    fn add(self, _rhs: Vec2Df64) -> Vec2Df64 {
        Vec2Df64 {
            x: self.x + _rhs.x,
            y: self.y + _rhs.y,
        }
    }
}

impl ops::Add<f64> for Vec2Df64 {
    type Output = Vec2Df64;

    fn add(self, _rhs: f64) -> Vec2Df64 {
        Vec2Df64 {
            x: self.x + _rhs,
            y: self.y + _rhs,
        }
    }
}

impl ops::Sub<Vec2Df64> for Vec2Df64 {
    type Output = Vec2Df64;

    fn sub(self, _rhs: Vec2Df64) -> Vec2Df64 {
        Vec2Df64 {
            x: self.x - _rhs.x,
            y: self.y - _rhs.y,
        }
    }
}

impl ops::Sub<f64> for Vec2Df64 {
    type Output = Vec2Df64;

    fn sub(self, _rhs: f64) -> Vec2Df64 {
        Vec2Df64 {
            x: self.x - _rhs,
            y: self.y - _rhs,
        }
    }
}

impl ops::Div<Vec2Df64> for Vec2Df64 {
    type Output = Vec2Df64;

    fn div(self, _rhs: Vec2Df64) -> Vec2Df64 {
        Vec2Df64 {
            x: self.x / _rhs.x,
            y: self.y / _rhs.y,
        }
    }
}

impl ops::Div<f64> for Vec2Df64 {
    type Output = Vec2Df64;

    fn div(self, _rhs: f64) -> Vec2Df64 {
        Vec2Df64 {
            x: self.x / _rhs,
            y: self.y / _rhs,
        }
    }
}

impl ops::Mul<Vec2Df64> for Vec2Df64 {
    type Output = Vec2Df64;

    fn mul(self, _rhs: Vec2Df64) -> Vec2Df64 {
        Vec2Df64 {
            x: self.x * _rhs.x,
            y: self.y * _rhs.y,
        }
    }
}

impl ops::Mul<f64> for Vec2Df64 {
    type Output = Vec2Df64;

    fn mul(self, _rhs: f64) -> Vec2Df64 {
        Vec2Df64 {
            x: self.x * _rhs,
            y: self.y * _rhs,
        }
    }
}
