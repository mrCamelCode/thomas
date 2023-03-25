use super::Renderer;

// TODO: The crate crossterm should make this renderer possible.
pub struct TerminalRenderer {

}

impl Renderer for TerminalRenderer {
  fn render(&self) {
    todo!();
  }
}

impl TerminalRenderer {
  pub fn new() -> Self {
    TerminalRenderer {  }
  }
}
