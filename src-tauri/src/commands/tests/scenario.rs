use super::*;
use tempfile::TempDir;

fn setup_dir() -> TempDir {
    TempDir::new().unwrap()
}

#[test]
fn test_create_scenario_root() {
    let dir = setup_dir();
    let root = dir.path().to_path_buf();

    let id = create_scenario(root.clone(), None, "Login Flow".to_string()).unwrap();
    assert_eq!(id, "login-flow");
    assert!(root.join("scenarios/login-flow.json").exists());
}

#[test]
fn test_create_scenario_in_group() {
    let dir = setup_dir();
    let root = dir.path().to_path_buf();

    let id = create_scenario(root.clone(), Some("auth".to_string()), "Register".to_string()).unwrap();
    assert_eq!(id, "auth/register");
    assert!(root.join("scenarios/auth/register.json").exists());
}

#[test]
fn test_get_scenario() {
    let dir = setup_dir();
    let root = dir.path().to_path_buf();

    create_scenario(root.clone(), None, "My Scenario".to_string()).unwrap();
    let scenario = get_scenario(root.clone(), "my-scenario".to_string()).unwrap();
    assert_eq!(scenario.name, "My Scenario");
    assert!(scenario.steps.is_empty());
}

#[test]
fn test_save_scenario() {
    use crate::models::scenario::{ScenarioStep, Scenario};
    use std::collections::HashMap;

    let dir = setup_dir();
    let root = dir.path().to_path_buf();

    create_scenario(root.clone(), None, "Flow".to_string()).unwrap();

    let updated = Scenario {
        name: "Flow".to_string(),
        inputs: vec![],
        steps: vec![ScenarioStep {
            id: "step-1".to_string(),
            name: "Step One".to_string(),
            request_id: "req/login".to_string(),
            overrides: HashMap::new(),
            extract: vec![],
        }],
    };
    save_scenario(root.clone(), "flow".to_string(), updated).unwrap();

    let loaded = get_scenario(root.clone(), "flow".to_string()).unwrap();
    assert_eq!(loaded.steps.len(), 1);
    assert_eq!(loaded.steps[0].id, "step-1");
}

#[test]
fn test_delete_scenario() {
    let dir = setup_dir();
    let root = dir.path().to_path_buf();

    create_scenario(root.clone(), None, "Temp".to_string()).unwrap();
    assert!(root.join("scenarios/temp.json").exists());

    delete_scenario(root.clone(), "temp".to_string()).unwrap();
    assert!(!root.join("scenarios/temp.json").exists());
}

#[test]
fn test_rename_scenario() {
    let dir = setup_dir();
    let root = dir.path().to_path_buf();

    create_scenario(root.clone(), None, "Old Name".to_string()).unwrap();
    let new_id = rename_scenario(root.clone(), "old-name".to_string(), "New Name".to_string()).unwrap();

    assert_eq!(new_id, "new-name");
    assert!(!root.join("scenarios/old-name.json").exists());
    assert!(root.join("scenarios/new-name.json").exists());

    let scenario = get_scenario(root.clone(), "new-name".to_string()).unwrap();
    assert_eq!(scenario.name, "New Name");
}

#[test]
fn test_duplicate_scenario() {
    let dir = setup_dir();
    let root = dir.path().to_path_buf();

    create_scenario(root.clone(), None, "Original".to_string()).unwrap();
    let copy_id = duplicate_scenario(root.clone(), "original".to_string()).unwrap();

    assert_eq!(copy_id, "original-copy");
    assert!(root.join("scenarios/original.json").exists());
    assert!(root.join("scenarios/original-copy.json").exists());

    let copy = get_scenario(root.clone(), "original-copy".to_string()).unwrap();
    assert_eq!(copy.name, "Original copy");
}

#[test]
fn test_duplicate_scenario_collision() {
    let dir = setup_dir();
    let root = dir.path().to_path_buf();

    create_scenario(root.clone(), None, "Flow".to_string()).unwrap();
    duplicate_scenario(root.clone(), "flow".to_string()).unwrap(); // creates flow-copy

    let copy2_id = duplicate_scenario(root.clone(), "flow".to_string()).unwrap();
    assert_eq!(copy2_id, "flow-copy-2");
}

#[test]
fn test_load_scenario_tree() {
    let dir = setup_dir();
    let root = dir.path().to_path_buf();

    create_scenario(root.clone(), None, "Root Scenario".to_string()).unwrap();
    create_scenario(root.clone(), Some("auth".to_string()), "Login".to_string()).unwrap();

    let tree = load_scenario_tree(root.clone()).unwrap();
    // Should have a Group "auth" and a Scenario "root-scenario"
    assert_eq!(tree.len(), 2);
}

#[test]
fn test_load_scenario_tree_empty() {
    let dir = setup_dir();
    let root = dir.path().to_path_buf();

    let tree = load_scenario_tree(root.clone()).unwrap();
    assert!(tree.is_empty());
}
