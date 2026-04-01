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

pub fn generate_default_body(schema: &Value, import_timestamp: &str) -> Value {
    let _ = (schema, import_timestamp);
    Value::Null // stub — implemented in Task 3
}

#[cfg(test)]
#[path = "tests/schema_defaults.rs"]
mod tests;
