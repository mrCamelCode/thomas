#[derive(Clone)]
pub struct Layer {
    value: i32,
}
impl Layer {
    pub fn new(value: i32) -> Self {
        Self { value }
    }

    pub fn base() -> Self {
        Self { value: 0 }
    }

    pub fn furthest_background() -> Self {
        Self { value: i32::MIN }
    }

    pub fn above(other: &Layer) -> Self {
        Self {
            value: other.value + 1,
        }
    }

    pub fn below(other: &Layer) -> Self {
        Self {
            value: other.value - 1,
        }
    }

    pub fn with(other: &Layer) -> Self {
        Self { value: other.value }
    }

    pub fn is_above(&self, other: &Layer) -> bool {
        self.value > other.value
    }

    pub fn is_below(&self, other: &Layer) -> bool {
        self.value < other.value
    }

    pub fn is_with(&self, other: &Layer) -> bool {
        self.value == other.value
    }

    pub fn value(&self) -> i32 {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod above {
        use super::*;

        #[test]
        fn it_produces_a_value_above_the_provided_layer() {
            let source = Layer::new(3);

            assert!(Layer::above(&source).is_above(&source));
        }
    }

    mod below {
        use super::*;

        #[test]
        fn it_produces_a_value_below_the_provided_layer() {
            let source = Layer::new(3);

            assert!(Layer::below(&source).is_below(&source));
        }
    }

    mod with {
        use super::*;

        #[test]
        fn it_produces_a_value_with_the_provided_layer() {
            let source = Layer::new(3);

            assert!(Layer::with(&source).is_with(&source));
        }
    }

    mod is_above {
        use super::*;

        #[test]
        fn returns_true_when_source_is_above_other() {
            let layer = Layer::new(2);

            assert!(layer.is_above(&Layer::new(1)));
        }

        #[test]
        fn returns_false_when_source_is_not_above_other() {
            let layer = Layer::new(0);

            assert!(!layer.is_above(&Layer::new(2)));
        }
    }

    mod is_below {
        use super::*;

        #[test]
        fn returns_true_when_source_is_below_other() {
            let layer = Layer::new(2);

            assert!(!layer.is_below(&Layer::new(1)));
        }

        #[test]
        fn returns_false_when_source_is_not_below_other() {
            let layer = Layer::new(0);

            assert!(layer.is_below(&Layer::new(2)));
        }
    }

    mod is_with {
        use super::*;

        #[test]
        fn returns_false_when_source_is_above_other() {
            let layer = Layer::new(2);

            assert!(!layer.is_with(&Layer::new(1)));
        }

        #[test]
        fn returns_false_when_source_is_below_other() {
            let layer = Layer::new(0);

            assert!(!layer.is_with(&Layer::new(2)));
        }

        #[test]
        fn returns_true_when_source_with_other() {
            let layer = Layer::new(3);

            assert!(layer.is_with(&Layer::new(3)));
        }
    }
}
