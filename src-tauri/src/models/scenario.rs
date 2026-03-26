use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Scenario {
    pub name: String,
    #[serde(default)]
    pub inputs: Vec<ScenarioInput>,
    pub steps: Vec<ScenarioStep>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScenarioInput {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub default: String,
    #[serde(default = "default_true")]
    pub required: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScenarioStep {
    pub id: String,
    pub name: String,
    #[serde(rename = "requestId")]
    pub request_id: String,
    #[serde(default)]
    pub overrides: HashMap<String, String>,
    #[serde(default)]
    pub extract: Vec<Extraction>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Extraction {
    pub variable: String,
    pub from: String,
    pub path: String,
}

#[cfg(test)]
#[path = "tests/scenario.rs"]
mod tests;
