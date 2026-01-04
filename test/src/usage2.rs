use data_classes::{deps::*, derive::*};

init_struct! {
    pub struct Foo {
        pub a: i32 = 42,
        pub b: String = "hello".to_string(),
        c: u8,
    }
}

fn main() {
    let f = Foo::new();
    assert_eq!(f.a, 42);
    assert_eq!(f.b, "hello");
    // c wasn't given an initializer, so it uses Default::default() which is 0 for u8
    assert_eq!(f.c, 0);
    println!("example OK: a={}, b={}, c={}", f.a, f.b, f.c);
}
