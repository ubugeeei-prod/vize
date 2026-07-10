
fn release_dir(target : String) -> String {
  let base : @path.Path = "target"
  base.join(target).join("release").resolve().to_string()
}

async fn run_app() -> Int {
  let args = @tool.cli_args()
  if args.length() != 3 {
    println(
      "Usage: moon run --target native tools/moon/cmd/github/create_cli_archive -- <target> <archive> <executable>",
    )
    return 1
  }
  let cwd = release_dir(args[0])
  let archive_path = @tool.resolve_path(args[1])
  if args[1].has_suffix(".zip") {
    return @process.run("7z", ["a", archive_path, args[2]], cwd=cwd)
  }
  @process.run("tar", ["-czvf", archive_path, args[2]], cwd=cwd)
}

async fn main {
  @sys.exit(run_app())
}
