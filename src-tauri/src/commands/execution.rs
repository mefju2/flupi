use std::sync::atomic::{AtomicBool, Ordering};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tauri::command;
use crate::error::FlupiError;
use crate::models::request::{AuthConfig, BodyConfig};
use crate::models::collection::Collection;
use crate::services::{http_client, inheritance, variable_resolver, file_io};
use crate::models::environment;

static EXECUTION_LOCK: AtomicBool = AtomicBool::new(false);

pub fn acquire_lock() -> Result<(), FlupiError> {
    if EXECUTION_LOCK.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
        return Err(FlupiError::Custom("Another execution is already in progress".to_string()));
    }
    Ok(())
}

pub fn release_lock() {
    EXECUTION_LOCK.store(false, Ordering::SeqCst);
}

fn resolve_request_path(project_path: &Path, request_id: &str) -> PathBuf {
    let parts: Vec<&str> = request_id.splitn(2, '/').collect();
    if parts.len() == 2 {
        let first = parts[0];
        let rest = parts[1];
        let collection_dir = project_path.join("collections").join(first);
        if collection_dir.exists() {
            let rest_path = rest.replace('/', std::path::MAIN_SEPARATOR_STR);
            return project_path
                .join("collections")
                .join(first)
                .join("requests")
                .join(format!("{}.json", rest_path));
        }
    }
    let rest_path = request_id.replace('/', std::path::MAIN_SEPARATOR_STR);
    project_path.join("requests").join(format!("{}.json", rest_path))
}

fn collection_folder_for(request_id: &str, project_path: &Path) -> Option<String> {
    let parts: Vec<&str> = request_id.splitn(2, '/').collect();
    if parts.len() == 2 {
        let first = parts[0];
        let collection_dir = project_path.join("collections").join(first);
        if collection_dir.exists() {
            return Some(first.to_string());
        }
    }
    None
}

