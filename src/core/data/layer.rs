#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Layer(pub i32);
impl Layer {
    pub fn base() -> Self {
        Self(0)
    }

    pub fn furthest_background() -> Self {
        Self(i32::MIN)
    }

    pub fn furthest_foreground() -> Self {
        Self(i32::MAX)
    }

    pub fn above(other: &Layer) -> Self {
        Self(other.value() + 1)
    }

    pub fn below(other: &Layer) -> Self {
        Self(other.value() - 1)
    }

    pub fn with(other: &Layer) -> Self {
        Self(other.value())
    }

    pub fn is_above(&self, other: &Layer) -> bool {
        self.value() > other.value()
    }

    pub fn is_below(&self, other: &Layer) -> bool {
        self.value() < other.value()
    }

    pub fn is_with(&self, other: &Layer) -> bool {
        self.value() == other.value()
    }

    pub fn value(&self) -> i32 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod above {
        use super::*;

        #[test]
        fn it_produces_a_value_above_the_provided_layer() {
            let source = Layer(3);

            assert!(Layer::above(&source).is_above(&source));
        }
    }

    mod below {
        use super::*;

        #[test]
        fn it_produces_a_value_below_the_provided_layer() {
            let source = Layer(3);

            assert!(Layer::below(&source).is_below(&source));
        }
    }

    mod with {
        use super::*;

        #[test]
        fn it_produces_a_value_with_the_provided_layer() {
            let source = Layer(3);

            assert!(Layer::with(&source).is_with(&source));
        }
    }

    mod is_above {
        use super::*;

        #[test]
        fn returns_true_when_source_is_above_other() {
            let layer = Layer(2);

            assert!(layer.is_above(&Layer(1)));
        }

        #[test]
        fn returns_false_when_source_is_not_above_other() {
            let layer = Layer(0);

            assert!(!layer.is_above(&Layer(2)));
        }
    }

    mod is_below {
        use super::*;

        #[test]
        fn returns_true_when_source_is_below_other() {
            let layer = Layer(2);

            assert!(!layer.is_below(&Layer(1)));
        }

        #[test]
        fn returns_false_when_source_is_not_below_other() {
            let layer = Layer(0);

            assert!(layer.is_below(&Layer(2)));
        }
    }

    mod is_with {
        use super::*;

        #[test]
        fn returns_false_when_source_is_above_other() {
            let layer = Layer(2);

            assert!(!layer.is_with(&Layer(1)));
        }

        #[test]
        fn returns_false_when_source_is_below_other() {
            let layer = Layer(0);

            assert!(!layer.is_with(&Layer(2)));
        }

        #[test]
        fn returns_true_when_source_with_other() {
            let layer = Layer(3);

            assert!(layer.is_with(&Layer(3)));
        }
    }
}
