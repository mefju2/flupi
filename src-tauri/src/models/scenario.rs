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

/// A step in a scenario. Three variants are supported:
/// - `Delay`: waits for `duration` milliseconds before continuing
/// - `Pause`: halts execution until the user clicks Resume (or aborts)
/// - `Request`: executes an HTTP request (the original step type)
///
/// Uses `untagged` deserialization — each struct has `deny_unknown_fields` so
/// serde tries them in order and picks the first that succeeds:
/// - JSON with `duration` → `Delay`
/// - JSON with `pause` (bool discriminator) → `Pause`
/// - JSON with `requestId` (and no `duration`/`pause`) → `Request` (backward compatible)
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum ScenarioStep {
    Delay(DelayStep),
    Pause(PauseStep),
    Request(RequestStep),
}

impl ScenarioStep {
    pub fn id(&self) -> &str {
        match self {
            ScenarioStep::Delay(s) => &s.id,
            ScenarioStep::Pause(s) => &s.id,
            ScenarioStep::Request(s) => &s.id,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            ScenarioStep::Delay(s) => &s.name,
            ScenarioStep::Pause(s) => &s.name,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct PauseStep {
    pub id: String,
    pub name: String,
    /// Discriminator field — always `true` in valid data. Its presence
    /// distinguishes this variant from Delay (has `duration`) and
    /// Request (has `requestId`) during untagged deserialization.
    pub pause: bool,
}

#[cfg(test)]
#[path = "tests/scenario.rs"]
mod tests;
