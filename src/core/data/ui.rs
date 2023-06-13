/// Where the UI element is anchored on the screen. The anchor represents where the element is positioned by default
/// when it has no offset.
#[derive(PartialEq, Eq, Debug, Hash)]
pub enum UiAnchor {
    TopLeft,
    MiddleTop,
    TopRight,
    MiddleRight,
    BottomRight,
    MiddleBottom,
    BottomLeft,
    MiddleLeft,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Alignment {
    Left,
    Middle,
    Right,
}

/// Represents an RGB color.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Rgb(pub u8, pub u8, pub u8);
impl Rgb {
  /// The red value of the color.
  pub fn r(&self) -> u8 {
    self.0
  }

  /// The green value of the color.
  pub fn g(&self) -> u8 {
    self.1
  }

  /// The blue value of the color.
  pub fn b(&self) -> u8 {
    self.2
  }

  pub fn white() -> Self {
    Self(255, 255, 255)
  }

  pub fn black() -> Self {
    Self(0, 0, 0)
  }

  pub fn red() -> Self {
    Self(255, 0, 0)
  }

  pub fn green() -> Self {
    Self(0, 255, 0)
  }

  pub fn blue() -> Self {
    Self(0, 0, 255)
  }

  pub fn cyan() -> Self {
    Self(0, 255, 255)
  }

  pub fn magenta() -> Self {
    Self(255, 0, 255)
  }

  pub fn yellow() -> Self {
    Self(255, 255, 0)
  }
}
