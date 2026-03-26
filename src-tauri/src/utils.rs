/// Converts a human-readable name into a URL/filesystem-safe slug.
/// Lowercases the name and replaces spaces with hyphens.
pub fn name_to_slug(name: &str) -> String {
    name.to_lowercase().replace(' ', "-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name_to_slug_basic() {
        assert_eq!(name_to_slug("Auth Service"), "auth-service");
    }

    #[test]
    fn test_name_to_slug_already_slug() {
        assert_eq!(name_to_slug("health-check"), "health-check");
    }

    #[test]
    fn test_name_to_slug_uppercase() {
        assert_eq!(name_to_slug("My API"), "my-api");
    }
}
