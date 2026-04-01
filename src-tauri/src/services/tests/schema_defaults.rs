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

// ── generate_default_body tests ──────────────────────────────────────────────

#[test]
fn test_default_body_string() {
    let schema = serde_json::json!({"type": "string"});
    assert_eq!(generate_default_body(&schema, "ts"), serde_json::json!(""));
}

#[test]
fn test_default_body_string_uuid_is_valid_uuid() {
    let schema = serde_json::json!({"type": "string", "format": "uuid"});
    let result = generate_default_body(&schema, "ts");
    let s = result.as_str().expect("should be a string");
    uuid::Uuid::parse_str(s).expect("should be a valid UUID v4");
}

#[test]
fn test_default_body_string_datetime_uses_timestamp() {
    let schema = serde_json::json!({"type": "string", "format": "date-time"});
    let result = generate_default_body(&schema, "2026-04-01T12:00:00Z");
    assert_eq!(result, serde_json::json!("2026-04-01T12:00:00Z"));
}

#[test]
fn test_default_body_integer() {
    let schema = serde_json::json!({"type": "integer"});
    assert_eq!(generate_default_body(&schema, "ts"), serde_json::json!(0));
}

#[test]
fn test_default_body_number() {
    let schema = serde_json::json!({"type": "number"});
    assert_eq!(generate_default_body(&schema, "ts"), serde_json::json!(0));
}

#[test]
fn test_default_body_boolean() {
    let schema = serde_json::json!({"type": "boolean"});
    assert_eq!(generate_default_body(&schema, "ts"), serde_json::json!(false));
}

#[test]
fn test_default_body_enum_uses_first_value() {
    let schema = serde_json::json!({"type": "string", "enum": ["Planned", "InProgress", "Done"]});
    assert_eq!(generate_default_body(&schema, "ts"), serde_json::json!("Planned"));
}

#[test]
fn test_default_body_object_with_required_array() {
    // Fields in `required` get type defaults; others get null
    let schema = serde_json::json!({
        "type": "object",
        "required": ["name"],
        "properties": {
            "name": {"type": "string"},
            "description": {"type": "string"}
        }
    });
    let result = generate_default_body(&schema, "ts");
    assert_eq!(result["name"], serde_json::json!(""));
    assert_eq!(result["description"], serde_json::Value::Null);
}

#[test]
fn test_default_body_dotnet_style_nullable_means_optional() {
    // .NET-generated specs: no `required` array; nullable: true = optional
    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "id": {"type": "string", "format": "uuid"},
            "name": {"type": "string", "nullable": true}
        }
    });
    let result = generate_default_body(&schema, "ts");
    // id: not nullable → required → valid UUID
    uuid::Uuid::parse_str(result["id"].as_str().unwrap())
        .expect("id should be a valid UUID");
    // name: nullable → null
    assert_eq!(result["name"], serde_json::Value::Null);
}

#[test]
fn test_default_body_array_with_string_items() {
    let schema = serde_json::json!({"type": "array", "items": {"type": "string"}});
    assert_eq!(generate_default_body(&schema, "ts"), serde_json::json!([""]));
}

#[test]
fn test_default_body_array_with_object_items() {
    let schema = serde_json::json!({
        "type": "array",
        "items": {
            "type": "object",
            "properties": {
                "count": {"type": "integer"}
            }
        }
    });
    let result = generate_default_body(&schema, "ts");
    // Array with one item, that item has count=0 (non-nullable → required)
    assert_eq!(result[0]["count"], serde_json::json!(0));
}

#[test]
fn test_default_body_nested_object() {
    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "address": {
                "type": "object",
                "properties": {
                    "zip": {"type": "string"}
                }
            }
        }
    });
    let result = generate_default_body(&schema, "ts");
    // address: not nullable → required → recurse
    assert_eq!(result["address"]["zip"], serde_json::json!(""));
}
