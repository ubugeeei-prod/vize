
/// Quotes a single shell argument for the POSIX shell used in CI and tests.
/// This script delegates to the `publish_npm_package` command package through
/// `moon run`. The delegation currently flows through `sh -c` so the command
/// can keep using the same shell-safe argument quoting on every CI runner.
fn shell_quote(value : String) -> String {
  "'" + value.replace_all(old="'", new="'\"'\"'") + "'"
}

/// Builds a shell-safe nested `moon run` command for one package publish.
fn child_publish_command(
  moon_bin : String,
  script_path : String,
  package_dir : String,
  forward_args : Array[String],
) -> String {
  let parts = [
    shell_quote(moon_bin),
    "run",
    "-q",
    "--target",
    "native",
    shell_quote(script_path),
    "--",
    shell_quote(package_dir),
  ]
  for arg in forward_args {
    parts.push(shell_quote(arg))
  }
  parts.join(" ")
}

/// Main entry point for publishing a directory of generated npm packages.
/// The script only forwards subdirectories that contain a `package.json`, which
/// prevents CI helper folders from being treated as publish targets. Each child
/// package is published serially so failures are attributed to a single package
/// and registry rate limits are less likely to be triggered.
async fn run_app() -> Int {
  let args = @tool.cli_args()
  if args.is_empty() {
    @stdio.stderr.write(
      "Usage: moon run --target native tools/moon/cmd/publish_npm_package_dirs -- <dir> [--provenance] [--ignore-failure] [--retries N] [--delay N]\n",
    )
    return 1
  }

  let base_directory = @tool.resolve_path(args[0])
  let forward_args = []
  for i in 1..<args.length() {
    forward_args.push(args[i])
  }

  let current_dir = @tool.resolve_path(@env.current_dir().unwrap_or("."))
  let moon_bin = @tool.tool_bin("MOON_BIN", "moon")
  let script_path = @tool.join_path(
    current_dir,
    "tools/moon/cmd/publish_npm_package",
  )
  let package_directories = []
  for entry_name in @tool.read_dir(base_directory) {
    let package_dir = @tool.join_path(base_directory, entry_name)
    if @tool.is_dir(package_dir) && @tool.path_exists(@tool.join_path(package_dir, "package.json")) {
      package_directories.push(package_dir)
    }
  }
  package_directories.sort_by((a, b) => a.lexical_compare(b))
  if package_directories.is_empty() {
    @stdio.stderr.write("No publishable package directories were found in \{base_directory}\n")
    return 1
  }

  for package_dir in package_directories {
    println("Publishing \{package_dir}...")
    let exit_code = @process.run(
      "sh",
      ["-c", child_publish_command(moon_bin, script_path, package_dir, forward_args)],
      cwd=current_dir,
    )
    if exit_code != 0 {
      return exit_code
    }
  }
  0
}

async fn main {
  @sys.exit(run_app())
}
