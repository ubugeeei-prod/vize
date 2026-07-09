
fn split_groups(args : Array[String]) -> Array[Array[String]] {
  let groups : Array[Array[String]] = []
  let current : Array[String] = []
  for arg in args {
    if arg == "--" {
      if !current.is_empty() {
        groups.push(current.copy())
        current.clear()
      }
      continue
    }
    current.push(arg)
  }
  if !current.is_empty() {
    groups.push(current.copy())
  }
  groups
}

async fn run_app() -> Int {
  let groups = split_groups(@tool.cli_args())
  if groups.is_empty() {
    println(
      "Usage: moon run --target native tools/moon/cmd/github/run_many -- <cmd...> -- <cmd...>",
    )
    return 1
  }
  for group in groups {
    let command = group[0]
    let command_args = []
    for i in 1..<group.length() {
      command_args.push(group[i])
    }
    let exit_code = @process.run(command, command_args)
    if exit_code != 0 {
      return exit_code
    }
  }
  0
}

async fn main {
  @sys.exit(run_app())
}
