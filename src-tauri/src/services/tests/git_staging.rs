use super::*;

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn make_temp_repo() -> PathBuf {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    let dir = std::env::temp_dir()
        .join(format!("flupi_test_{}_{}", std::process::id(), ts));
    fs::create_dir_all(&dir).unwrap();
    Command::new("git").args(["init"]).current_dir(&dir).output().unwrap();
    Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(&dir)
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(&dir)
        .output()
        .unwrap();
    fs::write(dir.join("README.md"), "init").unwrap();
    Command::new("git").args(["add", "."]).current_dir(&dir).output().unwrap();
    Command::new("git")
        .args(["commit", "-m", "init"])
        .current_dir(&dir)
        .output()
        .unwrap();
    dir
}

#[test]
fn test_stage_and_unstage_file() {
    let dir = make_temp_repo();
    fs::write(dir.join("foo.txt"), "hello").unwrap();

    stage_file(&dir, "foo.txt").unwrap();
    let status = super::super::git::get_status(&dir).unwrap();
    assert!(
        status.staged.contains(&"foo.txt".to_string()),
        "file should be staged"
    );

    unstage_file(&dir, "foo.txt").unwrap();
    let status = super::super::git::get_status(&dir).unwrap();
    assert!(
        !status.staged.contains(&"foo.txt".to_string()),
        "file should be unstaged"
    );

    fs::remove_dir_all(&dir).ok();
}

#[test]
fn test_stage_all_and_unstage_all() {
    let dir = make_temp_repo();
    fs::write(dir.join("a.txt"), "a").unwrap();
    fs::write(dir.join("b.txt"), "b").unwrap();

    stage_all(&dir).unwrap();
    let status = super::super::git::get_status(&dir).unwrap();
    assert!(status.staged.contains(&"a.txt".to_string()));
    assert!(status.staged.contains(&"b.txt".to_string()));

    unstage_all(&dir).unwrap();
    let status = super::super::git::get_status(&dir).unwrap();
    assert!(!status.staged.contains(&"a.txt".to_string()));
    assert!(!status.staged.contains(&"b.txt".to_string()));

    fs::remove_dir_all(&dir).ok();
}

#[test]
fn test_commit() {
    let dir = make_temp_repo();
    fs::write(dir.join("c.txt"), "c").unwrap();
    stage_all(&dir).unwrap();
    commit(&dir, "test commit message").unwrap();

    let log = Command::new("git")
        .args(["log", "--oneline", "-1"])
        .current_dir(&dir)
        .output()
        .unwrap();
    let msg = String::from_utf8_lossy(&log.stdout);
    assert!(msg.contains("test commit message"));

    fs::remove_dir_all(&dir).ok();
}

#[test]
fn test_commit_empty_message_fails() {
    let dir = make_temp_repo();
    let result = commit(&dir, "");
    assert!(result.is_err());
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn test_stage_file_rejects_path_traversal() {
    let dir = make_temp_repo();
    assert!(stage_file(&dir, "../etc/passwd").is_err());
    assert!(stage_file(&dir, "/etc/passwd").is_err());
    fs::remove_dir_all(&dir).ok();
}
