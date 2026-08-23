/// Publishes one npm package with rerun-safe registry visibility checks.

/// Minimal package metadata for registry checks.
struct NpmPackageInfo {
  name : String
  version : String
  tag : String
}

fn normalize_manifest_path(value : String) -> String? {
  let path = value.trim().to_owned()
  if path == "" {
    return None
  }
  if path.has_prefix("http://") || path.has_prefix("https://") || path.has_prefix("#") {
    return None
  }
  Some(path)
}

fn push_unique_string(items : Array[String], value : String) -> Unit {
  for item in items {
    if item == value {
      return
    }
  }
  items.push(value)
}

fn collect_manifest_paths(value : Json, paths : Array[String]) -> Unit {
  match value {
    String(path) => {
      match normalize_manifest_path(path) {
        Some(path) => push_unique_string(paths, path)
        None => ()
      }
    }
    Array(items) => {
      for item in items {
        collect_manifest_paths(item, paths)
      }
    }
    Object(obj) => {
      for _, child in obj {
        collect_manifest_paths(child, paths)
      }
    }
    _ => ()
  }
}

fn validate_publish_entrypoints(directory : String, package_json : Json) -> Array[String] {
  let missing = []
  guard package_json is Object(package_obj) else {
    missing.push("package.json root is not an object")
    return missing
  }

  let manifest_paths = []
  if package_obj.get("main") is Some(value) {
    collect_manifest_paths(value, manifest_paths)
  }
  if package_obj.get("types") is Some(value) {
    collect_manifest_paths(value, manifest_paths)
  }
  if package_obj.get("bin") is Some(value) {
    collect_manifest_paths(value, manifest_paths)
  }
  if package_obj.get("exports") is Some(value) {
    collect_manifest_paths(value, manifest_paths)
  }

  for manifest_path in manifest_paths {
    let resolved = @tool.join_path(directory, manifest_path)
    if !@tool.path_exists(resolved) {
      push_unique_string(missing, manifest_path)
    }
  }

  missing
}

fn find_repo_root(start : String) -> String? {
  let current : @path.Path = start
  let current_path = current.resolve().to_string()
  if @tool.path_exists(@tool.join_path(current_path, "pnpm-workspace.yaml")) &&
    @tool.path_exists(@tool.join_path(current_path, "npm")) {
    return Some(current_path)
  }
  let parent = current.dirname().to_string()
  if parent == current_path {
    None
  } else {
    find_repo_root(parent)
  }
}

/// Reads and parses a `package.json`, returning `None` on failure.
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

/// Parses a positive integer and falls back on invalid input.
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

fn env_positive_int(name : String, fallback : Int) -> Int {
  match @env.get_env_var(name) {
    Some(value) if value != "" => positive_int_or(value, fallback)
    _ => fallback
  }
}

/// Derives the npm dist-tag for a package version.
fn npm_tag(version : String) -> String {
  match @env.get_env_var("NPM_TAG") {
    Some(value) if value != "" => value
    _ =>
      if version.contains("-alpha") {
        "alpha"
      } else if version.contains("-beta") {
        "beta"
      } else if version.contains("-rc") {
        "rc"
      } else {
        "latest"
      }
  }
}

fn shell_quote(value : String) -> String {
  "'" + value.replace_all(old="'", new="'\"'\"'") + "'"
}

fn child_script_command(
  moon_bin : String,
  script_path : String,
  args : Array[String],
) -> String {
  let parts = [
    shell_quote(moon_bin),
    "run",
    "-q",
    "--target",
    "native",
    shell_quote(script_path),
    "--",
  ]
  for arg in args {
    parts.push(shell_quote(arg))
  }
  parts.join(" ")
}

/// Resolves workspace and catalog dependency protocols before publishing.
/// Release artifacts are published from downloaded directories rather than a
/// fully installed workspace, so we proactively rewrite protocol-based version
/// specs to concrete npm-compatible strings. This keeps the publish path
/// deterministic and removes a class of release-only pnpm failures.
async fn prepare_publish_manifest(directory : String) -> Int {
  let start_dir = match @env.get_env_var("VIZE_REPO_ROOT") {
    Some(value) if value != "" => @tool.resolve_path(value)
    _ => @tool.resolve_path(@env.current_dir().unwrap_or("."))
  }
  let repo_root = match find_repo_root(start_dir) {
    Some(path) => path
    None => return 0
  }
  let moon_bin = @tool.tool_bin("MOON_BIN", "moon")
  let script_path = @tool.join_path(repo_root, "tools/moon/cmd/prepare_npm_publish_manifest")
  let exit_code = @process.run(
    "sh",
    ["-c", child_script_command(moon_bin, script_path, [directory])],
    cwd=repo_root,
  )
  if exit_code != 0 {
    @stdio.stderr.write("Failed to prepare npm publish manifest for \{directory}\n")
  }
  exit_code
}

