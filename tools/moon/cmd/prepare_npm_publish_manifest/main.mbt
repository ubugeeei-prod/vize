/// Rewrites workspace-local dependency protocols into publishable npm versions.
/// Release jobs publish prebuilt package directories without a full workspace
/// install. That means protocol-based specs such as `workspace:*` and
/// `catalog:*` must be resolved inside `package.json` before `vp pm publish`
/// runs, otherwise pnpm aborts before it ever talks to the registry.
/// This helper makes a manifest standalone by:
/// 1. Finding the repository root that owns the target package.
/// 2. Collecting workspace package versions from `npm/**/package.json`.
/// 3. Reading catalog pins from `pnpm-workspace.yaml`.
/// 4. Rewriting dependency-like sections in place.

let dependency_sections : Array[String] = [
  "dependencies",
  "peerDependencies",
  "optionalDependencies",
  "devDependencies",
]

fn read_package_json(path : String) -> Json? {
  try {
    Some(@json.parse(@xfs.read_file_to_string(path)))
  } catch {
    _ => None
  }
}

fn package_name(json : Json) -> String? {
  guard json is Object(obj) else { return None }
  guard obj.get("name") is Some(String(name)) else { return None }
  guard name != "" else { return None }
  Some(name)
}

fn package_version(json : Json) -> String? {
  guard json is Object(obj) else { return None }
  guard obj.get("version") is Some(String(version)) else { return None }
  guard version != "" else { return None }
  Some(version)
}

fn yaml_scalar(value : String) -> String {
  value.trim().to_owned().replace_all(old="\"", new="")
}

fn find_repo_root(start : String) -> String? {
  let current : @path.Path = start
  let current_path = current.resolve().to_string()
  if @tool.path_exists(@tool.join_path(current_path, "pnpm-workspace.yaml")) &&
    @tool.is_dir(@tool.join_path(current_path, "npm")) {
    return Some(current_path)
  }
  let parent = current.dirname().to_string()
  if parent == current_path {
    None
  } else {
    find_repo_root(parent)
  }
}

fn collect_workspace_packages(directory : String, versions : Map[String, String]) -> Unit {
  for entry_name in @tool.read_dir(directory) {
    if entry_name == "node_modules" || entry_name == ".git" || entry_name == "_build" {
      continue
    }
    let entry_path = @tool.join_path(directory, entry_name)
    if @tool.is_dir(entry_path) {
      collect_workspace_packages(entry_path, versions)
      continue
    }
    if entry_name != "package.json" {
      continue
    }
    match read_package_json(entry_path) {
      Some(json) => {
        match (package_name(json), package_version(json)) {
          (Some(name), Some(version)) => {
            versions[name] = version
          }
          _ => ()
        }
      }
      None => ()
    }
  }
}

/// Flattens catalog pins into a lookup keyed by `<catalog>\0<package>`.
fn parse_catalog_versions(content : String) -> Map[String, String] {
  let versions : Map[String, String] = {}
  let mut in_catalogs = false
  let mut current_catalog = ""
  let lines : Array[String] = content.split("\n").map(view => view.to_owned()).collect()
  for line in lines {
    let trimmed = line.trim()
    if trimmed == "" || trimmed.has_prefix("#") {
      continue
    }
    if !in_catalogs {
      if trimmed == "catalogs:" {
        in_catalogs = true
      }
      continue
    }
    if !line.has_prefix("  ") {
      break
    }
    if line.has_prefix("  ") && !line.has_prefix("    ") && trimmed.has_suffix(":") {
      current_catalog = trimmed.to_owned().replace(old=":", new="")
      continue
    }
    if current_catalog == "" || !line.has_prefix("    ") {
      continue
    }
    let parts : Array[String] = trimmed.split(": ").map(view => view.to_owned()).collect()
    guard parts.length() >= 2 else { continue }
    let key = yaml_scalar(parts[0])
    let value = yaml_scalar(parts[1])
    versions[current_catalog + "\u0000" + key] = value
  }
  versions
}

fn normalize_workspace_spec(spec : String, version : String) -> String {
  let suffix = spec.replace(old="workspace:", new="")
  if suffix == "" || suffix == "*" {
    version
  } else if suffix == "^" {
    "^" + version
  } else if suffix == "~" {
    "~" + version
  } else {
    version
  }
}

