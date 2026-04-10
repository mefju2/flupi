use super::*;

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
    // Renamed file has x='R' (staged), so it appears in staged not modified
    assert_eq!(status.staged, vec!["new_name.rs"]);
    assert!(status.modified.is_empty());
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
    // src/a.rs: ".M" → unstaged modified only
    assert_eq!(status.modified, vec!["src/a.rs"]);
    // src/b_new.rs: "R." → staged rename
    assert_eq!(status.staged, vec!["src/b_new.rs"]);
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

#[test]
fn test_staged_modified_detected_as_staged() {
    // "MM" = staged modified + working tree modified
    let output = "# branch.head main\n1 MM N... 100644 100644 100644 abc def src/foo.rs\n";
    let status = parse_porcelain_v2(output).unwrap();
    assert!(
        status.staged.contains(&"src/foo.rs".to_string()),
        "staged modified must appear in staged"
    );
    assert!(
        status.modified.contains(&"src/foo.rs".to_string()),
        "working-tree modified must also appear in modified"
    );
}

#[test]
fn test_staged_deleted_detected_as_staged() {
    // "D." = staged deletion, clean working tree
    let output = "# branch.head main\n1 D. N... 100644 000000 000000 abc 0000000 src/gone.rs\n";
    let status = parse_porcelain_v2(output).unwrap();
    assert!(status.staged.contains(&"src/gone.rs".to_string()));
}

#[test]
fn test_unstaged_only_not_in_staged() {
    // ".M" = unstaged modified only
    let output = "# branch.head main\n1 .M N... 100644 100644 100644 abc abc src/bar.rs\n";
    let status = parse_porcelain_v2(output).unwrap();
    assert!(
        !status.staged.contains(&"src/bar.rs".to_string()),
        "unstaged-only must not appear in staged"
    );
    assert!(status.modified.contains(&"src/bar.rs".to_string()));
}
