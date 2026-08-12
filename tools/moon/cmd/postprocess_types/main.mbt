
/// Writes a complete file and reports success as a boolean.
/// The generated declaration is rewritten atomically from the caller's point of
/// view: either the replacement content was accepted by the filesystem helper,
/// or the script exits with a clear failure message.
fn write_string_to_file(path : String, content : String) -> Bool {
  try {
    @xfs.write_string_to_file(path, content)
    true
  } catch {
    _ => false
  }
}

/// Walks upward from `start` until it finds the Vize repository root.
/// Two markers are checked to avoid accidentally treating a nested fixture or
/// package directory as the root. Tests can bypass this search by setting
/// `VIZE_REPO_ROOT` to an isolated fixture workspace.
fn find_repo_root(start : String) -> String? {
  let current : @path.Path = start
  let current_path = current.resolve().to_string()
  if @tool.path_exists(@tool.join_path(current_path, "Cargo.toml")) &&
    @tool.path_exists(@tool.join_path(current_path, "npm/cli/package.json")) {
    return Some(current_path)
  }
  let parent = current.dirname().to_string()
  if parent == current_path {
    None
  } else {
    find_repo_root(parent)
  }
}

/// Returns the generated TypeScript declaration path relative to the repository
/// root.
fn generated_types_path(root : String) -> String {
  @tool.join_path(root, "npm/cli/src/types/generated.ts")
}

/// Applies Vize-specific declaration patches that are intentionally kept out of
/// the generic JSON Schema generation step.
/// The transform is idempotent. That property matters because release and local
/// generation tasks may be rerun after a partial failure; repeated execution
/// should converge on the same file rather than stacking duplicate aliases.
fn postprocess_generated_types(content : String) -> String {
  let content = content.replace_all(old="MuseaA11YConfig", new="MuseaA11yConfig")
  let content = content.replace_all(
    old="include?: string | string[];",
    new="include?: string | RegExp | (string | RegExp)[];",
  )
  let content = content.replace_all(
    old="exclude?: string | string[];",
    new="exclude?: string | RegExp | (string | RegExp)[];",
  )
  let schema_property =
    "  /**\n" +
    "   * JSON Schema reference for editor autocompletion\n" +
    "   */\n" +
    "  $schema?: string;\n"
  let content = content.replace(old=schema_property, new="")

  if content.contains("export type LintPreset = ") {
    return content
  }

  let marker = "regenerate this file.\n */\n"
  let type_aliases =
    "\n" +
    "export type LintPreset = \"happy-path\" | \"opinionated\" | \"essential\" | \"incremental\" | \"ecosystem\" | \"nuxt\";\n" +
    "\n" +
    "export type RuleSeverity = \"off\" | \"warn\" | \"error\";\n" +
    "\n" +
    "export type RuleCategory = \"correctness\" | \"suspicious\" | \"style\" | \"perf\" | \"a11y\" | \"security\";\n"

  content.replace(old=marker, new=marker + type_aliases)
}

/// Main command implementation for generated config type post-processing.
/// The function resolves the repository root, reads the generated declaration,
/// applies the deterministic declaration patches, and writes the result back to
/// disk. Returning an exit code keeps `main` trivial and makes tests exercise the
/// same control flow as the CLI invocation.
fn run_app() -> Int {
  let repo_root = match @env.get_env_var("VIZE_REPO_ROOT") {
    Some(value) => {
      let path : @path.Path = value
      path.resolve().to_string()
    }
    None =>
      match find_repo_root(@env.current_dir().unwrap_or(".")) {
        Some(path) => path
        None => {
          println("Failed to locate the repository root for postprocess_types")
          return 1
        }
      }
  }
  let output_path = generated_types_path(repo_root)
  let content = match @tool.read_file_to_string(output_path) {
    Some(content) => content
    None => {
      println("Failed to read \{output_path}")
      return 1
    }
  }
  let next_content = postprocess_generated_types(content)
  if !write_string_to_file(output_path, next_content) {
    println("Failed to write \{output_path}")
    return 1
  }
  println("Post-processed generated types: \{output_path}")
  0
}

/// Script entry point.
fn main {
  @sys.exit(run_app())
}
