use std::collections::BTreeMap;
use std::collections::HashMap;

use data_classes::*;

#[data]
pub enum Color {
    Red,
    Green,
    Blue,
}

// #[data(new, default, serde, rkyv(cmp), pod)]
// pub struct Color1 {
//     r: u8,
//     g: u8,
//     b: u8,
// }

#[repr(C)]
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Default,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    serde::Serialize,
    serde::Deserialize,
    bytemuck::Pod,
    bytemuck::Zeroable,
    Copy,
)]
#[rkyv(derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash,))]
#[rkyv(compare(PartialEq, PartialOrd))]
pub struct Color1 {
    r: u8,
    g: u8,
    b: u8,
}
impl Color1 {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
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

fn main() {
    let mut a: BTreeMap<MyPoint, i32> = BTreeMap::new();
    a.insert(MyPoint { x: 1, y: 2 }, 10);
    let p1 = Point { x: 10, y: 20 };
    let p2 = Point { x: 10, y: 20 };
    let p3 = Point { x: 15, y: 25 };

    // Test equality
    assert!(p1 == p2);
    assert!(p1 != p3);

    // Test cloning
    let p4 = p1.clone();
    assert!(p1 == p4);

    // Test debug representation
    println!("{:?}", p1);
}
