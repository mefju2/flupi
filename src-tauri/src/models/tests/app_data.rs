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