pub async fn execute_single_request(
    project_path: &Path,
    request_id: &str,
    env_file_name: &str,
    timeout_ms: u64,
    extra_vars: &HashMap<String, String>,
    path_param_overrides: &HashMap<String, String>,
) -> Result<http_client::HttpResponse, FlupiError> {
    use base64::Engine as _;
    use base64::engine::general_purpose::STANDARD;

    // 1. Load request
    let request_path = resolve_request_path(project_path, request_id);
    let request = file_io::read_json::<crate::models::request::Request>(&request_path)?;

    // 2. Find parent collection
    let collection: Option<Collection> = collection_folder_for(request_id, project_path)
        .and_then(|folder| {
            let col_path = project_path.join("collections").join(&folder).join("collection.json");
            file_io::read_json::<Collection>(&col_path).ok()
        });

    // 3. Apply inheritance
    let mut effective = inheritance::resolve_inheritance(&request, collection.as_ref());

    // 3b. Apply path param overrides from scenario step
    for (k, v) in path_param_overrides {
        effective.path_params.insert(k.clone(), v.clone());
    }

    // 4. Load env variables
    let env_path = project_path.join("environments").join(format!("{}.json", env_file_name));
    let env_vars = if env_path.exists() {
        environment::resolve_env_variables(&env_path)?
    } else {
        HashMap::new()
    };

    // 5. Build variable context
    let ctx = variable_resolver::build_context(env_vars, &[], None, Some(extra_vars));

    // 6. Resolve variables in method and path
    let method = variable_resolver::resolve_string(&effective.method, &ctx);
    let path_resolved = variable_resolver::resolve_path_params(
        &effective.path,
        &effective.path_params,
        &ctx,
    );
    let url = variable_resolver::resolve_string(&path_resolved, &ctx);

    // 7. Resolve headers
    let mut headers: HashMap<String, String> = effective.headers
        .iter()
        .map(|(k, v)| (k.clone(), variable_resolver::resolve_string(v, &ctx)))
        .collect();

    // 8. Apply auth → headers
    match &effective.auth {
        Some(AuthConfig::Bearer { token }) => {
            let resolved = variable_resolver::resolve_string(token, &ctx);
            headers.insert("Authorization".to_string(), format!("Bearer {}", resolved));
        }
        Some(AuthConfig::Basic { username, password }) => {
            let u = variable_resolver::resolve_string(username, &ctx);
            let p = variable_resolver::resolve_string(password, &ctx);
            let encoded = STANDARD.encode(format!("{}:{}", u, p).as_bytes());
            headers.insert("Authorization".to_string(), format!("Basic {}", encoded));
        }
        Some(AuthConfig::ApiKey { header, value }) => {
            let h = variable_resolver::resolve_string(header, &ctx);
            let v = variable_resolver::resolve_string(value, &ctx);
            headers.insert(h, v);
        }
        Some(AuthConfig::Custom { headers: custom_headers }) => {
            for (k, v) in custom_headers {
                headers.insert(k.clone(), variable_resolver::resolve_string(v, &ctx));
            }
        }
        _ => {}
    }

    // 9. Resolve body
    let body = effective.body.as_ref().and_then(|b| match b {
        BodyConfig::Json { content } => {
            // content may be stored as a JSON string (editor output) or a JSON object.
            // Extract the raw template text without re-encoding it.
            let template = match content {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            let resolved_str = variable_resolver::resolve_string(&template, &ctx);
            serde_json::from_str::<serde_json::Value>(&resolved_str).ok().map(|mut json_val| {
                // Apply body.* dot-path overrides from extra_vars so scenario
                // overrides like `body.scenarioParams.error` patch the JSON object
                // directly, regardless of whether the template uses {{...}} tokens.
                // Run resolve_string on each value first so that any residual
                // function tokens (e.g. {{$randomInt(12)}} that came through an
                // input variable) are expanded using the current ctx before the
                // value is written into the JSON body.
                for (k, v) in extra_vars {
                    if let Some(dot_path) = k.strip_prefix("body.") {
                        let resolved_v = variable_resolver::resolve_string(v, &ctx);
                        set_json_path(&mut json_val, dot_path, &resolved_v);
                    }
                }
                http_client::RequestBody::Json { content: json_val }
            })
        }
        BodyConfig::Form { content, disabled_fields } => {
            let resolved: HashMap<String, String> = content
                .iter()
                .filter(|(k, _)| !disabled_fields.contains(*k))
                .map(|(k, v)| (k.clone(), variable_resolver::resolve_string(v, &ctx)))
                .collect();
            Some(http_client::RequestBody::Form { content: resolved })
        }
        BodyConfig::Raw { content } => {
            Some(http_client::RequestBody::Raw {
                content: variable_resolver::resolve_string(content, &ctx),
            })
        }
        BodyConfig::None => None,
    });

    // 10. Execute
    let executable = http_client::ExecutableRequest {
        method,
        url,
        headers,
        body,
        timeout_ms,
    };

    eprintln!("[flupi] sending request:\n  url:     {}\n  method:  {}\n  headers: {:?}\n  body:    {:?}",
        executable.url, executable.method, executable.headers, executable.body);

    http_client::execute_request(&executable).await
}

fn set_json_path(value: &mut serde_json::Value, path: &str, raw: &str) {
    let new_val = serde_json::from_str::<serde_json::Value>(raw)
        .unwrap_or_else(|_| serde_json::Value::String(raw.to_string()));
    let (head, tail) = match path.find('.') {
        Some(i) => (&path[..i], Some(&path[i + 1..])),
        None => (path, None),
    };
    if let serde_json::Value::Object(map) = value {
        match tail {
            None => { map.insert(head.to_string(), new_val); }
            Some(rest) => {
                let entry = map.entry(head.to_string())
                    .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
                set_json_path(entry, rest, raw);
            }
        }
    }
}

fn apply_extractions_to_env(
    project_path: &Path,
    env_file_name: &str,
    extractions: &[crate::models::extraction::Extraction],
    response: &http_client::HttpResponse,
) -> Result<(), FlupiError> {
    use crate::models::environment::Environment;
    use crate::commands::execution_runner::apply_extraction;

    let env_path = project_path.join("environments").join(format!("{}.json", env_file_name));
    if !env_path.exists() {
        return Ok(());
    }

    let mut env: Environment = file_io::read_json(&env_path)?;
    let secrets_path = project_path.join("environments").join(format!("{}.secrets.json", env_file_name));
    let mut secrets: Option<HashMap<String, String>> = if secrets_path.exists() {
        Some(file_io::read_json(&secrets_path)?)
    } else {
        None
    };

    let mut env_dirty = false;
    let mut secrets_dirty = false;

    for extraction in extractions.iter().filter(|e| !e.variable.is_empty() && !e.path.is_empty()) {
        let value = match apply_extraction(extraction, &response.body, &response.headers) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if env.variables.contains_key(&extraction.variable) {
            env.variables.insert(extraction.variable.clone(), value);
            env_dirty = true;
        } else if env.secrets.contains(&extraction.variable) {
            let s = secrets.get_or_insert_with(HashMap::new);
            s.insert(extraction.variable.clone(), value);
            secrets_dirty = true;
        }
    }

    if env_dirty {
        file_io::write_json(&env_path, &env)?;
    }
    if secrets_dirty {
        if let Some(s) = &secrets {
            file_io::write_json(&secrets_path, s)?;
        }
    }
    Ok(())
}

#[command]
pub async fn send_request(
    project_path: PathBuf,
    request_id: String,
    env_file_name: String,
    timeout_ms: u64,
    injected_vars: Option<HashMap<String, String>>,
) -> Result<http_client::HttpResponse, FlupiError> {
    // Read extractions once before the request is sent to avoid a race where
    // the user saves new extractions while a slow request is in flight.
    let request_path = resolve_request_path(&project_path, &request_id);
    let extractions = file_io::read_json::<crate::models::request::Request>(&request_path)
        .map(|r| r.extractions)
        .unwrap_or_default();

    let extra_vars: HashMap<String, String> = injected_vars.unwrap_or_default();

    acquire_lock()?;
    let result = execute_single_request(&project_path, &request_id, &env_file_name, timeout_ms, &extra_vars, &HashMap::new()).await;

    if let Ok(ref response) = result {
        if !extractions.is_empty() {
            if let Err(e) = apply_extractions_to_env(&project_path, &env_file_name, &extractions, response) {
                eprintln!("[flupi] extraction write failed: {e}");
            }
        }
    }

    release_lock();
    result
}
