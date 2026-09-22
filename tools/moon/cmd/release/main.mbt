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
      "Usage: moon run --target native tools/moon/cmd/release -- <patch|minor|major|alpha|beta|rc|release> [-y]\n",
    )
    return 1
  }

  if args.length() == 2 && args[0] == "--resume" {
    return @process.run("rust-script", [
      "tools/commands/release/pr.rs",
      "resume",
      args[1],
    ])
  }
  let bump = args[0]
  let mut prepare_only = false
  let mut auto_confirm = false
  for i in 1..<args.length() {
    if args[i] == "--prepare-only" {
      prepare_only = true
      continue
    }
    if args[i] == "-y" {
      auto_confirm = true
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
  let new_version = match bump_version(current_version, bump) {
    Some(version) => version
    None => {
      @stdio.stderr.write("Failed to compute the next version\n")
      return 1
    }
  }
  let tag = "v" + new_version

  println("Current version: \{current_version}")
  println("New version: \{new_version} (tag: \{tag})")
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
