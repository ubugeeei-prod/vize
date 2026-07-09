
async fn run_app() -> Int {
  let args = @tool.cli_args()
  if args.length() > 1 {
    println("Usage: moon run --target native tools/moon/cmd/dev_app -- [target]")
    return 1
  }
  let extra_env : Map[String, String] = {}
  if args.length() == 1 {
    extra_env["usage_target"] = args[0]
  }
  @process.run(
    "node",
    ["tests/tooling/support/dev-app.ts"],
    extra_env=extra_env,
  )
}

async fn main {
  @sys.exit(run_app())
}
