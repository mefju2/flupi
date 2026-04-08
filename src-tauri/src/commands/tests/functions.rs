use super::*;
use tempfile::tempdir;
use crate::models::script_function::ScriptFunction;

#[test]
fn list_functions_empty_when_no_dir() {
    let dir = tempdir().unwrap();
    let result = list_functions(dir.path().to_path_buf()).unwrap();
    assert!(result.is_empty());
}

#[test]
fn save_and_list_function() {
    let dir = tempdir().unwrap();
    let f = ScriptFunction {
        name: "myFn".to_string(),
        body: "return 42;".to_string(),
        params: vec![],
    };
    save_function(dir.path().to_path_buf(), f.clone()).unwrap();
    let list = list_functions(dir.path().to_path_buf()).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].name, "myFn");
    assert_eq!(list[0].body, "return 42;");
}

#[test]
fn delete_function_removes_file() {
    let dir = tempdir().unwrap();
    let f = ScriptFunction {
        name: "toDelete".to_string(),
        body: "return 1;".to_string(),
        params: vec![],
    };
    save_function(dir.path().to_path_buf(), f).unwrap();
    delete_function(dir.path().to_path_buf(), "toDelete".to_string()).unwrap();
    let list = list_functions(dir.path().to_path_buf()).unwrap();
    assert!(list.is_empty());
}

#[test]
fn delete_nonexistent_function_is_ok() {
    let dir = tempdir().unwrap();
    let result = delete_function(dir.path().to_path_buf(), "ghost".to_string());
    assert!(result.is_ok());
}

#[test]
fn save_function_rejects_path_traversal() {
    let dir = tempdir().unwrap();
    let f = ScriptFunction {
        name: "../../etc/passwd".to_string(),
        body: "return 1;".to_string(),
        params: vec![],
    };
    let result = save_function(dir.path().to_path_buf(), f);
    assert!(result.is_err());
}

#[test]
fn save_function_rejects_empty_name() {
    let dir = tempdir().unwrap();
    let f = ScriptFunction {
        name: "".to_string(),
        body: "return 1;".to_string(),
        params: vec![],
    };
    let result = save_function(dir.path().to_path_buf(), f);
    assert!(result.is_err());
}

#[test]
fn delete_function_rejects_path_traversal() {
    let dir = tempdir().unwrap();
    let result = delete_function(dir.path().to_path_buf(), "../evil".to_string());
    assert!(result.is_err());
}


#[test]
fn rename_function_leaves_no_duplicate() {
    let dir = tempdir().unwrap();
    let f = ScriptFunction {
        name: "oldName".to_string(),
        body: "return 1;".to_string(),
        params: vec![],
    };
    save_function(dir.path().to_path_buf(), f).unwrap();
    rename_function(dir.path().to_path_buf(), "oldName".to_string(), "newName".to_string()).unwrap();
    let list = list_functions(dir.path().to_path_buf()).unwrap();
    assert_eq!(list.len(), 1, "expected exactly one function after rename, found {}", list.len());
    assert_eq!(list[0].name, "newName");
}
