use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ParamType {
    String,
    Number,
    Boolean,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionParam {
    pub name: String,
    pub param_type: ParamType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptFunction {
    pub name: String,
    pub body: String,
    #[serde(default)]
    pub params: Vec<FunctionParam>,
}
