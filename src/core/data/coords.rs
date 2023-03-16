#[derive(Clone)]
pub struct Coords {
    x: f64,
    y: f64,
    z: f64,
}

impl Coords {
    pub fn new(x: f64, y: f64, z: f64) -> Coords {
        Coords { x, y, z }
    }

    pub fn zero() -> Coords {
        Coords::new(0.0, 0.0, 0.0)
    }

    pub fn left() -> Coords {
        Coords::new(-1.0, 0.0, 0.0)
    }

    pub fn right() -> Coords {
        Coords::new(1.0, 0.0, 0.0)
    }

    pub fn up() -> Coords {
        Coords::new(0.0, 1.0, 0.0)
    }

    pub fn down() -> Coords {
        Coords::new(0.0, -1.0, 0.0)
    }

    pub fn forward() -> Coords {
        Coords::new(0.0, 0.0, -1.0)
    }

    pub fn backward() -> Coords {
        Coords::new(0.0, 0.0, 1.0)
    }

    pub fn distance_from_2d(&self, other: &Coords) -> f64 {
        let diff_x = f64::abs(self.x - other.x);
        let diff_y = f64::abs(self.y - other.y);

        f64::sqrt(diff_x.powf(2.0) + diff_y.powf(2.0))
    }

    pub fn add(&mut self, other: &Coords) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }

    pub fn z(&self) -> f64 {
        self.z
    }

    pub fn values(&self) -> (f64, f64, f64) {
        (self.x, self.y, self.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod distance_from_2d {
        use super::*;

        #[test]
        fn same() {
            let v = Coords::new(0.0, 5.0, 0.0);

            assert_eq!(v.distance_from_2d(&v), 0.0);
        }

        #[test]
        fn same_x() {
            let v = Coords::new(0.0, 5.0, 0.0);

            assert_eq!(v.distance_from_2d(&Coords::new(0.0, 10.0, 0.0)), 5.0);
        }

        #[test]
        fn same_y() {
            let v = Coords::new(2.0, 0.0, 0.0);

            assert_eq!(v.distance_from_2d(&Coords::new(5.0, 0.0, 0.0)), 3.0);
        }

        #[test]
        fn different_x_and_y() {
            let v1 = Coords::new(1.0, 2.0, 0.0);
            let v2 = Coords::new(2.0, 4.0, 0.0);

            assert_eq!(v1.distance_from_2d(&v2), f64::sqrt(5.0));
        }
    }

    mod add {
        use super::*;

        #[test]
        fn no_change() {
            let mut v1 = Coords::new(1.0, 2.0, 0.0);
            let v2 = Coords::zero();

            v1.add(&v2);

            assert_eq!(v1.x, 1.0);
            assert_eq!(v1.y, 2.0);
        }

        #[test]
        fn works_with_positive_values() {
            let mut v1 = Coords::new(1.0, 2.0, 0.0);
            let v2 = Coords::new(3.0, 4.0, 1.0);

            v1.add(&v2);

            assert_eq!(v1.x, 4.0);
            assert_eq!(v1.y, 6.0);
            assert_eq!(v1.z, 1.0);
        }

        #[test]
        fn works_with_negative_values() {
            let mut v1 = Coords::new(1.0, 2.0, 3.0);
            let v2 = Coords::new(-2.0, -7.0, -3.0);

            v1.add(&v2);

            assert_eq!(v1.x, -1.0);
            assert_eq!(v1.y, -5.0);
            assert_eq!(v1.z, 0.0);
        }

        #[test]
        fn works_with_mixed_values() {
            let mut v1 = Coords::new(1.0, 2.0, 0.0);
            let v2 = Coords::new(-2.0, 7.0, -2.0);

            v1.add(&v2);

            assert_eq!(v1.x, -1.0);
            assert_eq!(v1.y, 9.0);
            assert_eq!(v1.z, -2.0);
        }
    }

    mod values {
        use super::*;

        #[test]
        fn in_correct_order() {
            let v = Coords::new(1.0, 2.0, 3.0);

            assert_eq!(v.values(), (1.0, 2.0, 3.0));
        }
    }
}
