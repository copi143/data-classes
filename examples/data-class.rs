use std::collections::BTreeMap;
use std::collections::HashMap;

use data_classes::*;

#[data]
pub enum Color {
    Red,
    Green,
    Blue,
}

#[data(new, serde, rkyv(cmp, omit-bounds))]
struct Point {
    x: i32,
    y: i32,
}

/// aaa
#[default]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
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
