///|
/// A release-task request is consumed once and never inherited by child tools.
/// Direct Moon invocations retain their existing initialize-once policy.
struct ReleaseRegistryRefresh {
  required : Bool
  mut complete : Bool
}

///|
fn release_registry_refresh() -> ReleaseRegistryRefresh? {
  let request = @env.get_env_var("VIZE_RELEASE_REGISTRY_REFRESH")
  @env.unset_env_var("VIZE_RELEASE_REGISTRY_REFRESH")
  match request {
    None => Some({ required: false, complete: false })
    Some("1") => Some({ required: true, complete: false })
    _ => None
  }
}

///|
/// Task banners must reach inherited stdout before registry children write.
/// Direct Moon retains the original buffered println behavior.
async fn ReleaseRegistryRefresh::print_line(
  self : ReleaseRegistryRefresh,
  text : String,
) -> Unit {
  if self.required {
    @stdio.stdout.write(text + "\n")
  } else {
    println(text)
  }
}

///|
/// Keep the same registry wrapper, executable, argv and inherited environment.
/// Completion belongs only to this invocation, with no persistent bypass.
async fn ReleaseRegistryRefresh::ensure(self : ReleaseRegistryRefresh) -> Int {
  if !self.required || self.complete {
    return 0
  }
  let moon_bin = match @env.get_env_var("MOON_BIN") {
    Some(value) if value != "" => value
    _ => "moon"
  }
  let random = match @env.rand(16) {
    Some(bytes) => bytes
    None => {
      @stdio.stderr.write("Could not allocate registry completion identity\n")
      return 1
    }
  }
  let mut nonce = ""
  for byte in random {
    nonce += byte.to_int().to_string() + "_"
  }
  let temp_root = @tool.tool_bin(
    "TMPDIR", @tool.tool_bin("TEMP", @tool.tool_bin("TMP", "/tmp")),
  )
  let directory = @tool.join_path(temp_root, "vize-release-registry-" + nonce)
  try @xfs.create_dir(directory) catch {
    _ => {
      @stdio.stderr.write("Could not create registry completion directory\n")
      return 1
    }
  }
  let acknowledgment = @tool.join_path(directory, "status")
  defer {
    try @xfs.remove_file(acknowledgment) catch { _ => () }
    try @xfs.remove_dir(directory) catch { _ => () }
  }
  // The pinned async runtime cannot distinguish killed Unix children. A fresh
  // private acknowledgment and a separate normal-exit sentinel certify that
  // this invocation's unchanged updater completed, including its exact status.
  let status = @process.run("sh", [
    "-c",
    "ack=$1; shift; node \"$@\"; status=$?; printf '%s\\n' \"$status\" > \"$ack\" || exit 1; exit 41",
    "--", acknowledgment,
    "tools/support/release/moon-registry-update.mjs", moon_bin, "update",
  ])
  if status != 41 {
    @stdio.stderr.write("Registry freshness did not complete; no release files changed\n")
    return if status == 0 { 1 } else { status }
  }
  let completion = match @tool.read_file_to_string(acknowledgment) {
    Some(text) => match parse_int(text.trim().to_owned()) {
      Some(value) if value >= 0 && value <= 255 && text == "\{value}\n" => value
      _ => -1
    }
    None => -1
  }
  if completion < 0 {
    @stdio.stderr.write("Registry freshness did not complete; no release files changed\n")
    return 1
  }
  if completion == 0 {
    self.complete = true
  }
  completion
}

///|
/// Invalid CLI inputs retain the task's eager freshness-error precedence.
async fn ReleaseRegistryRefresh::fail(
  self : ReleaseRegistryRefresh,
  message : String,
) -> Int {
  let status = self.ensure()
  if status != 0 {
    return status
  }
  @stdio.stderr.write(message)
  1
}
