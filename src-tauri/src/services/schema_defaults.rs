use serde_json::Value;

const MAX_REF_DEPTH: u8 = 10;

/// Recursively resolves all `$ref` pointers in `schema` by inlining the
/// referenced component from `spec`. Depth-guarded at 10 levels to handle
/// circular references without panicking.
pub fn resolve_refs(schema: &Value, spec: &Value, depth: u8) -> Value {
    if depth >= MAX_REF_DEPTH {
        return Value::Null;
    }

    if let Some(ref_path) = schema.get("$ref").and_then(|v| v.as_str()) {
        if let Some(pointer) = ref_path.strip_prefix('#') {
            if let Some(target) = spec.pointer(pointer) {
                return resolve_refs(target, spec, depth + 1);
            }
        }
        // External/relative refs (e.g. "Pet.yaml") are out of scope — return null.
        return Value::Null;
    }

    match schema {
        Value::Object(obj) => {
            let mut result = serde_json::Map::new();
            for (key, value) in obj {
                result.insert(key.clone(), resolve_refs(value, spec, depth));
            }
            Value::Object(result)
        }
        Value::Array(arr) => {
            Value::Array(arr.iter().map(|v| resolve_refs(v, spec, depth)).collect())
        }
        other => other.clone(),
    }
}

/// Generates a default JSON body value from a fully-resolved JSON Schema.
/// `import_timestamp` is an ISO 8601 string used as the default for `date-time` fields.
pub fn generate_default_body(schema: &Value, import_timestamp: &str) -> Value {
    generate_value(schema, true, import_timestamp)
}

fn generate_value(schema: &Value, required: bool, import_timestamp: &str) -> Value {
    if !required {
        return Value::Null;
    }

    // Enum takes precedence over type
    if let Some(enum_values) = schema.get("enum").and_then(|v| v.as_array()) {
        if let Some(first) = enum_values.first() {
            return first.clone();
        }
    }

    match schema.get("type").and_then(|v| v.as_str()) {
        Some("object") => {
            let props = match schema.get("properties").and_then(|v| v.as_object()) {
                Some(p) => p,
                None => return Value::Object(serde_json::Map::new()),
            };
            // `required` array takes precedence; fall back to `nullable: true` only when
            // no `required` array is present (common in .NET-generated specs).
            let required_fields = schema.get("required").and_then(|v| v.as_array());
            let mut obj = serde_json::Map::new();
            for (name, field_schema) in props {
                let field_required = if let Some(req) = required_fields {
                    req.iter().any(|r| r.as_str() == Some(name.as_str()))
                } else {
                    !field_schema
                        .get("nullable")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                };
                obj.insert(
                    name.clone(),
                    generate_value(field_schema, field_required, import_timestamp),
                );
            }
            Value::Object(obj)
        }
        Some("array") => match schema.get("items") {
            Some(items) => Value::Array(vec![generate_value(items, true, import_timestamp)]),
            // No items schema — return empty array rather than [null]
            None => Value::Array(vec![]),
        },
        Some("string") => match schema.get("format").and_then(|v| v.as_str()) {
            Some("uuid") => Value::String(uuid::Uuid::new_v4().to_string()),
            Some("date-time") => Value::String(import_timestamp.to_string()),
            _ => Value::String(String::new()),
        },
        Some("integer") | Some("number") => Value::Number(serde_json::Number::from(0)),
        Some("boolean") => Value::Bool(false),
        _ => Value::Null,
    }
}

#[cfg(test)]
#[path = "tests/schema_defaults.rs"]
mod tests;
