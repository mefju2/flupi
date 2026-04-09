use super::*;

#[test]
fn test_parse_branches_local() {
    let output = "* main\n  feature/foo\n  fix/bar\n";
    let branches = parse_branch_list(output);
    assert_eq!(branches.len(), 3);
    assert!(branches.iter().any(|b| b.name == "main" && b.is_current && !b.is_remote));
    assert!(branches.iter().any(|b| b.name == "feature/foo" && !b.is_current && !b.is_remote));
    assert!(branches.iter().any(|b| b.name == "fix/bar" && !b.is_current && !b.is_remote));
}

#[test]
fn test_parse_branches_remote() {
    let output =
        "* main\n  remotes/origin/main\n  remotes/origin/dev\n  remotes/origin/HEAD -> origin/main\n";
    let branches = parse_branch_list(output);
    assert!(branches.iter().any(|b| b.name == "origin/main" && b.is_remote));
    assert!(branches.iter().any(|b| b.name == "origin/dev" && b.is_remote));
    // HEAD pointer should be filtered out
    assert!(!branches.iter().any(|b| b.name.contains("HEAD")));
}

#[test]
fn test_checkout_branch_rejects_suspicious_names() {
    use std::path::Path;
    assert!(checkout_branch(Path::new("/tmp"), "../../etc").is_err());
    assert!(checkout_branch(Path::new("/tmp"), "-bad").is_err());
}
