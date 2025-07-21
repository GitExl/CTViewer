pub const DIRECTION_COUNT: usize = 4;

// A sprite animation frame.
// Each frame references sprite frames for each direction.
pub struct SpriteAnimFrame {
    pub sprite_frames: [usize; DIRECTION_COUNT],
    pub duration: f64,
}

// A sprite animation.
pub struct SpriteAnim {
    pub frames: Vec<SpriteAnimFrame>,
}

impl SpriteAnim {
    pub fn empty() -> SpriteAnim {
        SpriteAnim {
            frames: Vec::new(),
        }
    }
}

// A set of sprite animations, usually associated with one sprite.
pub struct SpriteAnimSet {
    pub index: usize,
    pub anims: Vec<SpriteAnim>,
}

impl SpriteAnimSet {
    pub fn new(index: usize) -> SpriteAnimSet {
        SpriteAnimSet {
            index,
            anims: Vec::new(),
        }
    }

    pub fn dump(&self) {
        println!("Sprite animation set {}", self.index);
        for (j, anim) in self.anims.iter().enumerate() {
            println!("  Sprite animation {}", j);
            for (k, frame) in anim.frames.iter().enumerate() {
                println!("    Frame {}: U {:>3} D {:>3} L {:>3} R {:>3}, {:.3} seconds", k, frame.sprite_frames[0], frame.sprite_frames[1], frame.sprite_frames[2], frame.sprite_frames[3], frame.duration);
            }
        }
    }
}
