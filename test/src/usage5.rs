use data_classes::derive::*;

#[data(validate)]
struct User {
    #[check = name.len() > 0]
    name: String,
    #[check = *age >= 18]
    age: u8,
}

#[data]
struct Settings {
    #[get]
    name: String,
    #[get(mut)]
    flags: Vec<String>,
    #[set]
    #[check = *level <= 5]
    level: u8,
    #[get]
    #[with]
    #[check = tag.len() < 8]
    tag: String,
}

#[cfg(test)]
mod tests {
    use super::{Settings, User};

    #[test]
    fn test_validate_success() {
        let u = User {
            name: "alice".to_string(),
            age: 20,
        };
        assert!(u.validate());
    }

    #[test]
    fn test_validate_failure() {
        let u = User {
            name: "".to_string(),
            age: 16,
        };
        assert!(!u.validate());
    }

    #[test]
    fn test_accessors_with_checks() {
        let mut s = Settings {
            name: "base".to_string(),
            flags: vec!["a".to_string()],
            level: 1,
            tag: "ok".to_string(),
        };
        assert_eq!(s.get_name(), "base");
        s.get_flags_mut().push("b".to_string());
        assert_eq!(s.get_flags().len(), 2);
        s.set_level(3);
        let s = s.with_tag("short".to_string());
        assert_eq!(s.get_tag(), "short");
    }
}
