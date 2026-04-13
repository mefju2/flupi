use serde::{Deserialize, Serialize};
use indexmap::IndexMap;
use crate::models::extraction::Extraction;

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

/// A step in a scenario. Two variants are supported:
/// - `Delay`: waits for `duration` milliseconds before continuing
/// - `Request`: executes an HTTP request (the original step type)
///
/// Uses `untagged` deserialization so that:
/// - Old JSON with `requestId` but no `duration` → `Request` (backward compatible)
/// - New JSON with `duration` → `Delay`
///
/// `Delay` is listed first so serde tries it first; it requires `duration`
/// which old request-step JSON won't have, so it falls through to `Request`.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum ScenarioStep {
    Delay(DelayStep),
    Request(RequestStep),
}

impl ScenarioStep {
    pub fn id(&self) -> &str {
        match self {
            ScenarioStep::Delay(s) => &s.id,
            ScenarioStep::Request(s) => &s.id,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            ScenarioStep::Delay(s) => &s.name,
            ScenarioStep::Request(s) => &s.name,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct RequestStep {
    pub id: String,
    pub name: String,
    #[serde(rename = "requestId")]
    pub request_id: String,
    #[serde(default)]
    pub overrides: IndexMap<String, String>,
    #[serde(default)]
    pub extract: Vec<Extraction>,
    #[serde(rename = "expectedStatus", default)]
    pub expected_status: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct DelayStep {
    pub id: String,
    pub name: String,
    pub duration: u64, // milliseconds
}

#[cfg(test)]
#[path = "tests/scenario.rs"]
mod tests;
