///|
/// Publishes the VS Code extension to Open VSX in a rerun-safe way.
/// Release retries are common when a tag is force-pushed to re-run a partially
/// completed workflow. This script mirrors the VS Code Marketplace publish
/// guard by checking Open VSX metadata before publishing and after any non-zero
/// publish exit.
struct ExtensionInfo {
  extension_id : String
  publisher : String
  version : String
}

///|
fn parse_int(value : String) -> Int? {
  Some(@string.parse_int(value)) catch {
    _ => None
  }
}

///|
fn int_at_least_or(value : String, minimum : Int, fallback : Int) -> Int {
  match parse_int(value) {
    Some(parsed) if parsed >= minimum => parsed
    _ => fallback
  }
}

///|
fn env_int_at_least(name : String, minimum : Int, fallback : Int) -> Int {
  match @sys.get_env_var(name) {
    Some(value) if value != "" => int_at_least_or(value, minimum, fallback)
    _ => fallback
  }
}

///|
fn read_package_json(path : String) -> Json? {
  Some(@json.parse(@xfs.read_file_to_string(path))) catch {
    _ => None
  }
}

///|
fn extension_info_from_package_json(value : Json) -> ExtensionInfo? {
  guard value is Object(obj) else { return None }
  guard obj.get("publisher") is Some(String(publisher)) else { return None }
  guard obj.get("name") is Some(String(name)) else { return None }
  guard obj.get("version") is Some(String(version)) else { return None }
  guard publisher != "" && name != "" && version != "" else { return None }
  Some(ExtensionInfo::{
    extension_id: publisher + "." + name,
    publisher,
    version,
  })
}

///|
fn json_has_extension_version(value : Json, expected : String) -> Bool {
  guard value is Object(obj) else { return false }
  if obj.get("version") is Some(String(version)) && version == expected {
    return true
  }
  guard obj.get("versions") is Some(Array(versions)) else { return false }
  for item in versions {
    guard item is Object(version_obj) else { continue }
    if version_obj.get("version") is Some(String(version)) &&
      version == expected {
      return true
    }
  }
  false
}

///|
async fn extension_version_is_visible(
  vp_bin : String,
  extension_id : String,
  version : String,
) -> Bool {
  let (exit_code, stdout, _) = @process.collect_output(vp_bin, [
    "dlx", "-p", "ovsx@^1.0.0", "ovsx", "get", extension_id, "--metadata", "--versionRange",
    version,
  ])
  if exit_code != 0 {
    return false
  }
  let body = stdout.text().trim()
  if body == "" {
    return false
  }
  let parsed = @json.parse(body) catch { _ => return false }
  json_has_extension_version(parsed, version)
}

///|
async fn wait_for_extension_visibility(
  vp_bin : String,
  extension_id : String,
  version : String,
  max_attempts : Int,
  delay_seconds : Int,
) -> Bool {
  let mut attempt = 1
  while attempt <= max_attempts {
    if extension_version_is_visible(vp_bin, extension_id, version) {
      println("\{extension_id} \{version} is visible in Open VSX.")
      return true
    }
    if attempt >= max_attempts {
      return false
    }
    println(
      "Waiting for \{extension_id} \{version} to appear in Open VSX... (\{attempt}/\{max_attempts})",
    )
    @async.sleep(delay_seconds * 1000)
    attempt = attempt + 1
  }
  false
}

///|
async fn ensure_namespace(vp_bin : String, publisher : String) -> Bool {
  let (exit_code, stdout, stderr) = @process.collect_output(vp_bin, [
    "dlx", "-p", "ovsx@^1.0.0", "ovsx", "create-namespace", publisher,
  ])
  if exit_code == 0 {
    return true
  }
  let detail = [
      stdout.text().trim().to_owned(),
      stderr.text().trim().to_owned(),
    ]
    .filter(text => text != "")
    .join("\n")
  if detail.to_lower().contains("already exists") {
    println(
      "Open VSX namespace \{publisher} already exists; continuing to publish.",
    )
    return true
  }
  let suffix = if detail == "" { "" } else { "\n" + detail }
  @stdio.stderr.write(
    "Failed to ensure Open VSX namespace \{publisher} (exit \{exit_code}).\{suffix}\n",
  )
  false
}

///|
async fn run_app() -> Int {
  let args = @tool.cli_args()
  if args.is_empty() || args.length() > 2 {
    println(
      "Usage: moon run --target native tools/moon/cmd/publish_open_vsx_extension -- <vsix-path> [package-json-path]",
    )
    return 1
  }
  let package_json_path = if args.length() == 2 {
    @tool.resolve_path(args[1])
  } else {
    @tool.resolve_path("editors/vscode/package.json")
  }
  let extension_info = match read_package_json(package_json_path) {
    Some(value) =>
      match extension_info_from_package_json(value) {
        Some(info) => info
        None => {
          @stdio.stderr.write(
            "Failed to read extension metadata from \{package_json_path}\n",
          )
          return 1
        }
      }
    None => {
      @stdio.stderr.write("Failed to read \{package_json_path}\n")
      return 1
    }
  }
  let vp_bin = match @sys.get_env_var("VP_BIN") {
    Some(value) if value != "" => value
    _ => "vp"
  }
  let resolution_attempts = env_int_at_least(
    "PUBLISH_RESOLUTION_RETRY_LIMIT", 1, 24,
  )
  let resolution_delay_seconds = env_int_at_least(
    "PUBLISH_RESOLUTION_RETRY_DELAY", 0, 10,
  )
  if extension_version_is_visible(
      vp_bin,
      extension_info.extension_id,
      extension_info.version,
    ) {
    println(
      "\{extension_info.extension_id} \{extension_info.version} is already published, skipping Open VSX publish.",
    )
    return 0
  }
  if !ensure_namespace(vp_bin, extension_info.publisher) {
    return 1
  }

  let exit_code = @process.run(vp_bin, [
    "dlx",
    "-p",
    "ovsx@^1.0.0",
    "ovsx",
    "publish",
    @tool.resolve_path(args[0]),
  ])
  if wait_for_extension_visibility(
      vp_bin,
      extension_info.extension_id,
      extension_info.version,
      resolution_attempts,
      resolution_delay_seconds,
    ) {
    if exit_code != 0 {
      println(
        "\{extension_info.extension_id} \{extension_info.version} was published despite a non-zero ovsx publish exit.",
      )
    }
    return 0
  }
  if exit_code == 0 {
    @stdio.stderr.write(
      "Open VSX did not expose \{extension_info.extension_id} \{extension_info.version} before the visibility deadline.\n",
    )
    return 1
  }
  exit_code
}

///|
async fn main {
  @sys.exit(run_app())
}
