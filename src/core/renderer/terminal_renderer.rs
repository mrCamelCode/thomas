use super::Renderer;

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
