use data_classes::derive::*;

#[data(validate, no-eq)]
struct Limits {
    #[check = *n <= 3]
    n: u8,
}

#[data]
struct Access {
    #[set]
    #[check = *n <= 10]
    n: u8,
    #[with]
    #[check = name.len() > 0]
    name: String,
}

#[data(builder)]
struct BuilderDemo {
    #[builder(default)]
    tags: Vec<String>,
    #[check = name.len() > 0]
    name: String,
}

#[cfg(test)]
mod tests {
    use super::{Access, BuilderDemo, Limits};

    #[test]
    #[should_panic(expected = "check failed for field n")]
    fn test_set_check_failure() {
        let mut a = Access {
            n: 1,
            name: "x".to_string(),
        };
        a.set_n(99);
    }

    #[test]
    #[should_panic(expected = "check failed for field name")]
    fn test_with_check_failure() {
        let a = Access {
            n: 1,
            name: "x".to_string(),
        };
        let _ = a.with_name("".to_string());
    }

    #[test]
    #[should_panic(expected = "missing field name")]
    fn test_builder_missing_required() {
        let _ = BuilderDemo::builder()
            .with_tags(vec!["a".to_string()])
            .build();
    }

    #[test]
    #[should_panic(expected = "check failed for field name")]
    fn test_builder_check_failure() {
        let _ = BuilderDemo::builder().with_name("".to_string()).build();
    }

    #[test]
    fn test_validate_pass_fail() {
        let ok = Limits { n: 2 };
        let bad = Limits { n: 9 };
        assert!(ok.validate());
        assert!(!bad.validate());
    }
}
