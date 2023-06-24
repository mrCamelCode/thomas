/// Where the UI element is anchored on the screen. The anchor represents where the element is positioned by default
/// when it has no offset.
#[derive(PartialEq, Eq, Debug, Hash)]
pub enum UiAnchor {
    TopLeft,
    MiddleTop,
    TopRight,
    MiddleRight,
    BottomRight,
    MiddleBottom,
    BottomLeft,
    MiddleLeft,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Alignment {
    Left,
    Middle,
    Right,
}

/// Represents an RGB color.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Rgb(pub u8, pub u8, pub u8);
impl Rgb {
    /// The red value of the color.
    pub fn r(&self) -> u8 {
        self.0
    }

    /// The green value of the color.
    pub fn g(&self) -> u8 {
        self.1
    }

    /// The blue value of the color.
    pub fn b(&self) -> u8 {
        self.2
    }

    pub fn white() -> Self {
        Self(255, 255, 255)
    }

    pub fn black() -> Self {
        Self(0, 0, 0)
    }

    pub fn red() -> Self {
        Self(255, 0, 0)
    }

    pub fn green() -> Self {
        Self(0, 255, 0)
    }

    pub fn blue() -> Self {
        Self(0, 0, 255)
    }

    pub fn cyan() -> Self {
        Self(0, 255, 255)
    }

    pub fn magenta() -> Self {
        Self(255, 0, 255)
    }

    pub fn yellow() -> Self {
        Self(255, 255, 0)
    }
}
impl Lerp for Rgb {
    type Item = Rgb;

    fn lerp(start: &Self::Item, target: &Self::Item, interpolation_ratio: f32) -> Self::Item {
        Rgb(
            u8::lerp(&start.r(), &target.r(), interpolation_ratio),
            u8::lerp(&start.g(), &target.g(), interpolation_ratio),
            u8::lerp(&start.b(), &target.b(), interpolation_ratio),
        )
    }
}

pub trait Lerp {
    type Item;

    fn lerp(start: &Self::Item, target: &Self::Item, interpolation_ratio: f32) -> Self::Item;
}

impl Lerp for u8 {
    type Item = u8;

    fn lerp(start: &Self::Item, target: &Self::Item, interpolation_ratio: f32) -> Self::Item {
        // Implementation inspired by https://docs.unity3d.com/ScriptReference/Vector3.Lerp.html
        // Using the formula: a + (b - a) * t
        let a = *start as f32;
        let b = *target as f32;
        let t = interpolation_ratio.clamp(0.0, 1.0);

        (a + (b - a) * t).round() as u8
    }
}
