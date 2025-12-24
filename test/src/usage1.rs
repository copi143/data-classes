use std::collections::BTreeMap;
use std::collections::HashMap;

use data_classes::*;

#[data]
pub enum Color {
    Red,
    Green,
    Blue,
}

#[data(new, default, serde, rkyv(cmp), pod)]
pub struct Color1 {
    r: u8,
    g: u8,
    b: u8,
}

// If we derive Pod, Zeroable and Copy will be automatically added.
#[data(new, default, display(comma), serde, rkyv(cmp), pod)]
struct Point {
    #[default = 1]
    x: i32,
    #[default = 1]
    y: i32,
}

// If we derive Pod, Zeroable and Copy will be automatically added.
#[data(new, default, display(comma), serde, rkyv(cmp), pod)]
struct Point3D {
    #[default = 1]
    #[serde(default)]
    x: i32,
    #[default = 1]
    #[serde(default)]
    y: i32,
    #[default = 1]
    #[serde(default)]
    z: i32,
    #[new = _]
    #[serde(skip)]
    _pad: i32,
}

/// aaa
#[default]
#[data(copy)]
struct MyPoint {
    #[default = 1]
    x: i32,
    #[default = 1+1]
    y: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color() {
        let r = Color::Red;
        let g = Color::Green;
        let b = Color::Blue;
        assert!(r != g);
        assert!(g != b);
        assert!(b != r);
    }

    #[test]
    fn test_point() {
        let p = Point::default();
        assert_eq!(p.x, 1);
        assert_eq!(p.y, 1);
        let p = Point::new(3, 4);
        assert_eq!(p.x, 3);
        assert_eq!(p.y, 4);
    }
}
