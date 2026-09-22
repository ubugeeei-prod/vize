///|
/// Replaces the first package.json `"version"` field that matches the current release.
fn replace_package_json_version(
  content : String,
  current_version : String,
  new_version : String,
) -> String {
  let updated : Array[String] = []
  let mut replaced = false
  let lines : Array[String] = content
    .split("\n")
    .map(view => view.to_owned())
    .collect()
  for line in lines {
    if !replaced {
      let trimmed = line.trim()
      if trimmed.has_prefix("\"version\"") && line.contains(current_version) {
        updated.push(line.replace(old=current_version, new=new_version))
        replaced = true
        continue
      }
    }
    updated.push(line)
  }
  updated.join("\n")
}

///|
/// Nested npm group directories (npm/builder, npm/framework, npm/compose,
/// npm/ui, ...) hold their packages one level deeper than the flat
/// npm/<package> layout. Scanning them keeps every new group aligned with the
/// release version without editing a hardcoded list — the release preflight
/// verifies all tracked non-private manifests, so any package this scan
/// misses fails the release before publication.
async fn nested_npm_package_dirs() -> Array[String] {
  let package_dirs : Array[String] = []
  for group_name in @tool.read_dir("npm") {
    let group_dir = @tool.join_path("npm", group_name)
    if !@tool.is_dir(group_dir) {
      continue
    }
    if @tool.path_exists(@tool.join_path(group_dir, "package.json")) {
      continue
    }
    for entry_name in @tool.read_dir(group_dir) {
      let package_dir = @tool.join_path(group_dir, entry_name)
      if !@tool.is_dir(package_dir) {
        continue
      }
      let package_json_path = @tool.join_path(package_dir, "package.json")
      if !@tool.path_exists(package_json_path) {
        continue
      }
      // Mirror the release preflight exactly: it verifies tracked,
      // non-private manifests, so ignored build output and private
      // packages stay out of the release version sweep.
      if is_git_ignored(package_json_path) {
        continue
      }
      match @tool.read_file_to_string(package_json_path) {
        Some(content) => if content.contains("\"private\": true") { continue }
        None => continue
      }
      package_dirs.push(package_dir)
    }
  }
  package_dirs.sort_by((a, b) => a.lexical_compare(b))
  package_dirs
}

///|
/// Package manifests that are intentionally outside the npm workspace package
/// globs but still ship release artifacts and must stay aligned with the
/// workspace version.
async fn extra_release_package_json_paths() -> Array[String] {
  let paths : Array[String] = [
    "editors/vscode/package.json", "editors/vscode-art/package.json",
  ]
  for package_dir in nested_npm_package_dirs() {
    paths.push(@tool.join_path(package_dir, "package.json"))
  }
  paths
}

///|
/// Replaces the first TOML `version = "..."` line in a file.
/// This is used for the Zed extension manifests where a full TOML parser would
/// be overkill for the repository's simple version synchronization needs.
fn replace_first_version_line(
  content : String,
  current_version : String,
  new_version : String,
) -> String {
  let updated : Array[String] = []
  let mut replaced = false
  let lines : Array[String] = content
    .split("\n")
    .map(view => view.to_owned())
    .collect()
  for line in lines {
    if !replaced {
      let trimmed = line.trim()
      if trimmed.has_prefix("version = \"") &&
        trimmed.has_suffix("\"") &&
        line.contains(current_version) {
        updated.push(line.replace(old=current_version, new=new_version))
        replaced = true
        continue
      }
    }
    updated.push(line)
  }
  updated.join("\n")
}

///|
/// Detects whether the script can safely prompt the user for confirmation.
/// The release script is allowed to be interactive when run locally, but CI and
/// sessions without a controlling terminal must fail explicitly instead of
/// hanging on stdin reads.
async fn has_interactive_terminal() -> Bool {
  @process.run("sh", ["-c", ": </dev/tty >/dev/tty"]) == 0
}

///|
/// Prompts through the controlling terminal because task runners keep stdin.
async fn confirm_release() -> Bool {
  let command = "printf 'Proceed with release? [y/N] ' > /dev/tty; IFS= read -r answer < /dev/tty; case \"$answer\" in y|Y|yes|Yes|YES) exit 0 ;; *) exit 1 ;; esac"
  @process.run("sh", ["-c", command]) == 0
}

///|
/// Filters a list of candidate paths down to the ones that currently exist.
/// The release commit stages files opportunistically because some package
/// variants are optional in this repository.
fn stage_files(files : Array[String]) -> Array[String] {
  let staged = []
  for file in files {
    if @tool.path_exists(file) {
      staged.push(file)
    }
  }
  staged
}

///|
/// Runs the shared immutable-tag and current-main guard before any mutation.
/// Tests may override the script path while exercising isolated repositories.
async fn run_local_release_guard(tag : String) -> Bool {
  let script = match @env.get_env_var("VIZE_RELEASE_GUARD_SCRIPT") {
    Some(value) if value != "" => value
    _ => "tools/commands/ci/github/release-local-guard.rs"
  }
  if !@tool.path_exists(script) {
    @stdio.stderr.write("Release preflight script not found: " + script + "\n")
    return false
  }

  let runner = match @env.get_env_var("VIZE_RELEASE_GUARD_RUNNER") {
    Some(value) if value != "" => value
    _ => if script.has_suffix(".mjs") { "node" } else { "rust-script" }
  }
  let exit_code = @process.run(runner, [script, tag, "--candidate"])
  if exit_code != 0 {
    @stdio.stderr.write(
      "Release preflight failed before repository mutation.\n",
    )
    return false
  }

  true
}
