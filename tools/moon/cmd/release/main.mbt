///|
/// Confirm the requested version bump, then create and watch a release PR.
/// The internal prepare-only mode edits one isolated worktree without a tag.
async fn run_app() -> Int {
  let args = @tool.cli_args()
  if args.length() == 3 && args[0] == "--print-bump" {
    match bump_version(args[1], args[2]) {
      Some(version) => {
        println(version)
        return 0
      }
      None => return 1
    }
  }
  if args.length() == 4 && args[0] == "--print-readme-version-update" {
    match @tool.read_file_to_string(args[1]) {
      Some(content) => {
        @stdio.stdout.write(
          replace_readme_version_references(content, args[2], args[3]),
        )
        return 0
      }
      None => return 1
    }
  }
  if args.length() == 4 && args[0] == "--print-workspace-manifest-update" {
    match @tool.read_file_to_string(args[1]) {
      Some(content) => {
        @stdio.stdout.write(
          update_workspace_manifest_versions(content, args[2], args[3]),
        )
        return 0
      }
      None => return 1
    }
  }
  if args.length() == 4 && args[0] == "--print-workspace-catalog-update" {
    let input = match @tool.read_file_to_string(args[1]) {
      Some(text) => text
      None => {
        @stdio.stderr.write("Failed to read \{args[1]}\n")
        return 1
      }
    }
    @stdio.stdout.write(
      replace_native_binary_workspace_catalog(input, args[2], args[3]),
    )
    return 0
  }
  if args.length() == 4 && args[0] == "--print-lockfile-catalog-update" {
    let input = match @tool.read_file_to_string(args[1]) {
      Some(text) => text
      None => {
        @stdio.stderr.write("Failed to read \{args[1]}\n")
        return 1
      }
    }
    @stdio.stdout.write(
      replace_native_binary_lockfile_catalog(input, args[2], args[3]),
    )
    return 0
  }
  if args.length() == 1 && args[0] == "--print-extra-package-json-paths" {
    for path in extra_release_package_json_paths() {
      println(path)
    }
    return 0
  }
  if args.is_empty() {
    @stdio.stderr.write(
      "Usage: moon run --target native tools/moon/cmd/release -- <bump> [-y] [--pin] | --resume <PR> [--pin] | --retire <PR> --head <H> --tag <tag> --run <R> --operator-run <entry>\n",
    )
    return 1
  }

  if args[0] == "--retire" {
    guard args.length() == 10 &&
      args[2] == "--head" &&
      args[4] == "--tag" &&
      args[6] == "--run" &&
      args[8] == "--operator-run" else {
      @stdio.stderr.write(
        "Retirement requires explicit original PR/H/tag/R/operator-run identities\n",
      )
      return 1
    }
    let tag_parts = args[5].split(".").map(part => part.to_owned()).collect()
    guard positive_retirement_identity(args[1]) &&
      positive_retirement_identity(args[7]) &&
      positive_retirement_identity(args[9]) &&
      args[3].length() == 40 &&
      args[3]
      .iter()
      .all(char => {
        (char >= '0' && char <= '9') ||
        (char >= 'a' && char <= 'f') ||
        (char >= 'A' && char <= 'F')
      }) &&
      tag_parts.length() == 3 &&
      tag_parts[0] == "v0" &&
      positive_retirement_identity(tag_parts[1]) &&
      tag_parts[2] == "0" else {
      @stdio.stderr.write("Invalid original retirement identities\n")
      return 1
    }
    return @process.run("rust-script", [
      "tools/commands/release/pr.rs",
      "retire",
      args[1],
      args[3],
      args[5],
      args[7],
      args[9],
    ])
  }

  if args.length() == 2 && args[0] == "--resume" {
    return @process.run("rust-script", [
      "tools/commands/release/pr.rs",
      "resume",
      args[1],
    ])
  }
  if args.length() == 3 && args[0] == "--resume" && args[2] == "--pin" {
    return @process.run("rust-script", [
      "tools/commands/release/pr.rs",
      "resume",
      args[1],
      "--pin",
    ])
  }
  let bump = args[0]
  let mut prepare_only = false
  let mut auto_confirm = false
  let mut pin = false
  let mut pinned_target : String? = None
  let mut target_value_index = -1
  for i in 1..<args.length() {
    if i == target_value_index {
      continue
    }
    if args[i] == "--pinned-target" {
      guard i + 1 < args.length() && pinned_target == None else {
        @stdio.stderr.write("Invalid internal pinned preparation target\n")
        return 1
      }
      pinned_target = Some(args[i + 1])
      target_value_index = i + 1
      continue
    }
    if args[i] == "--prepare-only" {
      prepare_only = true
      continue
    }
    if args[i] == "-y" {
      auto_confirm = true
      continue
    }
    if args[i] == "--pin" {
      pin = true
      continue
    }
    if args[i] == "--force-tag" {
      @stdio.stderr.write(
        "Error: --force-tag is not supported; published release tags are immutable.\n",
      )
      return 1
    }
    if args[i] != "-y" {
      @stdio.stderr.write("Unknown release arguments\n")
      return 1
    }
  }
  if pin && prepare_only {
    @stdio.stderr.write("Error: --pin requires the release PR protocol.\n")
    return 1
  }
  if pinned_target != None && (!prepare_only || bump != "minor") {
    @stdio.stderr.write(
      "Pinned target is internal to validated minor preparation\n",
    )
    return 1
  }

  let cargo_toml_path = "Cargo.toml"
  let cargo_toml = match @tool.read_file_to_string(cargo_toml_path) {
    Some(content) => content
    None => {
      @stdio.stderr.write("Failed to read Cargo.toml\n")
      return 1
    }
  }
  let current_version = match workspace_version_from_content(cargo_toml) {
    Some(version) => version
    None => {
      @stdio.stderr.write("Failed to read current version from Cargo.toml\n")
      return 1
    }
  }
  let ordinary_version = match bump_version(current_version, bump) {
    Some(version) => version
    None => {
      @stdio.stderr.write("Failed to compute the next version\n")
      return 1
    }
  }
  let new_version = match pinned_target {
    Some(target) => {
      if @process.run("rust-script", [
          "tools/commands/release/pr.rs", "validate-preparation-target", bump, current_version,
          target,
        ]) !=
        0 {
        @stdio.stderr.write(
          "Pinned target was not authenticated; no release files changed\n",
        )
        return 1
      }
      target
    }
    None => ordinary_version
  }
  let tag = "v" + new_version

  println("Current version: \{current_version}")
  if pin {
    println(
      "The official pinned driver resolves the next unreserved version from immutable retirement evidence.",
    )
  } else {
    println("New version: \{new_version} (tag: \{tag})")
  }
  println("")

  if !auto_confirm {
    if !has_interactive_terminal() {
      @stdio.stderr.write(
        "Error: Confirmation requires an interactive terminal. Re-run with -y to skip the prompt.\n",
      )
      return 1
    }
    if !confirm_release() {
      println("Aborted.")
      return 1
    }
  }

  if !prepare_only {
    if pin {
      return @process.run("rust-script", [
        "tools/commands/release/pr.rs", "start", bump, "--pin",
      ])
    }
    return @process.run("rust-script", [
      "tools/commands/release/pr.rs", "start", bump,
    ])
  }

  prepare_candidate(cargo_toml, current_version, new_version)
}

///|
async fn main {
  @sys.exit(run_app())
}

///|
fn positive_retirement_identity(value : String) -> Bool {
  !value.is_empty() &&
  !value.has_prefix("0") &&
  value.iter().all(char => char >= '0' && char <= '9')
}
