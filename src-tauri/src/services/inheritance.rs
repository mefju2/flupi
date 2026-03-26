use crate::models::request::{Request, AuthConfig};
use crate::models::collection::Collection;

pub fn resolve_inheritance(request: &Request, collection: Option<&Collection>) -> Request {
    let mut effective = request.clone();

    if let Some(col) = collection {
        // Auth: inherit if absent or explicit Inherit
        match &effective.auth {
            None | Some(AuthConfig::Inherit) => {
                effective.auth = col.auth.clone();
            }
            _ => {}
        }

        // Headers: merge, request wins on conflict
        let mut merged = col.headers.clone();
        merged.extend(effective.headers.clone());
        effective.headers = merged;

        // BaseUrl: prepend if path is relative
        if let Some(base_url) = &col.base_url {
            if !effective.path.starts_with("http://")
                && !effective.path.starts_with("https://")
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
