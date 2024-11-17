#[derive(Clone, Copy)]
pub struct Box {
    pub x1: i32,
    pub y1: i32,
    pub xs: u32,
    pub ys: u32,
}

impl Box {
    pub fn area(&self) -> u64 {
        (self.xs as u64) * (self.ys as u64)
    }
    pub fn x2(&self) -> i32 {
        self.x1 + self.xs as i32
    }
    pub fn y2(&self) -> i32 {
        self.y1 + self.ys as i32
    }
}
