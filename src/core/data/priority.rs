use std::ops::Deref;

pub struct Priority {
    value: u128,
}
impl Priority {
    pub fn new(priority: u128) -> Self {
        Self { value: priority }
    }

    pub fn default() -> Self {
        Self { value: 100 }
    }

    pub fn highest() -> Self {
        Self { value: 0 }
    }

    pub fn lowest() -> Self {
        Self { value: u128::MAX }
    }
}
impl Deref for Priority {
  type Target = u128;

  fn deref(&self) -> &Self::Target {
      &self.value
  }
}
