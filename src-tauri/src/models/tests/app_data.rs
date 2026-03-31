use super::*;

#[test]
fn test_recent_projects_add_and_sort() {
    let mut rp = RecentProjects { projects: vec![] };
    rp.add("Test", "/path/to/test");
    rp.add("Other", "/path/to/other");

    assert_eq!(rp.projects.len(), 2);
    assert_eq!(rp.projects[0].name, "Other");
}

#[test]
fn test_recent_projects_dedup_by_path() {
    let mut rp = RecentProjects { projects: vec![] };
    rp.add("Test", "/path/to/test");
    rp.add("Test Updated", "/path/to/test");

    assert_eq!(rp.projects.len(), 1);
    assert_eq!(rp.projects[0].name, "Test Updated");
}

#[test]
fn test_recent_projects_remove() {
    let mut rp = RecentProjects { projects: vec![] };
    rp.add("Test", "/path/to/test");
    rp.remove("/path/to/test");

    assert_eq!(rp.projects.len(), 0);
}

#[test]
fn test_add_preserves_active_environment() {
    let mut rp = RecentProjects { projects: vec![] };
    rp.add("Test", "/path/to/test");
    rp.update_active_environment("/path/to/test", Some("prod.env.json"));
    // Re-adding the same project (e.g. on next app open) should preserve active_environment
    rp.add("Test", "/path/to/test");
    assert_eq!(rp.projects[0].active_environment, Some("prod.env.json".to_string()));
}

#[test]
fn test_recent_project_active_environment_defaults_to_none() {
    let json = r#"{"name":"Test","path":"/path","lastOpenedAt":"2024-01-01T00:00:00Z"}"#;
    let project: RecentProject = serde_json::from_str(json).unwrap();
    assert_eq!(project.active_environment, None);
}

#[test]
fn test_update_active_environment_sets_value() {
    let mut rp = RecentProjects { projects: vec![] };
    rp.add("Test", "/path/to/test");
    rp.update_active_environment("/path/to/test", Some("prod.env.json"));
    assert_eq!(rp.projects[0].active_environment, Some("prod.env.json".to_string()));
}

#[test]
fn test_update_active_environment_clears_value() {
    let mut rp = RecentProjects { projects: vec![] };
    rp.add("Test", "/path/to/test");
    rp.update_active_environment("/path/to/test", Some("prod.env.json"));
    rp.update_active_environment("/path/to/test", None);
    assert_eq!(rp.projects[0].active_environment, None);
}

#[test]
fn test_update_active_environment_unknown_path_is_noop() {
    let mut rp = RecentProjects { projects: vec![] };
    rp.add("Test", "/path/to/test");
    rp.update_active_environment("/other/path", Some("dev.env.json"));
    assert_eq!(rp.projects[0].active_environment, None);
}
