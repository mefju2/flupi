use std::collections::HashMap;

pub struct VariableContext {
    variables: HashMap<String, String>,
    secret_keys: Vec<String>,
}

impl VariableContext {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            secret_keys: Vec::new(),
        }
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.variables.insert(key.to_string(), value.to_string());
    }

    pub fn mark_secret(&mut self, key: &str) {
        if !self.secret_keys.contains(&key.to_string()) {
            self.secret_keys.push(key.to_string());
        }
    }

    pub fn is_secret(&self, key: &str) -> bool {
        self.secret_keys.contains(&key.to_string())
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.variables.get(key).map(|s| s.as_str())
    }

    pub fn all_keys(&self) -> Vec<String> {
        self.variables.keys().cloned().collect()
    }
}
