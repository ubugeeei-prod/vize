
fn bench_script(task : String) -> String? {
  match task {
    "run" => Some("tools/benchmarks/scripts/run.ts")
    "generate" => Some("tools/benchmarks/scripts/generate.mjs")
    "lint" => Some("tools/benchmarks/scripts/lint.ts")
    "fmt" => Some("tools/benchmarks/scripts/fmt.ts")
    "check" => Some("tools/benchmarks/scripts/check.ts")
    "vite" => Some("tools/benchmarks/scripts/vite.ts")
    "musea" => Some("tools/benchmarks/scripts/musea.mjs")
    "compare-tools" => Some("tools/benchmarks/scripts/compare-tools.mjs")
    _ => None
  }
}

async fn run_app() -> Int {
  let args = @tool.cli_args()
  if args.is_empty() {
    println(
      "Usage: moon run --target native tools/moon/cmd/bench -- <run|generate|lint|fmt|check|vite|musea|compare-tools> [args...]",
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
