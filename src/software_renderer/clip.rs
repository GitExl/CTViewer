#[derive(Copy, Clone)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Rect {
    pub fn new(left: i32, top: i32, right: i32, bottom: i32) -> Rect {
        Rect {
            left,
            top,
            right,
            bottom,
        }
    }

    pub fn clip_to(&self, other: &Rect) -> Rect {
        Rect {
            top: other.top.max(self.top),
            bottom: other.bottom.min(self.bottom),
            left: other.left.max(self.left),
            right: other.right.min(self.right),
        }
    }
}
