///|
/// Mutate and commit only the isolated candidate prepared from current main.
async fn prepare_candidate(
  cargo_toml : String,
  current_version : String,
  new_version : String,
) -> Int {
  let cargo_toml_path = "Cargo.toml"
  let tag = "v" + new_version
  if !run_local_release_guard(tag) {
    return 1
  }

  if !verify_manifest_alignment_before_mutation() {
    return 1
  }

  println("Updating Cargo.toml...")
  if !write_string_to_file(
      cargo_toml_path,
      update_workspace_manifest_versions(
        cargo_toml, current_version, new_version,
      ),
    ) {
    @stdio.stderr.write("Failed to update Cargo.toml\n")
    return 1
  }

  println("Updating Cargo.lock...")
  if @process.run("cargo", ["update", "--workspace"]) != 0 {
    @stdio.stderr.write("Failed to update Cargo.lock\n")
    return 1
  }

  let files_to_stage = ["Cargo.toml", "Cargo.lock"]
  println("Updating npm packages...")
  for entry_name in @tool.read_dir("npm") {
    let package_dir = @tool.join_path("npm", entry_name)
    if !@tool.is_dir(package_dir) {
      continue
    }
    let package_json_path = @tool.join_path(package_dir, "package.json")
    if @tool.path_exists(package_json_path) {
      let package_json = match @tool.read_file_to_string(package_json_path) {
        Some(content) => content
        None => {
          @stdio.stderr.write("Failed to read \{package_json_path}\n")
          return 1
        }
      }
      if !write_string_to_file(
          package_json_path,
          replace_package_json_version(
            package_json, current_version, new_version,
          ),
        ) {
        @stdio.stderr.write("Failed to update \{package_json_path}\n")
        return 1
      }
      println("  Updated \{package_json_path}")
      files_to_stage.push(package_json_path)
    }
  }
  for package_json_path in extra_release_package_json_paths() {
    if !@tool.path_exists(package_json_path) {
      continue
    }
    let package_json = match @tool.read_file_to_string(package_json_path) {
      Some(content) => content
      None => {
        @stdio.stderr.write("Failed to read \{package_json_path}\n")
        return 1
      }
    }
    if !write_string_to_file(
        package_json_path,
        replace_package_json_version(package_json, current_version, new_version),
      ) {
      @stdio.stderr.write("Failed to update \{package_json_path}\n")
      return 1
    }
    println("  Updated \{package_json_path}")
    files_to_stage.push(package_json_path)
  }

  let workspace_yaml_path = "pnpm-workspace.yaml"
  let workspace_yaml = match @tool.read_file_to_string(workspace_yaml_path) {
    Some(content) => content
    None => {
      @stdio.stderr.write("Failed to read \{workspace_yaml_path}\n")
      return 1
    }
  }
  let updated_workspace_yaml = replace_native_binary_workspace_catalog(
    workspace_yaml, current_version, new_version,
  )
  if updated_workspace_yaml != workspace_yaml {
    if !write_string_to_file(workspace_yaml_path, updated_workspace_yaml) {
      @stdio.stderr.write("Failed to update \{workspace_yaml_path}\n")
      return 1
    }
    println("  Updated \{workspace_yaml_path}")
    files_to_stage.push(workspace_yaml_path)
  }

  let lockfile_path = "pnpm-lock.yaml"
  let lockfile = match @tool.read_file_to_string(lockfile_path) {
    Some(content) => content
    None => {
      @stdio.stderr.write("Failed to read \{lockfile_path}\n")
      return 1
    }
  }
  let updated_lockfile = replace_native_binary_lockfile_catalog(
    lockfile, current_version, new_version,
  )
  if updated_lockfile != lockfile {
    if !write_string_to_file(lockfile_path, updated_lockfile) {
      @stdio.stderr.write("Failed to update \{lockfile_path}\n")
      return 1
    }
    println("  Updated \{lockfile_path} native-binaries catalog block")
    files_to_stage.push(lockfile_path)
  }

  let zed_cargo_toml = "editors/zed/Cargo.toml"
  let zed_extension_toml = "editors/zed/extension.toml"
  let zed_cargo_lock = "editors/zed/Cargo.lock"
  if @tool.path_exists(zed_cargo_toml) {
    let content = match @tool.read_file_to_string(zed_cargo_toml) {
      Some(content) => content
      None => {
        @stdio.stderr.write("Failed to read \{zed_cargo_toml}\n")
        return 1
      }
    }
    if !write_string_to_file(
        zed_cargo_toml,
        replace_first_version_line(content, current_version, new_version),
      ) {
      @stdio.stderr.write("Failed to update \{zed_cargo_toml}\n")
      return 1
    }
    println("  Updated \{zed_cargo_toml}")
    files_to_stage.push(zed_cargo_toml)
  }
  if @tool.path_exists(zed_extension_toml) {
    let content = match @tool.read_file_to_string(zed_extension_toml) {
      Some(content) => content
      None => {
        @stdio.stderr.write("Failed to read \{zed_extension_toml}\n")
        return 1
      }
    }
    if !write_string_to_file(
        zed_extension_toml,
        replace_first_version_line(content, current_version, new_version),
      ) {
      @stdio.stderr.write("Failed to update \{zed_extension_toml}\n")
      return 1
    }
    println("  Updated \{zed_extension_toml}")
    files_to_stage.push(zed_extension_toml)
  }
  if @tool.path_exists(zed_cargo_lock) {
    if @process.run("cargo", [
        "update", "--manifest-path", "editors/zed/Cargo.toml", "-p", "vize-zed-extension",
      ]) !=
      0 {
      @stdio.stderr.write("Failed to update editors/zed/Cargo.lock\n")
      return 1
    }
    println("  Updated \{zed_cargo_lock}")
    files_to_stage.push(zed_cargo_lock)
  }

  // These standalone workspaces depend on the in-tree SDK and build with
  // --locked. Updating the root workspace lockfile cannot refresh them.
  for manifest in [
    "crates/vize_extension_host/tests/guests/expression-echo/Cargo.toml",
    "crates/vize_extension_host/tests/guests/output-echo/Cargo.toml",
    "examples/volt-target/Cargo.toml",
  ] {
    if !@tool.path_exists(manifest) {
      continue
    }
    if @process.run("cargo", [
        "update", "--manifest-path", manifest, "-p", "vize_extension_sdk", "--offline",
      ]) != 0 {
      @stdio.stderr.write("Failed to update the SDK guest lockfile for \{manifest}\n")
      return 1
    }
    let guest_lockfile = manifest.replace(old="Cargo.toml", new="Cargo.lock")
    println("  Updated \{guest_lockfile}")
    files_to_stage.push(guest_lockfile)
  }

  if !update_release_readmes(current_version, new_version, files_to_stage) {
    return 1
  }

  println("Committing changes...")
  let git_add_args = ["add"]
  for file in stage_files(files_to_stage) {
    git_add_args.push(file)
  }
  if @process.run("git", git_add_args) != 0 {
    @stdio.stderr.write("Failed to stage release files\n")
    return 1
  }

  let (staged_exit, staged_output) = @process.collect_stdout("git", [
    "diff", "--cached", "--name-only",
  ])
  if staged_exit != 0 {
    @stdio.stderr.write("Failed to inspect staged release files\n")
    return 1
  }
  if staged_output.text().trim() == "" {
    @stdio.stderr.write(
      "No release changes were staged; refusing to create an empty release.\n",
    )
    return 1
  }
  let commit_args = ["commit", "--no-verify", "-m", "chore: release \{tag}"]
  if @process.run("git", commit_args) != 0 {
    @stdio.stderr.write("Failed to create the release commit\n")
    return 1
  }

  if !verify_manifest_alignment_after_commit() {
    return 1
  }

  println("Release candidate commit prepared; no tag was created.")
  0
}
