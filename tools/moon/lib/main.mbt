/// Returns command-line arguments after the generated executable path.
pub fn cli_args() -> Array[String] {
  let raw = @env.args()
  let args = []
  for i in 1..<raw.length() {
    args.push(raw[i])
  }
  args
}

/// Resolves a path to an absolute normalized string.
pub fn resolve_path(value : String) -> String {
  let path : @path.Path = value
  path.resolve().to_string()
}

/// Joins two path fragments using MoonBit's cross-platform path type.
pub fn join_path(base : String, child : String) -> String {
  let path : @path.Path = base
  path.join(child).to_string()
}

/// Returns whether a filesystem path exists without throwing.
pub fn path_exists(path : String) -> Bool {
  @xfs.path_exists(path)
}

/// Returns whether a path is a directory, treating IO errors as false.
pub fn is_dir(path : String) -> Bool {
  try {
    @xfs.is_dir(path)
  } catch {
    _ => false
  }
}

/// Reads a directory, returning an empty list when enumeration fails.
pub fn read_dir(path : String) -> Array[String] {
  try {
    @xfs.read_dir(path)
  } catch {
    _ => []
  }
}

/// Reads a UTF-8 file and collapses IO failures into None.
pub fn read_file_to_string(path : String) -> String? {
  try {
    Some(@xfs.read_file_to_string(path))
  } catch {
    _ => None
  }
}

/// Resolves a binary name from the environment, falling back to a default.
pub fn tool_bin(env_name : String, fallback : String) -> String {
  match @env.get_env_var(env_name) {
    Some(value) if value != "" => value
    _ => fallback
  }
}
