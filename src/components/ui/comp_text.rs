use crate::{Component, UiAnchor, Alignment, IntCoords2d};

#[derive(Component, Debug)]
pub struct Text {
  pub value: String,
  pub anchor: UiAnchor,
  pub justification: Alignment,
  pub offset: IntCoords2d,
}