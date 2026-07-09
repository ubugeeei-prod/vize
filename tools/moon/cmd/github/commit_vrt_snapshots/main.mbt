
async fn run_app() -> Int {
  let args = @tool.cli_args()
  if args.length() != 1 {
    println(
      "Usage: moon run --target native tools/moon/cmd/github/commit_vrt_snapshots -- <branch>",
    )
    return 1
  }
  let branch = args[0]
  if @process.run("git", ["config", "user.name", "github-actions[bot]"]) != 0 {
    return 1
  }
  if @process.run(
    "git",
    ["config", "user.email", "github-actions[bot]@users.noreply.github.com"],
  ) != 0 {
    return 1
  }
  if @process.run("git", ["add", "playground/e2e/vrt/__snapshots__/"]) != 0 {
    return 1
  }
  let diff_exit = @process.run("git", ["diff", "--cached", "--quiet"])
  if diff_exit == 0 {
    println("No snapshot changes")
    return 0
  }
  if diff_exit != 1 {
    return diff_exit
  }
  if @process.run("git", ["commit", "-m", "chore: update VRT snapshots"]) != 0 {
    return 1
  }
  @process.run("git", ["push", "origin", "HEAD:" + branch])
}

async fn main {
  @sys.exit(run_app())
}
