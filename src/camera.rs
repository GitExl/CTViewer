pub struct Camera {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,

    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,

    last_x: f64,
    last_y: f64,
    pub lerp_x: f64,
    pub lerp_y: f64,
}

impl Camera {
    pub fn new(x: f64, y: f64, width: f64, height: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> Camera {
        let mut camera = Camera {
            x, y, width, height,
            x1, y1, x2, y2,

            last_x: x, last_y: y,
            lerp_x: x, lerp_y: y,
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
        self.last_x = self.x;
        self.last_y = self.y;
    }

    pub fn lerp(&mut self, lerp: f64) {
        self.lerp_x = self.last_x + (self.x - self.last_x) * lerp;
        self.lerp_y = self.last_y + (self.y - self.last_y) * lerp;
    }

    pub fn clamp(&mut self) {
        self.x = self.x.min(self.x2 - self.width).max(self.x1);
        self.y = self.y.min(self.y2 - self.height).max(self.y1);
    }

    pub fn wrap(&mut self) {
        // @todo broken, wraps but not to the correct spot

        if self.x < self.x1 {
            self.x = self.x2 - self.width + (self.x - self.x1);
        } else if self.x + self.width >= self.x2 {
            self.x = self.x1 + (self.x + self.width - self.x2);
        }

        if self.y < self.y1 {
            self.y = self.y2 - self.height + (self.y - self.y1);
        } else if self.y + self.height >= self.y2 {
            self.y = self.y1 + (self.y + self.height - self.y2);
        }
    }

    pub fn center_to(&mut self, x: f64, y: f64) {
        self.x = x - self.width / 2.0;
        self.y = y - self.height / 2.0;
        self.clamp();

        self.last_x = self.x;
        self.last_y = self.y;
        self.lerp_x = self.x;
        self.lerp_y = self.y;
    }
}
