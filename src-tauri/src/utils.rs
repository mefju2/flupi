/// Converts a human-readable name into a URL/filesystem-safe slug.
/// Lowercases the name and replaces spaces with hyphens.
pub fn name_to_slug(name: &str) -> String {
    name.to_lowercase().replace(' ', "-")
}

#[cfg(test)]
#[path = "tests/utils.rs"]
mod tests;
