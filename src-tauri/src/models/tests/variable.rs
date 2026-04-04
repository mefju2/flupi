use super::*;

#[test]
fn test_variable_context_set_and_get() {
    let mut ctx = VariableContext::new();
    ctx.set("token", "abc123");
    assert_eq!(ctx.get("token"), Some("abc123"));
    assert_eq!(ctx.get("missing"), None);
}

#[test]
fn test_variable_context_overwrite() {
    let mut ctx = VariableContext::new();
    ctx.set("key", "old");
    ctx.set("key", "new");
    assert_eq!(ctx.get("key"), Some("new"));
}

#[test]
fn test_variable_context_mark_secret() {
    let mut ctx = VariableContext::new();
    ctx.set("password", "s3cr3t");
    assert!(!ctx.is_secret("password"));
    ctx.mark_secret("password");
    assert!(ctx.is_secret("password"));
    assert!(!ctx.is_secret("token"));
}

#[test]
fn test_variable_context_all_keys() {
    let mut ctx = VariableContext::new();
    ctx.set("a", "1");
    ctx.set("b", "2");
    let mut keys = ctx.all_keys();
    keys.sort();
    assert_eq!(keys, vec!["a", "b"]);
}

#[test]
fn test_variable_context_default_is_empty() {
    let ctx = VariableContext::default();
    assert_eq!(ctx.all_keys().len(), 0);
}

#[test]
fn test_variable_context_clone() {
    let mut ctx = VariableContext::new();
    ctx.set("x", "1");
    let cloned = ctx.clone();
    assert_eq!(cloned.get("x"), Some("1"));
}
