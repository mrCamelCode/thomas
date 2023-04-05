#[derive(Clone)]
pub struct Dimensions2d {
    height: u64,
    width: u64,
}

impl Dimensions2d {
    pub fn new(height: u64, width: u64) -> Self {
        Dimensions2d { height, width }
    }

    pub fn height(&self) -> u64 {
        self.height
    }

    pub fn width(&self) -> u64 {
        self.width
    }
}
