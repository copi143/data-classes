use std::collections::BTreeMap;
use std::collections::HashMap;

use data_classes::*;

#[data]
pub enum Color {
    Red,
    Green,
    Blue,
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
