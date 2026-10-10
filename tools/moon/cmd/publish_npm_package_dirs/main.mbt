///|
/// A complete result for one package, including its registry visibility wait.
struct PublishResult {
  package_dir : String
  exit_code : Int
  walltime_ms : Int
} derive(ToJson)

///|
/// Keep each launched child's complete decoded UTF8 diagnostic streams.
struct PublishOutput {
  result : PublishResult
  stdout : String
  stderr : String
}

///|
/// Preserve complete decoded child texts beside the original result metadata.
struct PublishDiagnostics {
  package_dir : String
  stdout : String
  stderr : String
} derive(ToJson)

///|
/// Versioned receipt for controlled benchmarks and release diagnostics.
/// Results retain sorted package order even when children finish out of order.
struct PublishReceipt {
  schema_version : Int
  concurrency : Int
  package_count : Int
  walltime_ms : Int
  exit_code : Int
  results : Array[PublishResult]
  skipped : Array[String]
  diagnostics : Array[PublishDiagnostics]
} derive(ToJson)

///|
/// Keep an explicit ceiling on registry pressure, and reject malformed input
/// instead of silently falling back to a different publish mode.
fn parse_concurrency(value : String) -> Int? {
  match value {
    "1" => Some(1)
    "2" => Some(2)
    "3" => Some(3)
    "4" => Some(4)
    _ => None
  }
}

///|
/// Run the existing single-package publisher unchanged: it owns provenance,
/// rerun safety, retries, and exact version + dist-tag visibility checks.
/// Ordinary child failures are results, so every launched child can finish and
/// keep its diagnostics. Cancellation still propagates through collect_output's
/// structured process lifecycle rather than leaving a detached publisher.
async fn publish_package(
  moon_bin : String,
  script_path : String,
  current_dir : String,
  package_dir : String,
  forward_args : Array[String],
) -> PublishOutput {
  let started = @env.now()
  let child_args = [
    "run", "-q", "--target", "native", script_path, "--", package_dir,
  ]
  for arg in forward_args {
    child_args.push(arg)
  }
  println("Publishing \{package_dir}...")
  let (exit_code, stdout_text, stderr_text) = try {
    let (exit_code, stdout, stderr) = @process.collect_output(
      moon_bin,
      child_args,
      cwd=current_dir,
    )
    (exit_code, stdout.text(), stderr.text())
  } catch {
    err if @async.is_being_cancelled() => raise err
    err => (1, "", "Failed to run publisher for \{package_dir}: \{err}")
  }
  {
    result: {
      package_dir,
      exit_code,
      walltime_ms: (@env.now() - started).to_int(),
    },
    stdout: stdout_text,
    stderr: stderr_text,
  }
}

