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

    let ScenarioStep::Request(ref s) = scenario.steps[0] else {
        panic!("expected request step");
    };
    assert_eq!(s.id, "step-1");
    assert_eq!(s.request_id, "auth/login");
    assert_eq!(s.extract.len(), 1);
    assert_eq!(s.extract[0].variable, "token");

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
    let ScenarioStep::Request(ref r) = step else {
        panic!("expected request step");
    };
    assert!(r.overrides.is_empty());
    assert!(r.extract.is_empty());
    assert_eq!(r.request_id, "req-id");
}

#[test]
fn test_delay_step_round_trip() {
    let json = r#"{
        "id": "d1",
        "name": "Wait",
        "duration": 300
    }"#;

    let step: ScenarioStep = serde_json::from_str(json).unwrap();
    let ScenarioStep::Delay(ref d) = step else {
        panic!("expected delay step");
    };
    assert_eq!(d.id, "d1");
    assert_eq!(d.name, "Wait");
    assert_eq!(d.duration, 300);

    let serialized = serde_json::to_string(&step).unwrap();
    let re_parsed: ScenarioStep = serde_json::from_str(&serialized).unwrap();
    let ScenarioStep::Delay(ref d2) = re_parsed else {
        panic!("expected delay step after round-trip");
    };
    assert_eq!(d2.duration, 300);
}

#[test]
fn test_legacy_request_step_no_type() {
    // Old JSON files have no "type" field — must deserialize as Request
    let json = r#"{
        "id": "s1",
        "name": "Login",
        "requestId": "auth/login",
        "overrides": {},
        "extract": []
    }"#;

    let step: ScenarioStep = serde_json::from_str(json).unwrap();
    assert!(matches!(step, ScenarioStep::Request(_)), "old step without type field should be Request");
}

#[test]
fn test_mixed_steps_round_trip() {
    let json = r#"{
        "name": "Mixed",
        "steps": [
            {"id": "r1", "name": "Login", "requestId": "auth/login"},
            {"id": "d1", "name": "Wait", "duration": 200},
            {"id": "r2", "name": "Fetch", "requestId": "api/data"}
        ]
    }"#;

    let scenario: Scenario = serde_json::from_str(json).unwrap();
    assert_eq!(scenario.steps.len(), 3);
    assert!(matches!(scenario.steps[0], ScenarioStep::Request(_)));
    assert!(matches!(scenario.steps[1], ScenarioStep::Delay(_)));
    assert!(matches!(scenario.steps[2], ScenarioStep::Request(_)));

    let ScenarioStep::Delay(ref d) = scenario.steps[1] else { panic!() };
    assert_eq!(d.duration, 200);

    // Round-trip
    let serialized = serde_json::to_string(&scenario).unwrap();
    let re_parsed: Scenario = serde_json::from_str(&serialized).unwrap();
    assert!(matches!(re_parsed.steps[0], ScenarioStep::Request(_)));
    assert!(matches!(re_parsed.steps[1], ScenarioStep::Delay(_)));
    assert!(matches!(re_parsed.steps[2], ScenarioStep::Request(_)));
}

#[test]
fn test_pause_step_round_trip() {
    let json = r#"{
        "id": "p1",
        "name": "Wait for user",
        "pause": true
    }"#;

    let step: ScenarioStep = serde_json::from_str(json).unwrap();
    let ScenarioStep::Pause(ref p) = step else {
        panic!("expected pause step");
    };
    assert_eq!(p.id, "p1");
    assert_eq!(p.name, "Wait for user");
    assert!(p.pause);

    let serialized = serde_json::to_string(&step).unwrap();
    let re_parsed: ScenarioStep = serde_json::from_str(&serialized).unwrap();
    let ScenarioStep::Pause(ref p2) = re_parsed else {
        panic!("expected pause step after round-trip");
    };
    assert_eq!(p2.id, "p1");
    assert!(p2.pause);
}

#[test]
fn test_mixed_steps_with_pause() {
    let json = r#"{
        "name": "With Pause",
        "steps": [
            {"id": "r1", "name": "Login", "requestId": "auth/login"},
            {"id": "p1", "name": "Check manually", "pause": true},
            {"id": "r2", "name": "Fetch", "requestId": "api/data"}
        ]
    }"#;

    let scenario: Scenario = serde_json::from_str(json).unwrap();
    assert_eq!(scenario.steps.len(), 3);
    assert!(matches!(scenario.steps[0], ScenarioStep::Request(_)));
    assert!(matches!(scenario.steps[1], ScenarioStep::Pause(_)));
    assert!(matches!(scenario.steps[2], ScenarioStep::Request(_)));

    let ScenarioStep::Pause(ref p) = scenario.steps[1] else { panic!() };
    assert_eq!(p.name, "Check manually");

    // Round-trip
    let serialized = serde_json::to_string(&scenario).unwrap();
    let re_parsed: Scenario = serde_json::from_str(&serialized).unwrap();
    assert!(matches!(re_parsed.steps[0], ScenarioStep::Request(_)));
    assert!(matches!(re_parsed.steps[1], ScenarioStep::Pause(_)));
    assert!(matches!(re_parsed.steps[2], ScenarioStep::Request(_)));
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
