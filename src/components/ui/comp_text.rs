use crate::{Component, UiAnchor, Alignment, IntCoords2d, Rgb};

/// Text UI data that describes how the Text should be visible on the screen.
#[derive(Component, Debug)]
pub struct Text {
  pub value: String,
  pub anchor: UiAnchor,
  pub justification: Alignment,
  pub offset: IntCoords2d,
  pub foreground_color: Option<Rgb>,
  pub background_color: Option<Rgb>,
}