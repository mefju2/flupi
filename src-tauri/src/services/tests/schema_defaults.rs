use super::*;

#[test]
fn test_resolve_refs_no_ref_returns_unchanged() {
    let schema = serde_json::json!({"type": "string"});
    let spec = serde_json::json!({});
    let result = resolve_refs(&schema, &spec, 0);
    assert_eq!(result, schema);
}

#[test]
fn test_resolve_refs_resolves_component_ref() {
    let spec = serde_json::json!({
        "components": {
            "schemas": {
                "MySchema": {"type": "string"}
            }
        }
    });
    let schema = serde_json::json!({"$ref": "#/components/schemas/MySchema"});
    let result = resolve_refs(&schema, &spec, 0);
    assert_eq!(result, serde_json::json!({"type": "string"}));
}

#[test]
fn test_resolve_refs_resolves_nested_property_ref() {
    let spec = serde_json::json!({
        "components": {
            "schemas": {
                "Inner": {"type": "integer"},
                "Outer": {
                    "type": "object",
                    "properties": {
                        "value": {"$ref": "#/components/schemas/Inner"}
                    }
                }
            }
        }
    });
    let schema = serde_json::json!({"$ref": "#/components/schemas/Outer"});
    let result = resolve_refs(&schema, &spec, 0);
    assert_eq!(result["properties"]["value"]["type"], "integer");
}

#[test]
fn test_resolve_refs_unknown_ref_returns_null() {
    let spec = serde_json::json!({"components": {"schemas": {}}});
    let schema = serde_json::json!({"$ref": "#/components/schemas/DoesNotExist"});
    let result = resolve_refs(&schema, &spec, 0);
    assert_eq!(result, serde_json::Value::Null);
}

#[test]
fn test_resolve_refs_depth_guard_returns_null() {
    // Circular: A → A. At depth 9 we try to follow $ref which hits depth 10 → Null.
    let spec = serde_json::json!({
        "components": {
            "schemas": {
                "A": {"$ref": "#/components/schemas/A"}
            }
        }
    });
    let schema = serde_json::json!({"$ref": "#/components/schemas/A"});
    // Calling with depth = 9: resolves A (depth becomes 10 on recursive call → Null)
    let result = resolve_refs(&schema, &spec, 9);
    assert_eq!(result, serde_json::Value::Null);
}
