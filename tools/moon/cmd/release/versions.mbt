/// Version parsing and synchronized manifest updates for release candidates.

///|
/// Parsed semantic version components used by the release bump logic.
/// The struct stores only the fields the repository needs today: major / minor /
/// patch plus an optional prerelease label and numeric suffix.
struct Version {
  major : Int
  minor : Int
  patch : Int
  prerelease : String?
  prerelease_num : Int?
}

///|
async fn is_git_ignored(path : String) -> Bool {
  @process.run("git", ["check-ignore", "-q", path]) == 0
}

///|
/// Writes a text file and converts write failures into a boolean result.
fn write_string_to_file(path : String, content : String) -> Bool {
  try {
    @xfs.write_string_to_file(path, content)
    true
  } catch {
    _ => false
  }
}

///|
/// Parses an integer but keeps the caller in control of error handling.
fn parse_int(value : String) -> Int? {
  Some(@string.parse_int(value)) catch {
    _ => None
  }
}

///|
/// Parses the repository's release version format into structured fields.
/// Supported inputs are:
/// - `major.minor.patch`
/// - `major.minor.patch-label`
/// - `major.minor.patch-label.number`
/// The parser is intentionally strict so release automation never guesses when
/// the current version format has drifted unexpectedly.
fn parse_version(value : String) -> Version? {
  let parts : Array[String] = value
    .split("-")
    .map(view => view.to_owned())
    .collect()
  guard parts.length() >= 1 && parts.length() <= 2 else { return None }

  let base_parts : Array[String] = parts[0]
    .split(".")
    .map(view => view.to_owned())
    .collect()
  guard base_parts.length() == 3 else { return None }
  let major = match parse_int(base_parts[0]) {
    Some(value) => value
    None => return None
  }
  let minor = match parse_int(base_parts[1]) {
    Some(value) => value
    None => return None
  }
  let patch = match parse_int(base_parts[2]) {
    Some(value) => value
    None => return None
  }

  let mut prerelease : String? = None
  let mut prerelease_num : Int? = None
  if parts.length() == 2 {
    let prerelease_parts : Array[String] = parts[1]
      .split(".")
      .map(view => view.to_owned())
      .collect()
    guard prerelease_parts.length() >= 1 && prerelease_parts.length() <= 2 else {
      return None
    }
    guard prerelease_parts[0] != "" else { return None }
    prerelease = Some(prerelease_parts[0])
    if prerelease_parts.length() == 2 {
      prerelease_num = match parse_int(prerelease_parts[1]) {
        Some(value) => Some(value)
        None => return None
      }
    }
  }

  Some(Version::{ major, minor, patch, prerelease, prerelease_num })
}

///|
/// Formats a structured version back into the repository's release string form.
fn format_version(
  major : Int,
  minor : Int,
  patch : Int,
  prerelease : String?,
  prerelease_num : Int?,
) -> String {
  let base = "\{major}.\{minor}.\{patch}"
  match prerelease {
    None => base
    Some(label) =>
      match prerelease_num {
        Some(number) => "\{base}-\{label}.\{number}"
        None => "\{base}-\{label}"
      }
  }
}

///|
/// Computes the next release version for a given bump strategy.
/// Stable bumps (`patch`, `minor`, `major`) reset the numeric suffix, while
/// prerelease bumps preserve the existing base version and increment the suffix
/// only when the current version already uses the same prerelease channel.
fn bump_version(current : String, bump : String) -> String? {
  let version = match parse_version(current) {
    Some(version) => version
    None => return None
  }
  match bump {
    "patch" =>
      Some(
        format_version(
          version.major,
          version.minor,
          version.patch + 1,
          None,
          None,
        ),
      )
    "minor" =>
      Some(format_version(version.major, version.minor + 1, 0, None, None))
    "major" => Some(format_version(version.major + 1, 0, 0, None, None))
    "alpha" =>
      if version.prerelease == Some("alpha") {
        Some(
          format_version(
            version.major,
            version.minor,
            version.patch,
            Some("alpha"),
            Some(version.prerelease_num.unwrap_or(1) + 1),
          ),
        )
      } else {
        Some(
          format_version(
            version.major,
            version.minor,
            version.patch,
            Some("alpha"),
            None,
          ),
        )
      }
    "beta" =>
      Some(
        format_version(
          version.major,
          version.minor,
          version.patch,
          Some("beta"),
          None,
        ),
      )
    "rc" =>
      Some(
        format_version(
          version.major,
          version.minor,
          version.patch,
          Some("rc"),
          None,
        ),
      )
    "release" =>
      Some(
        format_version(version.major, version.minor, version.patch, None, None),
      )
    _ => None
  }
}

///|
/// Extracts the current workspace version from the root `Cargo.toml`.
fn workspace_version_from_content(content : String) -> String? {
  let mut section = ""
  let lines : Array[String] = content
    .split("\n")
    .map(view => view.to_owned())
    .collect()
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

///|
/// Rewrites all workspace-level Rust version pins that must stay aligned.
/// This updates both `[workspace.package].version` and the exact-version
/// `vize_*` dependencies inside `[workspace.dependencies]`.
fn update_workspace_manifest_versions(
  content : String,
  current_version : String,
  new_version : String,
) -> String {
  let mut section = ""
  let updated : Array[String] = []
  let lines : Array[String] = content
    .split("\n")
    .map(view => view.to_owned())
    .collect()
  for line in lines {
    let trimmed = line.trim()
    if trimmed.has_prefix("[") && trimmed.has_suffix("]") {
      section = trimmed.to_owned()
      updated.push(line)
      continue
    }
    if section == "[workspace.package]" &&
      trimmed == "version = \"\{current_version}\"" {
      updated.push(line.replace(old=current_version, new=new_version))
      continue
    }
    if section == "[workspace.dependencies]" &&
      trimmed.has_prefix("vize_") &&
      line.contains("version = \"=\{current_version}\"") {
      updated.push(
        line.replace_all(
          old="version = \"=\{current_version}\"",
          new="version = \"=\{new_version}\"",
        ),
      )
      continue
    }
    updated.push(line)
  }
  updated.join("\n")
}