/// Recursively checks whether a JSON value contains the expected string.
/// Registry commands sometimes return a bare string and sometimes return JSON
/// arrays or objects, depending on the field and the package manager wrapper.
/// This helper keeps the visibility checks tolerant to those shape changes.
fn json_contains_string(value : Json, expected : String) -> Bool {
  match value {
    String(text) => text == expected
    Array(items) => {
      for item in items {
        if json_contains_string(item, expected) {
          return true
        }
      }
      false
    }
    _ => false
  }
}

/// Returns `true` when a `dist-tags` JSON object maps `tag` to `version`.
fn json_dist_tag_matches(value : Json, tag : String, version : String) -> Bool {
  guard value is Object(obj) else { return false }
  guard obj.get(tag) is Some(String(tag_version)) else { return false }
  tag_version == version
}

/// Checks whether the exact package version is visible from npm registry reads.
/// Visibility is more important than the publish command's exit code: if the
/// registry already serves `name@version`, rerunning the workflow should treat
/// that state as success rather than attempting to publish again.
async fn npm_version_is_visible(vp_bin : String, name : String, version : String) -> Bool {
  let (exit_code, stdout, _) = @process.collect_output(
    vp_bin,
    ["pm", "view", name + "@" + version, "version", "--json"],
  )
  if exit_code != 0 {
    return false
  }
  let body = stdout.text().trim()
  if body == "" {
    return false
  }
  if body == version || body == "\"" + version + "\"" {
    return true
  }
  let parsed = try {
    @json.parse(body)
  } catch {
    _ => return false
  }
  json_contains_string(parsed, version)
}

/// Checks whether the package's dist-tag has propagated to the expected version.
/// For consumers of prerelease channels, the dist-tag is often the real source
/// of truth. We therefore wait for both the version and the dist-tag instead of
/// declaring success immediately after `publish` returns.
async fn npm_tag_points_to_version(
  vp_bin : String,
  name : String,
  tag : String,
  version : String,
) -> Bool {
  let (exit_code, stdout, _) = @process.collect_output(
    vp_bin,
    ["pm", "view", name, "dist-tags", "--json"],
  )
  if exit_code != 0 {
    return false
  }
  let body = stdout.text().trim()
  if body == "" {
    return false
  }
  let parsed = try {
    @json.parse(body)
  } catch {
    _ => return false
  }
  json_dist_tag_matches(parsed, tag, version)
}

/// Polls the registry until both the version and dist-tag become visible.
/// Publishing to npm is eventually consistent. This helper turns that
/// propagation window into an explicit, bounded retry loop so workflow failures
/// point to a real timeout rather than a transient post-publish race.
async fn wait_for_registry_visibility(
  vp_bin : String,
  info : NpmPackageInfo,
  max_attempts : Int,
  delay_seconds : Int,
) -> Int {
  let mut attempt = 1
  while attempt <= max_attempts {
    let version_visible = npm_version_is_visible(vp_bin, info.name, info.version)
    let tag_visible = npm_tag_points_to_version(vp_bin, info.name, info.tag, info.version)
    if version_visible && tag_visible {
      println(
        "\{info.name}@\{info.version} is visible in npm with dist-tag \{info.tag}.",
      )
      return 0
    }
    if attempt >= max_attempts {
      @stdio.stderr.write(
        "Timed out waiting for \{info.name}@\{info.version} to appear in npm with dist-tag \{info.tag}.\n",
      )
      return 1
    }
    println(
      "Waiting for \{info.name}@\{info.version} to appear in npm with dist-tag \{info.tag}... (\{attempt}/\{max_attempts})",
    )
    @async.sleep(delay_seconds * 1000)
    attempt = attempt + 1
  }
  1
}

