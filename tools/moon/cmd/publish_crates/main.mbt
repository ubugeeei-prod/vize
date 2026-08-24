/// Publishes workspace crates to crates.io with retry and propagation checks.
/// The release workflow uses this script instead of inline shell so that the
/// publication order, retry policy, and post-publish verification all live in a
/// single place. The script is deliberately idempotent: if a crate version is
/// already published, it skips the publish call and only waits until the crate
/// becomes resolvable from crates.io index queries.

/// Ordered list of publishable crates.
/// The order matters because many of these crates depend on previously
/// published workspace crates via exact-version dependencies. The accompanying
/// test asserts that this list remains topologically valid.
/// Crates in `manual_publish_crates` are intentionally omitted here:
/// crates.io Trusted Publishing can publish only crates whose trusted-publisher
/// handoff is configured for this repository/workflow/environment. First
/// publishes and crates whose handoff has not been completed yet must be
/// published manually once before this job can own subsequent versions.
let published_crates : Array[String] = [
  "vize_carton",
  "vize_relief",
  "vize_armature",
  "vize_croquis",
  "vize_atelier_core",
  "vize_atelier_dom",
  "vize_atelier_vapor",
  "vize_atelier_ssr",
  "vize_atelier_sfc",
  "vize_musea",
  "vize_fresco",
]

/// Publishable crates that need manual owner-token work before GitHub Trusted
/// Publishing can publish them from the release workflow.
let manual_publish_crates : Array[String] = [
  "vize_croquis_cf",
  "vize_atelier_jsx",
  "vize_marquette",
  "vize_doctor",
]

/// Existing crates whose current version depends on a crate that still needs a
/// manual publish handoff.
let blocked_by_manual_publish_crates : Array[String] = [
  "vize_canon",
  "vize_patina",
]

fn includes_string(items : Array[String], value : String) -> Bool {
  for item in items {
    if item == value {
      return true
    }
  }
  false
}

fn known_release_crates() -> Array[String] {
  let crates : Array[String] = []
  for crate_name in published_crates {
    crates.push(crate_name)
  }
  for crate_name in manual_publish_crates {
    crates.push(crate_name)
  }
  for crate_name in blocked_by_manual_publish_crates {
    crates.push(crate_name)
  }
  crates
}

/// Parses a positive integer from a string and falls back on invalid input.
fn positive_int_or(value : String, fallback : Int) -> Int {
  try {
    let parsed = @string.parse_int(value)
    if parsed > 0 {
      parsed
    } else {
      fallback
    }
  } catch {
    _ => fallback
  }
}

/// Reads a retry-related integer from the environment with a validated default.
fn env_positive_int(name : String, fallback : Int) -> Int {
  match @env.get_env_var(name) {
    Some(value) if value != "" => positive_int_or(value, fallback)
    _ => fallback
  }
}

/// Extracts the workspace package version from the root `Cargo.toml`.
/// We only need the `[workspace.package]` version line, so a lightweight line
/// scanner is more predictable than shelling out to another TOML parser.
fn workspace_version_from_content(content : String) -> String? {
  let mut section = ""
  let lines : Array[String] = content.split("\n").map(view => view.to_owned()).collect()
  for line in lines {
    let trimmed = line.trim()
    if trimmed.has_prefix("[") && trimmed.has_suffix("]") {
      section = trimmed.to_owned()
      continue
    }
    if section == "[workspace.package]" &&
      trimmed.has_prefix("version = \"") &&
      trimmed.has_suffix("\"") {
      return Some(
        trimmed
          .to_owned()
          .replace(old="version = \"", new="")
          .replace_all(old="\"", new=""),
      )
    }
  }
  None
}

