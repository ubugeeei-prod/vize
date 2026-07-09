
fn basename(path_string : String) -> String {
  let path : @path.Path = path_string
  path.basename().to_owned()
}

fn safe_is_file(path : String) -> Bool {
  try {
    @xfs.is_file(path)
  } catch {
    _ => false
  }
}

fn collect_node_files(directory : String, collected : Array[String]) -> Unit {
  for entry_name in @tool.read_dir(directory) {
    if entry_name == "node_modules" {
      continue
    }
    let entry_path = @tool.join_path(directory, entry_name)
    if @tool.is_dir(entry_path) {
      collect_node_files(entry_path, collected)
    } else if entry_name.has_suffix(".node") && safe_is_file(entry_path) {
      collected.push(entry_path)
    }
  }
}

/// Ensures the destination directory exists before artifacts are copied.
/// Release workflows sometimes download artifacts into package directories that
/// already exist in the checked-out repository. Treating an existing directory
/// as success keeps the helper idempotent and avoids failing late-stage
/// packaging jobs on harmless reruns.
async fn ensure_directory(path : String) -> Unit {
  if @tool.is_dir(path) {
    return
  }
  @afs.mkdir(path, recursive=true, permission=0o755)
}

async fn run_app() -> Int {
  let args = @tool.cli_args()
  if args.length() != 3 {
    println(
      "Usage: moon run --target native tools/moon/cmd/github/collect_native_artifacts -- <source-dir> <output-dir> <label>",
    )
    return 1
  }
  let collected = []
  collect_node_files(args[0], collected)
  if collected.is_empty() {
    println("No \{args[2]} .node files found after build.")
    return 1
  }
  ensure_directory(args[1])
  let copied = []
  for file in collected {
    let destination = @tool.join_path(args[1], basename(file))
    @afs.write_file(
      destination,
      @afs.read_file(file),
      create_mode=CreateOrTruncate,
      permission=0o644,
    )
    copied.push(destination)
  }
  copied.sort()
  println("Collected \{args[2]} artifacts:")
  for file in copied {
    println("  \{file}")
  }
  0
}

async fn main {
  @sys.exit(run_app())
}
