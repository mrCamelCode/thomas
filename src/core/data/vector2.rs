pub struct Vector2 {
    x: f64,
    y: f64,
}

impl Vector2 {
    pub fn new(x: f64, y: f64) -> Vector2 {
        Vector2 { x, y }
    }

    pub fn zero() -> Vector2 {
        Vector2 { x: 0.0, y: 0.0 }
    }

    pub fn left() -> Vector2 {
        Vector2 { x: -1.0, y: 0.0 }
    }

    pub fn right() -> Vector2 {
        Vector2 { x: 1.0, y: 0.0 }
    }

    pub fn up() -> Vector2 {
        Vector2 { x: 0.0, y: 1.0 }
    }

    pub fn down() -> Vector2 {
        Vector2 { x: 0.0, y: -1.0 }
    }

    pub fn distance_from(&self, other: &Vector2) -> f64 {
        let diff_x = f64::abs(self.x - other.x);
        let diff_y = f64::abs(self.y - other.y);

        f64::sqrt(diff_x.powf(2.0) + diff_y.powf(2.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod distance_from {
        use super::*;

        #[test]
        fn same() {
            let v = Vector2::new(0.0, 5.0);

            assert_eq!(v.distance_from(&v), 0.0);
        }

        #[test]
        fn same_x() {
            let v = Vector2::new(0.0, 5.0);

            assert_eq!(v.distance_from(&Vector2::new(0.0, 10.0)), 5.0);
        }

        #[test]
        fn same_y() {
            let v = Vector2::new(2.0, 0.0);

            assert_eq!(v.distance_from(&Vector2::new(5.0, 0.0)), 3.0);
        }

        #[test]
        fn different_x_and_y() {
            let v1 = Vector2::new(1.0, 2.0);
            let v2 = Vector2::new(2.0, 4.0);

            assert_eq!(v1.distance_from(&v2), f64::sqrt(5.0));
        }
    }
}
