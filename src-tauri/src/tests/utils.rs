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
