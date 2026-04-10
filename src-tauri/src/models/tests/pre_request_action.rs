use super::*;

#[test]
fn deserialize_set_variable() {
    let json =
        r#"{"type":"set_variable","variable":"myVar","function_name":"myFn","args":["a","b"]}"#;
    let action: PreRequestAction = serde_json::from_str(json).unwrap();
    assert!(
        matches!(action, PreRequestAction::SetVariable { variable, .. } if variable == "myVar")
    );
}

#[test]
fn deserialize_set_variable_no_args() {
    let json = r#"{"type":"set_variable","variable":"x","function_name":"fn1"}"#;
    let action: PreRequestAction = serde_json::from_str(json).unwrap();
    let PreRequestAction::SetVariable { args, .. } = action;
    assert!(args.is_empty());
}

#[test]
fn roundtrip() {
    let action = PreRequestAction::SetVariable {
        variable: "token".to_string(),
        function_name: "generateToken".to_string(),
        args: vec!["arg1".to_string()],
    };
    let json = serde_json::to_string(&action).unwrap();
    let back: PreRequestAction = serde_json::from_str(&json).unwrap();
    assert_eq!(action, back);
}
