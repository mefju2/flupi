use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PreRequestAction {
    SetVariable {
        variable: String,
        function_name: String,
        #[serde(default)]
        args: Vec<String>,
    },
}

#[cfg(test)]
#[path = "tests/pre_request_action.rs"]
mod tests;
