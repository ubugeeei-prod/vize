
fn safe_is_file(path : String) -> Bool {
  try {
    @xfs.is_file(path)
  } catch {
    _ => false
  }
}

fn safe_remove_file(path : String) -> Unit {
  try {
    @xfs.remove_file(path)
  } catch {
    _ => ()
  }
}

fn run_app() -> Int {
  let directories = @tool.cli_args()
  if directories.is_empty() {
    println(
      "Usage: moon run --target native tools/moon/cmd/github/clean_node_binaries -- <dir> [dir...]",
    )
    return 1
  }
  for directory in directories {
    for entry in @tool.read_dir(directory) {
      let entry_name = entry.to_string()
      let entry_path = @tool.join_path(directory, entry_name)
      if entry_name.has_suffix(".node") && safe_is_file(entry_path) {
        safe_remove_file(entry_path)
      }
    }
  }
  0
}

fn main {
  @sys.exit(run_app())
}
