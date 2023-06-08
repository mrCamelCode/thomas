
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
