use super::*;

#[test]
fn test_script_function_round_trip() {
    let json = r#"{
        "name": "randomInt",
        "body": "return Math.floor(Math.random() * max);",
        "params": [{"name": "max", "param_type": "number"}]
    }"#;

    let func: ScriptFunction = serde_json::from_str(json).unwrap();
    assert_eq!(func.name, "randomInt");
    assert_eq!(func.params.len(), 1);
    assert_eq!(func.params[0].name, "max");
    assert_eq!(func.params[0].param_type, ParamType::Number);

    let serialized = serde_json::to_string(&func).unwrap();
    let re_parsed: ScriptFunction = serde_json::from_str(&serialized).unwrap();
    assert_eq!(re_parsed.name, func.name);
    assert_eq!(re_parsed.params.len(), func.params.len());
}

#[test]
fn test_script_function_params_default_empty() {
    let json = r#"{"name": "noParams", "body": "return 1;"}"#;
    let func: ScriptFunction = serde_json::from_str(json).unwrap();
    assert!(func.params.is_empty());
}

#[test]
fn test_param_type_serialization() {
    assert_eq!(
        serde_json::to_string(&ParamType::String).unwrap(),
        "\"string\""
    );
    assert_eq!(
        serde_json::to_string(&ParamType::Number).unwrap(),
        "\"number\""
    );
    assert_eq!(
        serde_json::to_string(&ParamType::Boolean).unwrap(),
        "\"boolean\""
    );
}

#[test]
fn test_param_type_equality() {
    assert_eq!(ParamType::String, ParamType::String);
    assert_ne!(ParamType::String, ParamType::Number);
}
