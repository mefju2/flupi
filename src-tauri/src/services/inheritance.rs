use crate::models::request::{Request, AuthConfig};
use crate::models::collection::Collection;
use indexmap::IndexMap;

pub fn resolve_inheritance(request: &Request, collection: Option<&Collection>) -> Request {
    let mut effective = request.clone();

    // Filter disabled request-level headers before merging
    effective.headers.retain(|k, _| !effective.disabled_headers.contains(k));

    if let Some(col) = collection {
        // Auth: inherit if absent or explicit Inherit
        match &effective.auth {
            None | Some(AuthConfig::Inherit) => {
                effective.auth = col.auth.clone();
            }
            _ => {}
        }

        // Headers: collection first, then request (request wins on conflict).
        // Disabled collection headers are excluded from the merge.
        let col_headers: IndexMap<String, String> = col.headers
            .iter()
            .filter(|(k, _)| !effective.disabled_collection_headers.contains(*k))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let mut merged = col_headers;
        merged.extend(effective.headers.clone());
        effective.headers = merged;

        // BaseUrl: prepend if path is relative (not an absolute URL or a template variable
        // that will resolve to one, e.g. {{BaseUrl}}/path)
        if let Some(base_url) = &col.base_url {
            if !effective.path.starts_with("http://")
                && !effective.path.starts_with("https://")
                && !effective.path.starts_with("{{")
            {
                effective.path =
                    format!("{}{}", base_url.trim_end_matches('/'), effective.path);
            }
        }
    }

    effective
}

#[cfg(test)]
#[path = "tests/inheritance.rs"]
mod tests;
