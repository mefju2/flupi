use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptFunction {
    pub name: String,
    pub body: String,
}
