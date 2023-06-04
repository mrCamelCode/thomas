
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