///|
/// Main entry point for publishing generated npm package directories.
/// Serial execution is the compatible default; release opts into bounded
/// concurrency. A failed child stops queued launches while active children are
/// reaped with complete diagnostics. Any failure prevents the wrapper publish.
async fn run_app() -> Int {
  let args = @tool.cli_args()
  if args.is_empty() {
    @stdio.stderr.write(
      "Usage: moon run --target native tools/moon/cmd/publish_npm_package_dirs -- <dir> [--concurrency 1..4] [--receipt path] [--provenance] [--ignore-failure] [--retries N] [--delay N]\n",
    )
    return 1
  }

  let base_directory = @tool.resolve_path(args[0])
  let forward_args = []
  let mut concurrency = 1
  let mut receipt_path = None
  let mut index = 1
  while index < args.length() {
    let arg = args[index]
    if arg == "--concurrency" {
      let value = if index + 1 < args.length() { args[index + 1] } else { "" }
      concurrency = match parse_concurrency(value) {
        Some(value) => value
        None => {
          @stdio.stderr.write("--concurrency requires an integer from 1 to 4\n")
          return 1
        }
      }
      index += 2
      continue
    }
    if arg == "--receipt" {
      if index + 1 >= args.length() ||
        args[index + 1] == "" ||
        args[index + 1].has_prefix("--") {
        @stdio.stderr.write("--receipt requires a file path\n")
        return 1
      }
      receipt_path = Some(@tool.resolve_path(args[index + 1]))
      index += 2
      continue
    }
    forward_args.push(arg)
    index += 1
  }

  let current_dir = @tool.resolve_path(@env.current_dir().unwrap_or("."))
  let moon_bin = @tool.tool_bin("MOON_BIN", "moon")
  let script_path = @tool.join_path(
    current_dir, "tools/moon/cmd/publish_npm_package",
  )
  let package_directories = []
  for entry_name in @tool.read_dir(base_directory) {
    let package_dir = @tool.join_path(base_directory, entry_name)
    if @tool.is_dir(package_dir) &&
      @tool.path_exists(@tool.join_path(package_dir, "package.json")) {
      package_directories.push(package_dir)
    }
  }
  package_directories.sort_by((a, b) => a.lexical_compare(b))
  if package_directories.is_empty() {
    @stdio.stderr.write(
      "No publishable package directories were found in \{base_directory}\n",
    )
    return 1
  }

  let started = @env.now()
  let mut stop_launching = false
  let tasks : Array[async () -> PublishOutput?] = package_directories.map(package_dir => {
    () => {
      // The semaphore admits this body only when a slot is available. No await
      // separates the stop check from launch or the failure from its stop flag.
      if stop_launching {
        return None
      }
      let output = publish_package(
        moon_bin, script_path, current_dir, package_dir, forward_args,
      )
      if output.result.exit_code != 0 && !stop_launching {
        stop_launching = true
        // Only the first failure writes this progress marker. Child streams
        // remain buffered until collection, so async stderr has one writer.
        @stdio.stderr.write(
          "Stopping queued package launches after publisher failure: \{package_dir}\n",
        )
      }
      Some(output)
    }
  })
  // Structured collection reaps active publishers without launching queued
  // packages after an observed failure, and retains original package order.
  let outputs = @async.all(tasks, max_concurrent=concurrency)
  let walltime_ms = (@env.now() - started).to_int()
  let results = []
  let skipped = []
  let diagnostics = []
  let mut output_index = 0
  let mut exit_code = 0
  for item in outputs {
    let package_dir = package_directories[output_index]
    output_index += 1
    let output = match item {
      Some(output) => output
      None => {
        skipped.push(package_dir)
        println(
          "Skipped queued package after publisher failure: \{package_dir}",
        )
        continue
      }
    }
    let result = output.result
    results.push(result)
    diagnostics.push({
      package_dir,
      stdout: output.stdout,
      stderr: output.stderr,
    })
    println(
      "Publish result for \{result.package_dir}: exit \{result.exit_code}",
    )
    if output.stdout != "" {
      @stdio.stdout.write(output.stdout)
    }
    if output.stderr != "" {
      @stdio.stderr.write("Publish diagnostics for \{result.package_dir}:\n")
      @stdio.stderr.write(output.stderr)
    }
    if exit_code == 0 && result.exit_code != 0 {
      exit_code = if result.exit_code > 0 { result.exit_code } else { 1 }
    }
  }
  let receipt = PublishReceipt::{
    schema_version: 1,
    concurrency,
    package_count: package_directories.length(),
    walltime_ms,
    exit_code,
    results,
    skipped,
    diagnostics,
  }
  println(
    "Platform package publish summary: \{receipt.package_count} packages, concurrency \{concurrency}, \{receipt.walltime_ms} ms, exit \{exit_code}",
  )
  if receipt_path is Some(path) {
    @xfs.write_string_to_file(path, receipt.to_json().stringify() + "\n") catch {
      @xfs.IOError(message) => {
        @stdio.stderr.write(
          "Failed to write publish receipt to \{path}: \{message}\n",
        )
        return if exit_code != 0 { exit_code } else { 1 }
      }
    }
  }
  exit_code
}

///|
async fn main {
  @sys.exit(run_app())
}
