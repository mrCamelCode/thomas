use std::ops::Deref;

/// A representation of how important something is relative to something else.
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

    pub fn lower_than(other: &Priority) -> Self {
        Self {
            value: if other.value == u128::MAX {
                u128::MAX
            } else {
                other.value + 1
            },
        }
    }

    pub fn higher_than(other: &Priority) -> Self {
        Self {
            value: if other.value == 0 {
                0
            } else {
                other.value - 1
            }
        }
    }
}
impl Deref for Priority {
    type Target = u128;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
