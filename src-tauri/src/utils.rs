/// Converts a human-readable name into a URL/filesystem-safe slug.
/// Lowercases the name and replaces spaces with hyphens.
pub fn name_to_slug(name: &str) -> String {
    name.to_lowercase().replace(' ', "-")
}

/// Creates a `Command` for `git` that never opens a visible console window on
/// Windows (the `CREATE_NO_WINDOW` flag). On other platforms it behaves exactly
/// like `Command::new("git")`.
pub fn git_command() -> std::process::Command {
    let cmd = std::process::Command::new("git");
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}

#[cfg(test)]
#[path = "tests/utils.rs"]
mod tests;
