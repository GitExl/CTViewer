pub struct ClipRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl ClipRect {
    pub fn new(left: i32, top: i32, right: i32, bottom: i32) -> ClipRect {
        ClipRect {
            left,
            top,
            right,
            bottom,
        }
    }
}
