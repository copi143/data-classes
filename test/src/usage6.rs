use data_classes::derive::*;

#[data(builder)]
struct Project {
    #[builder(default)]
    tags: Vec<String>,
    #[check = name.len() > 0]
    name: String,
}

#[cfg(test)]
mod tests {
    use super::Project;

    #[test]
    fn test_builder_success() {
        let p = Project::builder()
            .with_name("demo".to_string())
            .with_tags(vec!["a".to_string()])
            .build();
        assert_eq!(p.name, "demo");
        assert_eq!(p.tags.len(), 1);
    }
}
