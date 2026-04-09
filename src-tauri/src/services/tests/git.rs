use super::*;

fn make_status(
    branch: &str,
    upstream: Option<&str>,
    ahead: u32,
    behind: u32,
    modified: Vec<&str>,
    untracked: Vec<&str>,
) -> GitStatus {
    GitStatus {
        branch: branch.to_string(),
        upstream: upstream.map(|s| s.to_string()),
        ahead,
        behind,
        modified: modified.into_iter().map(|s| s.to_string()).collect(),
        deleted: vec![],
        untracked: untracked.into_iter().map(|s| s.to_string()).collect(),
        is_git_repo: true,
    }
}

#[test]
fn test_parse_clean_repo() {
    let output = "\
# branch.oid abc123\n\
# branch.head main\n\
# branch.upstream origin/main\n\
# branch.ab +0 -0\n";
    let status = parse_porcelain_v2(output).unwrap();
    assert_eq!(status.branch, "main");
    assert_eq!(status.upstream, Some("origin/main".to_string()));
    assert_eq!(status.ahead, 0);
    assert_eq!(status.behind, 0);
    assert!(status.modified.is_empty());
    assert!(status.untracked.is_empty());
    assert!(status.is_git_repo);
}

#[test]
fn test_parse_ahead_behind() {
    let output = "\
# branch.oid abc123\n\
# branch.head feat/thing\n\
# branch.upstream origin/main\n\
# branch.ab +3 -2\n";
    let status = parse_porcelain_v2(output).unwrap();
    assert_eq!(status.ahead, 3);
    assert_eq!(status.behind, 2);
}

#[test]
fn test_parse_no_upstream() {
    let output = "\
# branch.oid abc123\n\
# branch.head local-only\n";
    let status = parse_porcelain_v2(output).unwrap();
    assert_eq!(status.branch, "local-only");
    assert_eq!(status.upstream, None);
    assert_eq!(status.ahead, 0);
    assert_eq!(status.behind, 0);
}

#[test]
fn test_parse_ordinary_modified_file() {
    let output = "\
# branch.head main\n\
1 .M N... 100644 100644 100644 aabbcc aabbcc src/lib.rs\n";
    let status = parse_porcelain_v2(output).unwrap();
    assert_eq!(status.modified, vec!["src/lib.rs"]);
    assert!(status.untracked.is_empty());
}

#[test]
fn test_parse_renamed_file() {
    let output = "\
# branch.head main\n\
2 R. N... 100644 100644 100644 aabbcc ddeeff R100 new_name.rs\told_name.rs\n";
    let status = parse_porcelain_v2(output).unwrap();
    assert_eq!(status.modified, vec!["new_name.rs"]);
}

#[test]
fn test_parse_untracked_file() {
    let output = "\
# branch.head main\n\
? some/new/file.ts\n";
    let status = parse_porcelain_v2(output).unwrap();
    assert!(status.modified.is_empty());
    assert_eq!(status.untracked, vec!["some/new/file.ts"]);
}

#[test]
fn test_parse_mixed_changes() {
    let output = "\
# branch.head main\n\
# branch.upstream origin/main\n\
# branch.ab +1 -0\n\
1 .M N... 100644 100644 100644 aabbcc aabbcc src/a.rs\n\
2 R. N... 100644 100644 100644 aabbcc ddeeff R90 src/b_new.rs\tsrc/b_old.rs\n\
? untracked.txt\n";
    let status = parse_porcelain_v2(output).unwrap();
    assert_eq!(status.ahead, 1);
    assert_eq!(status.modified, vec!["src/a.rs", "src/b_new.rs"]);
    assert_eq!(status.untracked, vec!["untracked.txt"]);
}

#[test]
fn test_path_traversal_rejected() {
    use std::path::Path;
    let result = get_file_diff(Path::new("/tmp"), "../../etc/passwd");
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("Invalid file path"),
        "expected 'Invalid file path', got: {msg}"
    );
}

#[test]
fn test_absolute_path_rejected() {
    use std::path::Path;
    let result = get_file_diff(Path::new("/tmp"), "/etc/passwd");
    assert!(result.is_err());
}
