/// Cargo still resolves dev-dependencies while preparing a package, so each
/// publish call strips dev-dependency sections from its local manifest and
/// restores them immediately after Cargo exits. The temporary manifest can make
/// Cargo refresh the workspace lockfile, so the lockfile is restored too.

/// Writes a text file and converts write failures into a boolean result.
fn write_string_to_file(path : String, content : String) -> Bool {
  try {
    @xfs.write_string_to_file(path, content)
    true
  } catch {
    _ => false
  }
}

fn crate_manifest_path(crate_name : String) -> String {
  @tool.join_path(@tool.join_path("crates", crate_name), "Cargo.toml")
}

fn is_manifest_section_header(trimmed : String) -> Bool {
  trimmed.has_prefix("[") && trimmed.has_suffix("]")
}

fn is_publish_stripped_section(trimmed : String) -> Bool {
  trimmed == "[dev-dependencies]" ||
  (trimmed.has_prefix("[target.") && trimmed.has_suffix(".dev-dependencies]"))
}

fn publish_manifest_without_dev_dependencies(content : String) -> String {
  let lines : Array[String] = content.split("\n").map(view => view.to_owned()).collect()
  let kept : Array[String] = []
  let mut skipping = false
  for line in lines {
    let trimmed = line.trim().to_owned()
    if is_manifest_section_header(trimmed) {
      skipping = is_publish_stripped_section(trimmed)
    }
    if !skipping {
      kept.push(line)
    }
  }
  let stripped = kept.join("\n")
  if content.has_suffix("\n") && !stripped.has_suffix("\n") {
    stripped + "\n"
  } else {
    stripped
  }
}

fn stage_publish_manifest(crate_name : String) -> (String, String, String)? {
  let manifest_path = crate_manifest_path(crate_name)
  let original = match @tool.read_file_to_string(manifest_path) {
    Some(content) => content
    None => return None
  }
  let original_lockfile = match @tool.read_file_to_string("Cargo.lock") {
    Some(content) => content
    None => return None
  }
  let publish_manifest = publish_manifest_without_dev_dependencies(original)
  if publish_manifest != original && !write_string_to_file(manifest_path, publish_manifest) {
    return None
  }
  Some((manifest_path, original, original_lockfile))
}

fn restore_publish_manifest(
  manifest_path : String,
  original_manifest : String,
  original_lockfile : String,
) -> Bool {
  write_string_to_file(manifest_path, original_manifest) &&
  write_string_to_file("Cargo.lock", original_lockfile)
}
