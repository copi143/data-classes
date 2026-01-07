use data_classes::derive::*;

#[data]
pub struct Wrapper<T> {
    #[deref]
    inner: T,
}

#[data]
pub struct WrapperMut<T> {
    #[deref(mut)]
    inner: T,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deref() {
        let w = Wrapper {
            inner: "hi".to_string(),
        };
        assert_eq!(w.len(), 2);
    }

    #[test]
    fn test_deref_mut() {
        let mut w = WrapperMut { inner: vec![1, 2] };
        w.push(3);
        assert_eq!(w.len(), 3);
    }
}
