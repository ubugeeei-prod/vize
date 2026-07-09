
fn bench_script(task : String) -> String? {
  match task {
    "run" => Some("bench/run.ts")
    "generate" => Some("bench/generate.mjs")
    "lint" => Some("bench/lint.ts")
    "fmt" => Some("bench/fmt.ts")
    "check" => Some("bench/check.ts")
    "vite" => Some("bench/vite.ts")
    "compare-tools" => Some("bench/compare-tools.mjs")
    _ => None
  }
}

async fn run_app() -> Int {
  let args = @tool.cli_args()
  if args.is_empty() {
    println(
      "Usage: moon run --target native tools/moon/cmd/bench -- <run|generate|lint|fmt|check|vite|compare-tools> [args...]",
    )
    return 1
  }

  match bench_script(args[0]) {
    Some(script) => {
      let node_args = [script]
      for i in 1..<args.length() {
        node_args.push(args[i])
      }
      @process.run("node", node_args)
    }
    None => {
      println("Unknown benchmark task: \{args[0]}")
      1
    }
  }
}

async fn main {
  @sys.exit(run_app())
}