/// Main entry point for ordered and rerun-safe crates.io publishing.
/// Each crate is published serially, retried on transient failure, and then
/// verified through Cargo resolution before the next dependent crate is
/// attempted. This avoids the common race where a dependent crate publishes
/// before crates.io index propagation catches up.
async fn run_app() -> Int {
  let args = @tool.cli_args()
  let mut dry_run = false
  let selected_crates : Array[String] = []
  let usage = "Usage: moon run --target native tools/moon/cmd/publish_crates -- [--dry-run] [--crate <crate>...]\n"
  let mut index = 0
  while index < args.length() {
    match args[index] {
      "--dry-run" => {
        if dry_run {
          @stdio.stderr.write(usage)
          return 1
        }
        dry_run = true
        index = index + 1
      }
      "--crate" => {
        guard index + 1 < args.length() else {
          @stdio.stderr.write(usage)
          return 1
        }
        selected_crates.push(args[index + 1])
        index = index + 2
      }
      _ => {
        @stdio.stderr.write(usage)
        return 1
      }
    }
  }

  let publish_targets : Array[String] = []
  if selected_crates.is_empty() {
    for crate_name in published_crates {
      publish_targets.push(crate_name)
    }
  } else {
    let known_crates = known_release_crates()
    for crate_name in selected_crates {
      if !includes_string(known_crates, crate_name) {
        @stdio.stderr.write("Unknown release crate: \{crate_name}\n" + usage)
        return 1
      }
      if includes_string(publish_targets, crate_name) {
        @stdio.stderr.write("Duplicate release crate: \{crate_name}\n" + usage)
        return 1
      }
      publish_targets.push(crate_name)
    }
  }

  let cargo_toml = match @tool.read_file_to_string("Cargo.toml") {
    Some(content) => content
    None => {
      @stdio.stderr.write("Failed to read Cargo.toml\n")
      return 1
    }
  }
  let version = match workspace_version_from_content(cargo_toml) {
    Some(version) => version
    None => {
      @stdio.stderr.write("Failed to read current version from Cargo.toml\n")
      return 1
    }
  }

  let max_attempts = env_positive_int("PUBLISH_RETRY_LIMIT", 60)
  let delay_seconds = env_positive_int("PUBLISH_RETRY_DELAY", 5)
  let resolution_attempts = env_positive_int("PUBLISH_RESOLUTION_RETRY_LIMIT", max_attempts)
  let resolution_delay_seconds = env_positive_int(
    "PUBLISH_RESOLUTION_RETRY_DELAY",
    delay_seconds,
  )
  println("Expected version: \{version}")
  if !manual_publish_crates.is_empty() {
    println(
      "Deferred manual-publish crates: " + manual_publish_crates.join(", "),
    )
  }
  if !blocked_by_manual_publish_crates.is_empty() {
    println(
      "Deferred dependent crates: " + blocked_by_manual_publish_crates.join(", "),
    )
  }
  if !selected_crates.is_empty() {
    println("Selected crate publish plan: " + publish_targets.join(", "))
  }

  if dry_run {
    let package_args = ["package", "--locked", "--no-verify"]
    for crate_name in publish_targets {
      package_args.push("-p")
      package_args.push(crate_name)
    }
    let plan_label = if selected_crates.is_empty() { "complete supported" } else { "selected" }
    println("Packaging the \{plan_label} crate plan for v\{version}...")
    if @process.run("cargo", package_args) != 0 {
      @stdio.stderr.write("Crate package validation failed for v\{version}.\n")
      return 1
    }

    let mut frontier : String? = None
    for crate_name in publish_targets {
      match query_crate_version(crate_name, version) {
        QueryFailed(message) => {
          @stdio.stderr.write("Release registry check failed: " + message + "\n")
          return 1
        }
        Missing => {
          frontier = Some(crate_name)
          break
        }
        Published =>
          match crate_resolution_error(crate_name, version) {
            Some(message) => {
              @stdio.stderr.write("Release registry check failed: " + message + "\n")
              return 1
            }
            None =>
              println(
                "\{crate_name} v\{version} is already published and resolvable.",
              )
          }
      }
    }
    let frontier = match frontier {
      Some(frontier) => frontier
      None => {
        println(
          "Every crate in the v\{version} publish plan is already published and resolvable.",
        )
        return 0
      }
    }
    println("Dry-running registry-resolvable frontier \{frontier} v\{version}...")
    if @process.run(
      "cargo",
      ["publish", "--dry-run", "--locked", "-p", frontier],
    ) != 0 {
      @stdio.stderr.write("Crate publish dry-run failed for \{frontier} v\{version}.\n")
      return 1
    }
    println(
      "Package validation covered all supported crates; registry dry-run covered \{frontier}.",
    )
    return 0
  }

  for crate_name in publish_targets {
    println("Publishing \{crate_name} v\{version}...")
    if crate_is_published(crate_name, version) {
      println("\{crate_name} v\{version} is already published, skipping publish.")
      if wait_for_resolution(
        crate_name,
        version,
        resolution_attempts,
        resolution_delay_seconds,
      ) != 0 {
        return 1
      }
      continue
    }

    let mut attempt = 1
    let mut published = false
    while attempt <= max_attempts {
      let (publish_exit, publish_stdout, publish_stderr) = @process.collect_output(
        "cargo",
        ["publish", "--locked", "-p", crate_name],
      )
      if publish_exit == 0 {
        published = true
        break
      }
      let stdout_text = publish_stdout.text().trim()
      let stderr_text = publish_stderr.text().trim()
      if stdout_text != "" {
        println(stdout_text)
      }
      if stderr_text != "" {
        @stdio.stderr.write(stderr_text + "\n")
      }
      if crate_is_published(crate_name, version) {
        println(
          "\{crate_name} v\{version} was published despite a non-zero cargo publish exit.",
        )
        published = true
        break
      }
      if crate_is_resolvable(crate_name, version) {
        println(
          "\{crate_name} v\{version} is already resolvable despite a non-zero cargo publish exit.",
        )
        published = true
        break
      }
      if attempt >= max_attempts {
        @stdio.stderr.write("Timed out publishing \{crate_name} v\{version}.\n")
        return 1
      }
      println(
        "Retrying cargo publish for \{crate_name} v\{version}... (\{attempt}/\{max_attempts})",
      )
      @async.sleep(delay_seconds * 1000)
      attempt = attempt + 1
    }

    if !published {
      return 1
    }
    println("\{crate_name} v\{version} published successfully.")
    if wait_for_resolution(
      crate_name,
      version,
      resolution_attempts,
      resolution_delay_seconds,
    ) != 0 {
      return 1
    }
  }

  println("Done!")
  0
}

async fn main {
  @sys.exit(run_app())
}
