use regex::Regex;
use std::collections::HashMap;
use indexmap::IndexMap;
use once_cell::sync::Lazy;
pub use crate::models::variable::VariableContext;

static VAR_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\{\{([^{}]+)\}\}").unwrap()
});

pub fn resolve_string(template: &str, ctx: &VariableContext) -> String {
    VAR_REGEX.replace_all(template, |caps: &regex::Captures| {
        let key = &caps[1];
        ctx.get(key).unwrap_or(&caps[0]).to_string()
    })
    .to_string()
}

pub fn find_unresolved(template: &str, ctx: &VariableContext) -> Vec<String> {
    VAR_REGEX.captures_iter(template)
        .filter_map(|cap| {
            let key = cap[1].to_string();
            if ctx.get(&key).is_none() {
                Some(key)
            } else {
                None
            }
        })
        .collect()
}

pub fn resolve_path_params(
    path: &str,
    path_params: &IndexMap<String, String>,
    ctx: &VariableContext,
) -> String {
    let mut result = path.to_string();
    for (param, template) in path_params {
        let value = resolve_string(template, ctx);
        result = result.replace(&format!("{{{}}}", param), &value);
    }
    result
}

pub fn build_context(
    env_vars: HashMap<String, String>,
    secret_keys: &[String],
    scenario_inputs: Option<&HashMap<String, String>>,
    extracted: Option<&HashMap<String, String>>,
) -> VariableContext {
    let mut ctx = VariableContext::new();
    for (k, v) in &env_vars {
        ctx.set(k, v);
    }
    for key in secret_keys {
        ctx.mark_secret(key);
    }
    if let Some(inputs) = scenario_inputs {
        for (k, v) in inputs {
            ctx.set(k, v);
        }
    }
    if let Some(ext) = extracted {
        for (k, v) in ext {
            ctx.set(k, v);
        }
    }
    ctx
}

#[cfg(test)]
#[path = "tests/variable_resolver.rs"]
mod tests;
