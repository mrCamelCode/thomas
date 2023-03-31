#[derive(Clone, Copy)]
pub struct Dimensions2d {
  height: usize,
  width: usize,
}

impl Dimensions2d {
  pub fn new(height: usize, width: usize) -> Self {
    Dimensions2d { height, width }
  }

  pub fn height(&self) -> usize {
    self.height
  }

  pub fn width(&self) -> usize {
    self.width
  }
}