fn normalize_dependency_sections(
  package_json : Json,
  workspace_versions : Map[String, String],
  catalog_versions : Map[String, String],
  native_binary_version : String?,
) -> Array[String] {
  let unresolved = []
  guard package_json is Object(package_obj) else {
    unresolved.push("package.json root is not an object")
    return unresolved
  }
  for section in dependency_sections {
    if package_obj.get(section) is Some(Object(dependencies)) {
      for dependency_name, dependency_value in dependencies {
        guard dependency_value is String(version_spec) else { continue }
        if version_spec.has_prefix("workspace:") {
          match workspace_versions.get(dependency_name) {
            Some(version) => {
              dependencies[dependency_name] = Json::string(
                normalize_workspace_spec(version_spec, version),
              )
            }
            None =>
              unresolved.push(
                "Missing workspace version for \{dependency_name} referenced from \{section}",
              )
          }
          continue
        }
        if version_spec.has_prefix("catalog:") {
          let catalog_name = version_spec.replace(old="catalog:", new="")
          if catalog_name == "native-binaries" && dependency_name.has_prefix("@vizejs/native-") {
            match native_binary_version {
              Some(version) => {
                dependencies[dependency_name] = Json::string(version)
              }
              None =>
                unresolved.push(
                  "Missing native package version for \{dependency_name} from \{section}",
                )
            }
            continue
          }
          let catalog_key = catalog_name + "\u0000" + dependency_name
          match catalog_versions.get(catalog_key) {
            Some(version) => {
              dependencies[dependency_name] = Json::string(version)
            }
            None =>
              unresolved.push(
                "Missing catalog version for \{dependency_name} from \{catalog_name} in \{section}",
              )
          }
        }
      }
    }
  }
  unresolved
}

/// Main entry point for standalone publish manifest preparation.
/// The script accepts either a package directory or a direct `package.json`
/// path. `--print` is useful in CI when we need to inspect the rewritten
/// manifest that will actually be published.
async fn run_app() -> Int {
  let args = @tool.cli_args()
  if args.is_empty() {
    @stdio.stderr.write(
      "Usage: moon run --target native tools/moon/cmd/prepare_npm_publish_manifest -- <dir-or-package-json> [--print]\n",
    )
    return 1
  }
  let mut should_print = false
  let positional = []
  for arg in args {
    if arg == "--print" {
      should_print = true
    } else {
      positional.push(arg)
    }
  }
  if positional.is_empty() {
    @stdio.stderr.write("Missing package directory or package.json path\n")
    return 1
  }

  let raw_target = @tool.resolve_path(positional[0])
  let package_json_path = if @tool.is_dir(raw_target) {
    @tool.join_path(raw_target, "package.json")
  } else {
    raw_target
  }
  let package_directory : @path.Path = package_json_path
  let package_json = match read_package_json(package_json_path) {
    Some(json) => json
    None => {
      @stdio.stderr.write("Failed to parse \{package_json_path}\n")
      return 1
    }
  }
  let repo_root = match find_repo_root(package_directory.dirname().to_string()) {
    Some(path) => path
    None => {
      let unresolved = []
      guard package_json is Object(package_obj) else {
        @stdio.stderr.write("Failed to read object from \{package_json_path}\n")
        return 1
      }
      for section in dependency_sections {
        if package_obj.get(section) is Some(Object(dependencies)) {
          for dependency_name, dependency_value in dependencies {
            guard dependency_value is String(version_spec) else { continue }
            if version_spec.has_prefix("workspace:") || version_spec.has_prefix("catalog:") {
              unresolved.push(
                "Cannot normalize \{dependency_name} from \{section} because the repository root could not be located",
              )
            }
          }
        }
      }
      if unresolved.is_empty() {
        println("No workspace or catalog dependencies needed rewriting for \{package_json_path}")
        return 0
      }
      for item in unresolved {
        @stdio.stderr.write(item + "\n")
      }
      return 1
    }
  }

  let workspace_versions : Map[String, String] = {}
  collect_workspace_packages(@tool.join_path(repo_root, "npm"), workspace_versions)
  let workspace_yaml_path = @tool.join_path(repo_root, "pnpm-workspace.yaml")
  let workspace_yaml = match @tool.read_file_to_string(workspace_yaml_path) {
    Some(content) => content
    None => {
      @stdio.stderr.write("Failed to read \{workspace_yaml_path}\n")
      return 1
    }
  }
  let catalog_versions = parse_catalog_versions(workspace_yaml)
  let native_binary_version = match workspace_versions.get("@vizejs/native") {
    Some(version) => Some(version)
    None => package_version(package_json)
  }
  let unresolved = normalize_dependency_sections(
    package_json,
    workspace_versions,
    catalog_versions,
    native_binary_version,
  )
  if !unresolved.is_empty() {
    for item in unresolved {
      @stdio.stderr.write(item + "\n")
    }
    return 1
  }
  let updated = package_json.stringify(indent=2) + "\n"
  @xfs.write_string_to_file(package_json_path, updated)
  println("Prepared npm publish manifest at \{package_json_path}")
  if should_print {
    @stdio.stdout.write(updated)
  }
  0
}

async fn main {
  @sys.exit(run_app())
}
