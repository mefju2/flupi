use super::*;

#[test]
fn test_scenario_round_trip() {
    let json = r#"{
        "name": "Auth Flow",
        "inputs": [
            {
                "name": "username",
                "description": "Test user",
                "default": "admin",
                "required": true
            }
        ],
        "steps": [
            {
                "id": "step-1",
                "name": "Login",
                "requestId": "auth/login",
                "overrides": {"path": "/api/v2/login"},
                "extract": [
                    {
                        "variable": "token",
                        "from": "response.body",
                        "path": "$.data.token"
                    }
                ]
            }
        ]
    }"#;

    let scenario: Scenario = serde_json::from_str(json).unwrap();
    assert_eq!(scenario.name, "Auth Flow");
    assert_eq!(scenario.inputs.len(), 1);
    assert_eq!(scenario.inputs[0].name, "username");
    assert_eq!(scenario.inputs[0].default, "admin");
    assert!(scenario.inputs[0].required);
    assert_eq!(scenario.steps.len(), 1);
    assert_eq!(scenario.steps[0].id, "step-1");
    assert_eq!(scenario.steps[0].request_id, "auth/login");
    assert_eq!(scenario.steps[0].extract.len(), 1);
    assert_eq!(scenario.steps[0].extract[0].variable, "token");

    let serialized = serde_json::to_string(&scenario).unwrap();
    let re_parsed: Scenario = serde_json::from_str(&serialized).unwrap();
    assert_eq!(re_parsed.name, scenario.name);
    assert_eq!(re_parsed.steps.len(), scenario.steps.len());
}

#[test]
fn test_scenario_defaults() {
    let json = r#"{
        "name": "Minimal",
        "steps": []
    }"#;

    let scenario: Scenario = serde_json::from_str(json).unwrap();
    assert_eq!(scenario.name, "Minimal");
    assert!(scenario.inputs.is_empty());
    assert!(scenario.steps.is_empty());
}

#[test]
fn test_scenario_input_required_defaults_true() {
    let json = r#"{
        "name": "name",
        "description": "",
        "default": ""
    }"#;

    let input: ScenarioInput = serde_json::from_str(json).unwrap();
    assert!(input.required);
}

#[test]
fn test_step_defaults() {
    let json = r#"{
        "id": "s1",
        "name": "Step 1",
        "requestId": "req-id"
    }"#;

    let step: ScenarioStep = serde_json::from_str(json).unwrap();
    assert!(step.overrides.is_empty());
    assert!(step.extract.is_empty());
    assert_eq!(step.request_id, "req-id");
}

#[test]
fn test_extraction_round_trip() {
    let ext = Extraction {
        variable: "token".to_string(),
        from: "response.body".to_string(),
        path: "$.auth.token".to_string(),
        scope: "scenario".to_string(),
    };

    let json = serde_json::to_string(&ext).unwrap();
    let parsed: Extraction = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.variable, "token");
    assert_eq!(parsed.from, "response.body");
    assert_eq!(parsed.path, "$.auth.token");
}