/// Main entry point for resilient single-package npm publishing.
/// The function first decides whether publishing is necessary, then performs
/// retries around `vp pm publish`, and finally waits for registry propagation.
/// `--ignore-failure` only affects the final hard failure after all retries and
/// visibility checks have been exhausted.
async fn run_app() -> Int {
  let args = @tool.cli_args()
  if args.is_empty() {
    @stdio.stderr.write(
      "Usage: moon run --target native tools/moon/cmd/publish_npm_package -- <dir> [--provenance] [--ignore-failure] [--retries N] [--delay N]\n",
    )
    return 1
  }

  let directory = @tool.resolve_path(args[0])
  let vp_bin = @tool.tool_bin("VP_BIN", "vp")
  let mut retries = env_positive_int("PUBLISH_RETRY_LIMIT", 3)
  let mut delay_seconds = env_positive_int("PUBLISH_RETRY_DELAY", 15)
  let resolution_retries = env_positive_int("PUBLISH_RESOLUTION_RETRY_LIMIT", retries)
  let resolution_delay_seconds = env_positive_int(
    "PUBLISH_RESOLUTION_RETRY_DELAY",
    delay_seconds,
  )
  let mut provenance = false
  let mut ignore_failure = false
  let mut index = 1
  while index < args.length() {
    let arg = args[index]
    if arg == "--provenance" {
      provenance = true
      index = index + 1
      continue
    }
    if arg == "--ignore-failure" {
      ignore_failure = true
      index = index + 1
      continue
    }
    if arg == "--retries" && index + 1 < args.length() {
      retries = positive_int_or(args[index + 1], retries)
      index = index + 2
      continue
    }
    if arg == "--delay" && index + 1 < args.length() {
      delay_seconds = positive_int_or(args[index + 1], delay_seconds)
      index = index + 2
      continue
    }
    @stdio.stderr.write("Unknown argument: \{arg}\n")
    return 1
  }

  let package_json_path = @tool.join_path(directory, "package.json")
  let prepare_exit_code = prepare_publish_manifest(directory)
  if prepare_exit_code != 0 {
    return prepare_exit_code
  }
  let package_json = match read_package_json(package_json_path) {
    Some(json) => json
    None => {
      @stdio.stderr.write("Failed to parse \{package_json_path}\n")
      return 1
    }
  }
  let version = match package_version(package_json) {
    Some(version) => version
    None => {
      @stdio.stderr.write("Failed to read version from \{package_json_path}\n")
      return 1
    }
  }
  let name = match package_name(package_json) {
    Some(name) => name
    None => {
      @stdio.stderr.write("Failed to read name from \{package_json_path}\n")
      return 1
    }
  }
  let info = NpmPackageInfo::{
    name,
    version,
    tag: npm_tag(version),
  }
  let missing_entrypoints = validate_publish_entrypoints(directory, package_json)
  if !missing_entrypoints.is_empty() {
    @stdio.stderr.write(
      "Refusing to publish \{info.name}@\{info.version} because package.json points at files that are missing from \{directory}:\n",
    )
    for missing_path in missing_entrypoints {
      @stdio.stderr.write("  - \{missing_path}\n")
    }
    return 1
  }
  let publish_args = [
    "pm",
    "publish",
    "--access",
    "public",
    "--no-git-checks",
    "--tag",
    info.tag,
  ]
  if provenance {
    publish_args.push("--")
    publish_args.push("--provenance")
  }

  let max_attempts = if retries > 0 { retries } else { 1 }
  if npm_version_is_visible(vp_bin, info.name, info.version) {
    println(
      "\{info.name}@\{info.version} is already published. Skipping publish and verifying registry visibility.",
    )
    return wait_for_registry_visibility(
      vp_bin,
      info,
      resolution_retries,
      resolution_delay_seconds,
    )
  }
  let mut attempt = 1
  while attempt <= max_attempts {
    let command_display = publish_args.join(" ")
    println(
      "Attempt \{attempt}/\{max_attempts}: publish \{info.name}@\{info.version} with \{vp_bin} \{command_display}",
    )
    let (exit_code, stdout, stderr) = @process.collect_output(
      vp_bin,
      publish_args,
      cwd=directory,
    )
    let stdout_text = stdout.text().trim()
    let stderr_text = stderr.text().trim()
    if stdout_text != "" {
      println(stdout_text)
    }
    if stderr_text != "" {
      @stdio.stderr.write(stderr_text + "\n")
    }
    if exit_code == 0 {
      return wait_for_registry_visibility(
        vp_bin,
        info,
        resolution_retries,
        resolution_delay_seconds,
      )
    }
    if npm_version_is_visible(vp_bin, info.name, info.version) {
      println(
        "\{info.name}@\{info.version} appears in npm despite a non-zero publish exit. Verifying registry visibility.",
      )
      return wait_for_registry_visibility(
        vp_bin,
        info,
        resolution_retries,
        resolution_delay_seconds,
      )
    }
    let publish_detail = stdout_text + "\n" + stderr_text
    if publish_detail.contains("Cannot publish over previously staged version") {
      println(
        "\{info.name}@\{info.version} is staged in npm but not yet visible. Waiting for registry visibility before retrying.",
      )
      return wait_for_registry_visibility(
        vp_bin,
        info,
        resolution_retries,
        resolution_delay_seconds,
      )
    }
    if attempt < max_attempts {
      println(
        "Publish attempt failed for \{info.name}@\{info.version}. Retrying in \{delay_seconds}s...",
      )
      @async.sleep(delay_seconds * 1000)
    }
    attempt = attempt + 1
  }

  if ignore_failure {
    println(
      "Failed to publish \{info.name}@\{info.version} from \{directory}, but continuing because --ignore-failure is set.",
    )
    return 0
  }
  @stdio.stderr.write("Failed to publish \{info.name}@\{info.version} from \{directory}\n")
  1
}

async fn main {
  @sys.exit(run_app())
}
