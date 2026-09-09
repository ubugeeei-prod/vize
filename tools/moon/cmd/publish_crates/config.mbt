/// Parses a positive integer from a string and falls back on invalid input.
fn positive_int_or(value : String, fallback : Int) -> Int {
  try {
    let parsed = @string.parse_int(value)
    if parsed > 0 {
      parsed
    } else {
      fallback
    }
  } catch {
    _ => fallback
  }
}

/// Reads a retry-related integer from the environment with a validated default.
fn env_positive_int(name : String, fallback : Int) -> Int {
  match @env.get_env_var(name) {
    Some(value) if value != "" => positive_int_or(value, fallback)
    _ => fallback
  }
}

/// Extracts the workspace package version from the root `Cargo.toml`.
/// We only need the `[workspace.package]` version line, so a lightweight line
/// scanner is more predictable than shelling out to another TOML parser.
fn workspace_version_from_content(content : String) -> String? {
  let mut section = ""
  let lines : Array[String] = content.split("\n").map(view => view.to_owned()).collect()
  for line in lines {
    let trimmed = line.trim()
    if trimmed.has_prefix("[") && trimmed.has_suffix("]") {
      section = trimmed.to_owned()
      continue
    }
    if section == "[workspace.package]" &&
      trimmed.has_prefix("version = \"") &&
      trimmed.has_suffix("\"") {
      return Some(
        trimmed
          .to_owned()
          .replace(old="version = \"", new="")
          .replace_all(old="\"", new=""),
      )
    }
  }
  None
}
