use crate::util::vec2df64::Vec2Df64;

#[derive(PartialEq, Debug)]
pub enum CameraMoveTo {
    Disabled,
    Enabled,
    Complete,
}

pub struct Camera {
    pub pos: Vec2Df64,
    pos_last: Vec2Df64,
    pub pos_lerp: Vec2Df64,
    pub size: Vec2Df64,

    pub move_to: Vec2Df64,
    pub move_to_state: CameraMoveTo,

    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
}

impl Camera {
    pub fn new(x: f64, y: f64, width: f64, height: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> Camera {
        let mut camera = Camera {
            pos: Vec2Df64::new(x, y),
            pos_last: Vec2Df64::new(x, y),
            pos_lerp: Vec2Df64::new(x, y),
            size: Vec2Df64::new(width, height),
            move_to: Vec2Df64::new(0.0, 0.0),
            move_to_state: CameraMoveTo::Disabled,
            x1, y1, x2, y2,
        };
        camera.clamp();
        camera.tick(0.0);

        camera
    }

    pub fn set_area(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) {
        self.x1 = x1;
        self.y1 = y1;
        self.x2 = x2;
        self.y2 = y2;
    }

    pub fn tick(&mut self, _: f64) {
        self.pos_last = self.pos;

        if self.move_to_state == CameraMoveTo::Enabled {
            if self.pos.x as i32 == self.move_to.x as i32 && self.pos.y as i32 == self.move_to.y as i32 {
                self.pos = self.move_to;
                self.move_to_state = CameraMoveTo::Complete;
            } else {
                let distance = (self.move_to - self.pos).as_vec2d_i32();
                if distance.x != 0 {
                    self.pos.x += distance.x.signum() as f64;
                }
                if distance.y != 0 {
                    self.pos.y += distance.y.signum() as f64;
                }
            }
        }
    }

    pub fn move_to(&mut self, move_to: Vec2Df64) {
        self.move_to = move_to;

        // Clamp destination to camera size, otherwise the movement will never complete.
        self.move_to.x = self.move_to.x.min(self.x2 - self.size.x).max(self.x1);
        self.move_to.y = self.move_to.y.min(self.y2 - self.size.y).max(self.y1);

        self.move_to_state = CameraMoveTo::Enabled;
    }

    pub fn lerp(&mut self, lerp: f64) {
        self.pos_lerp = Vec2Df64::interpolate(self.pos_last, self.pos, lerp);
    }

    pub fn clamp(&mut self) {
        if self.size.x >= self.x2 - self.x1 {
            self.pos.x = (self.x1 + (self.x2 - self.x1) / 2.0) - self.size.x / 2.0;
        } else {
            self.pos.x = self.pos.x.min(self.x2 - self.size.x).max(self.x1);
        }

        if self.size.y >= self.y2 - self.y1 {
            self.pos.y = (self.y1 + (self.y2 - self.y1) / 2.0) - self.size.y / 2.0;
        } else {
            self.pos.y = self.pos.y.min(self.y2 - self.size.y).max(self.y1);
        }
    }

    pub fn wrap(&mut self) {
        let mut center = self.pos + self.size / 2.0;

        if center.x < self.x1 {
            center.x = self.x2 - (self.x1 - center.x);
        } else if center.x >= self.x2 {
            center.x = self.x1 + (center.x - self.x2);
        }

        if center.y < self.y1 {
            center.y = self.y2 - (self.y1 - center.y);
        } else if center.y >= self.y2 {
            center.y = self.y1 + (center.y - self.y2);
        }

        self.pos = center - self.size / 2.0;
    }

    pub fn center_to(&mut self, center: Vec2Df64) {
        self.pos = center - self.size / 2.0;

        self.clamp();

        self.pos_last = self.pos;
        self.pos_lerp = self.pos;
    }
}
