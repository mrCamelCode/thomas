use std::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub struct IntCoords2d {
    x: i64,
    y: i64,
}
impl IntCoords2d {
    pub fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self::new(0, 0)
    }

    pub fn left() -> Self {
        Self::new(-1, 0)
    }

    pub fn right() -> Self {
        Self::new(1, 0)
    }

    pub fn up() -> Self {
        Self::new(0, 1)
    }

    pub fn down() -> Self {
        Self::new(0, -1)
    }
    
    pub fn distance_from(&self, other: &Self) -> f64 {
        let diff_x = self.x as f64 - other.x as f64;
        let diff_y = self.y as f64 - other.y as f64;

        f64::sqrt(diff_x.powf(2.0) + diff_y.powf(2.0))
    }

    pub fn x(&self) -> i64 {
        self.x
    }

    pub fn y(&self) -> i64 {
        self.y
    }

    pub fn values(&self) -> (i64, i64) {
        (self.x, self.y)
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct IntCoords {
    coords2d: IntCoords2d,
    z: i64,
}
impl IntCoords {
    pub fn new(x: i64, y: i64, z: i64) -> Self {
        Self { coords2d: IntCoords2d::new(x, y), z }
    }
    
    pub fn zero() -> Self {
        Self::new(0, 0, 0)
    }

    pub fn left() -> Self {
        Self::new(-1, 0, 0)
    }

    pub fn right() -> Self {
        Self::new(1, 0, 0)
    }

    pub fn up() -> Self {
        Self::new(0, 1, 0)
    }

    pub fn down() -> Self {
        Self::new(0, -1, 0)
    }

    pub fn forward() -> Self {
        Self::new(0, 0, -1)
    }
    pub fn forwards() -> Self {
        Self::forward()
    }

    pub fn backward() -> Self {
        Self::new(0, 0, 1)
    }
    pub fn backwards() -> Self {
        Self::backward()
    }

    pub fn distance_from(&self, other: &Coords) -> f64 {
        let IntCoords2d { x, y } = self.coords2d;
        let IntCoords2d {
            x: other_x,
            y: other_y,
        } = self.coords2d;

        let diff_x = x as f64 - other_x as f64;
        let diff_y = y as f64 - other_y as f64;
        let diff_z = self.z as f64 - other.z as f64;

        f64::sqrt(diff_x.powf(2.0) + diff_y.powf(2.0) + diff_z.powf(2.0))
    }

    pub fn x(&self) -> i64 {
        self.coords2d.x
    }

    pub fn y(&self) -> i64 {
        self.coords2d.y
    }

    pub fn z(&self) -> i64 {
        self.z
    }

    pub fn values(&self) -> (i64, i64, i64) {
        (self.coords2d.x, self.coords2d.y, self.z)
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub struct Coords2d {
    x: f64,
    y: f64,
}
impl Coords2d {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self::new(0.0, 0.0)
    }

    pub fn left() -> Self {
        Self::new(-1.0, 0.0)
    }

    pub fn right() -> Self {
        Self::new(1.0, 0.0)
    }

    pub fn up() -> Self {
        Self::new(0.0, 1.0)
    }

    pub fn down() -> Self {
        Self::new(0.0, -1.0)
    }

    pub fn distance_from(&self, other: &Self) -> f64 {
        let diff_x = self.x - other.x;
        let diff_y = self.y - other.y;

        f64::sqrt(diff_x.powf(2.0) + diff_y.powf(2.0))
    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }

    pub fn values(&self) -> (f64, f64) {
        (self.x, self.y)
    }
}
impl Add for Coords2d {
    type Output = Coords2d;

    fn add(self, rhs: Self) -> Self::Output {
        Coords2d {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
impl Sub for Coords2d {
    type Output = Coords2d;

    fn sub(self, rhs: Self) -> Self::Output {
        Coords2d {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}
impl AddAssign for Coords2d {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}
impl SubAssign for Coords2d {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub struct Coords {
    coords2d: Coords2d,
    z: f64,
}
impl Coords {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self {
            coords2d: Coords2d::new(x, y),
            z,
        }
    }

    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    pub fn left() -> Self {
        Self::new(-1.0, 0.0, 0.0)
    }

    pub fn right() -> Self {
        Self::new(1.0, 0.0, 0.0)
    }

    pub fn up() -> Self {
        Self::new(0.0, 1.0, 0.0)
    }

    pub fn down() -> Self {
        Self::new(0.0, -1.0, 0.0)
    }

    pub fn forward() -> Self {
        Self::new(0.0, 0.0, -1.0)
    }
    pub fn forwards() -> Self {
        Self::forward()
    }

    pub fn backward() -> Self {
        Self::new(0.0, 0.0, 1.0)
    }
    pub fn backwards() -> Self {
        Self::backward()
    }

    pub fn distance_from(&self, other: &Coords) -> f64 {
        let Coords2d { x, y } = self.coords2d;
        let Coords2d {
            x: other_x,
            y: other_y,
        } = self.coords2d;

        let diff_x = x - other_x;
        let diff_y = y - other_y;
        let diff_z = self.z - other.z;

        f64::sqrt(diff_x.powf(2.0) + diff_y.powf(2.0) + diff_z.powf(2.0))
    }

    pub fn coords2d(&self) -> &Coords2d {
        &self.coords2d
    }

    pub fn x(&self) -> f64 {
        self.coords2d.x
    }

    pub fn y(&self) -> f64 {
        self.coords2d.y
    }

    pub fn z(&self) -> f64 {
        self.z
    }

    pub fn values(&self) -> (f64, f64, f64) {
        let (x, y) = self.coords2d.values();

        (x, y, self.z)
    }
}
impl Add for Coords {
    type Output = Coords;

    fn add(self, rhs: Self) -> Self::Output {
        let Coords2d { x, y } = Coords2d::add(self.coords2d, rhs.coords2d);

        Coords {
            coords2d: Coords2d::new(x, y),
            z: self.z + rhs.z,
        }
    }
}
impl Sub for Coords {
    type Output = Coords;

    fn sub(self, rhs: Self) -> Self::Output {
        let Coords2d { x, y } = Coords2d::sub(self.coords2d, rhs.coords2d);

        Coords {
            coords2d: Coords2d::new(x, y),
            z: self.z - rhs.z,
        }
    }
}
impl AddAssign for Coords {
    fn add_assign(&mut self, rhs: Self) {
        Coords2d::add_assign(&mut self.coords2d, rhs.coords2d);

        self.z += rhs.z;
    }
}
impl SubAssign for Coords {
    fn sub_assign(&mut self, rhs: Self) {
        Coords2d::sub_assign(&mut self.coords2d, rhs.coords2d);

        self.z -= rhs.z;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod add {
        use super::*;

        #[test]
        fn no_change() {
            let v1 = Coords::new(1.0, 2.0, 0.0);
            let v2 = Coords::zero();

            let v3 = v1 + v2;

            assert_eq!(v3.x(), 1.0);
            assert_eq!(v3.y(), 2.0);
        }

        #[test]
        fn works_with_positive_values() {
            let v1 = Coords::new(1.0, 2.0, 0.0);
            let v2 = Coords::new(3.0, 4.0, 1.0);

            let v3 = v1 + v2;

            assert_eq!(v3.x(), 4.0);
            assert_eq!(v3.y(), 6.0);
            assert_eq!(v3.z(), 1.0);
        }

        #[test]
        fn works_with_negative_values() {
            let v1 = Coords::new(1.0, 2.0, 3.0);
            let v2 = Coords::new(-2.0, -7.0, -3.0);

            let v3 = v1 + v2;

            assert_eq!(v3.x(), -1.0);
            assert_eq!(v3.y(), -5.0);
            assert_eq!(v3.z(), 0.0);
        }

        #[test]
        fn works_with_mixed_values() {
            let v1 = Coords::new(1.0, 2.0, 0.0);
            let v2 = Coords::new(-2.0, 7.0, -2.0);

            let v3 = v1 + v2;

            assert_eq!(v3.x(), -1.0);
            assert_eq!(v3.y(), 9.0);
            assert_eq!(v3.z(), -2.0);
        }
    }

    mod subtract {
        use super::*;

        #[test]
        fn no_change() {
            let v1 = Coords::new(1.0, 2.0, 0.0);
            let v2 = Coords::zero();

            let v3 = v1 - v2;

            assert_eq!(v3.x(), 1.0);
            assert_eq!(v3.y(), 2.0);
        }

        #[test]
        fn works_with_positive_values() {
            let v1 = Coords::new(1.0, 2.0, 0.0);
            let v2 = Coords::new(3.0, 4.0, 1.0);

            let v3 = v1 - v2;

            assert_eq!(v3.x(), -2.0);
            assert_eq!(v3.y(), -2.0);
            assert_eq!(v3.z, -1.0);
        }

        #[test]
        fn works_with_negative_values() {
            let v1 = Coords::new(1.0, 2.0, 3.0);
            let v2 = Coords::new(-2.0, -7.0, -3.0);

            let v3 = v1 - v2;

            assert_eq!(v3.x(), 3.0);
            assert_eq!(v3.y(), 9.0);
            assert_eq!(v3.z(), 6.0);
        }

        #[test]
        fn works_with_mixed_values() {
            let v1 = Coords::new(1.0, 2.0, 0.0);
            let v2 = Coords::new(-2.0, 7.0, -2.0);

            let v3 = v1 - v2;

            assert_eq!(v3.x(), 3.0);
            assert_eq!(v3.y(), -5.0);
            assert_eq!(v3.z(), 2.0);
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
