use super::*;
use indexmap::IndexMap;

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

#[test]
fn test_resolve_path_params_literal() {
    let mut params = IndexMap::new();
    params.insert("id".to_string(), "42".to_string());
    let ctx = VariableContext::new();
    assert_eq!(resolve_path_params("/users/{id}", &params, &ctx), "/users/42");
}

#[test]
fn test_resolve_path_params_variable_reference() {
    let mut params = IndexMap::new();
    params.insert("id".to_string(), "{{userId}}".to_string());
    let mut ctx = VariableContext::new();
    ctx.set("userId", "99");
    assert_eq!(resolve_path_params("/users/{id}", &params, &ctx), "/users/99");
}

#[test]
fn test_resolve_path_params_missing_preserved() {
    let params = IndexMap::new();
    let ctx = VariableContext::new();
    assert_eq!(resolve_path_params("/users/{id}", &params, &ctx), "/users/{id}");
}

#[test]
fn test_resolve_path_params_multiple() {
    let mut params = IndexMap::new();
    params.insert("org".to_string(), "acme".to_string());
    params.insert("userId".to_string(), "{{uid}}".to_string());
    let mut ctx = VariableContext::new();
    ctx.set("uid", "123");
    assert_eq!(
        resolve_path_params("/orgs/{org}/users/{userId}", &params, &ctx),
        "/orgs/acme/users/123"
    );
}
