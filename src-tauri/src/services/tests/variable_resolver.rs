use super::*;

#[test]
fn test_resolve_simple_variable() {
    let mut ctx = VariableContext::new();
    ctx.set("baseUrl", "https://api.dev");
    let result = resolve_string("{{baseUrl}}/auth", &ctx);
    assert_eq!(result, "https://api.dev/auth");
}

#[test]
fn test_resolve_multiple_variables() {
    let mut ctx = VariableContext::new();
    ctx.set("host", "api.dev");
    ctx.set("token", "abc123");
    let result = resolve_string("https://{{host}}/auth?token={{token}}", &ctx);
    assert_eq!(result, "https://api.dev/auth?token=abc123");
}

#[test]
fn test_unresolved_variable_preserved() {
    let ctx = VariableContext::new();
    let result = resolve_string("{{missing}}", &ctx);
    assert_eq!(result, "{{missing}}");
}

#[test]
fn test_list_unresolved_variables() {
    let mut ctx = VariableContext::new();
    ctx.set("host", "api.dev");
    let unresolved = find_unresolved("{{host}}/{{path}}", &ctx);
    assert_eq!(unresolved, vec!["path"]);
}

#[test]
fn test_priority_order() {
    let mut ctx = VariableContext::new();
    ctx.set("key", "env-value");
    ctx.set("key", "input-value");
    let result = resolve_string("{{key}}", &ctx);
    assert_eq!(result, "input-value");
}
