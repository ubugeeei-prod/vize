///|
/// Publishes the VS Code extension in a rerun-safe way.
/// Release retries are common when a tag is force-pushed to re-run a partially
/// completed workflow. npm packages and crates are already idempotent in this
/// repository, so this script applies the same principle to the VS Code
/// Marketplace publish step:
/// 1. Read the extension id and target version from `package.json`.
/// 2. Query Marketplace metadata through `vsce show --json`.
/// 3. Skip publish entirely when the exact version is already visible.
/// 4. Treat a non-zero `vsce publish` exit as success if the version becomes
///    visible immediately after the attempted publish.
struct ExtensionInfo {
  extension_id : String
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
  match @env.get_env_var(name) {
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
fn configured_vsce_dlx_bin() -> String {
  match @env.get_env_var("VSCE_DLX_BIN") {
    Some(value) if value != "" => value
    _ =>
      match @env.get_env_var("VP_BIN") {
        Some(value) if value != "" => value
        _ => "vp"
      }
  }
}

///|
fn vsce_dlx_args(
  dlx_bin : String,
  command : String,
  extra_args : Array[String],
) -> Array[String] {
  let args = ["dlx"]
  if dlx_bin == "pnpm" {
    args.push("--allow-build")
    args.push("@vscode/vsce-sign")
    args.push("--allow-build")
    args.push("keytar")
    args.push("--package")
  } else {
    args.push("-p")
  }
  args.push("@vscode/vsce@^3.3.2")
  args.push("vsce")
  args.push(command)
  for arg in extra_args {
    args.push(arg)
  }
  args
}

///|
fn extension_info_from_package_json(value : Json) -> ExtensionInfo? {
  guard value is Object(obj) else { return None }
  guard obj.get("publisher") is Some(String(publisher)) else { return None }
  guard obj.get("name") is Some(String(name)) else { return None }
  guard obj.get("version") is Some(String(version)) else { return None }
  guard publisher != "" && name != "" && version != "" else { return None }
  Some(ExtensionInfo::{ extension_id: publisher + "." + name, version })
}

///|
fn json_has_extension_version(value : Json, expected : String) -> Bool {
  guard value is Object(obj) else { return false }
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
  dlx_bin : String,
  extension_id : String,
  version : String,
) -> Bool {
  let (exit_code, stdout, _) = @process.collect_output(
    dlx_bin,
    vsce_dlx_args(dlx_bin, "show", [extension_id, "--json"]),
  )
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
  dlx_bin : String,
  extension_id : String,
  version : String,
  max_attempts : Int,
  delay_seconds : Int,
) -> Bool {
  let mut attempt = 1
  while attempt <= max_attempts {
    if extension_version_is_visible(dlx_bin, extension_id, version) {
      println(
        "\{extension_id} \{version} is visible in the VS Code Marketplace.",
      )
      return true
    }
    if attempt >= max_attempts {
      return false
    }
    println(
      "Waiting for \{extension_id} \{version} to appear in the VS Code Marketplace... (\{attempt}/\{max_attempts})",
    )
    @async.sleep(delay_seconds * 1000)
    attempt = attempt + 1
  }
  false
}

///|
async fn run_app() -> Int {
  let args = @tool.cli_args()
  if args.is_empty() || args.length() > 2 {
    println(
      "Usage: moon run --target native tools/moon/cmd/publish_vscode_extension -- <vsix-path> [package-json-path]",
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
  let dlx_bin = configured_vsce_dlx_bin()
  let resolution_attempts = env_int_at_least(
    "PUBLISH_RESOLUTION_RETRY_LIMIT", 1, 24,
  )
  let resolution_delay_seconds = env_int_at_least(
    "PUBLISH_RESOLUTION_RETRY_DELAY", 0, 10,
  )
  if extension_version_is_visible(
      dlx_bin,
      extension_info.extension_id,
      extension_info.version,
    ) {
    println(
      "\{extension_info.extension_id} \{extension_info.version} is already published, skipping VS Code Marketplace publish.",
    )
    return 0
  }

  let tag = match @env.get_env_var("NPM_TAG") {
    Some(value) if value != "" => value
    _ => "latest"
  }
  let publish_args = [
    "--no-dependencies",
    "--packagePath",
    @tool.resolve_path(args[0]),
  ]
  if tag != "latest" {
    publish_args.push("--pre-release")
  }
  let exit_code = @process.run(dlx_bin, vsce_dlx_args(dlx_bin, "publish", publish_args))
  if wait_for_extension_visibility(
      dlx_bin,
      extension_info.extension_id,
      extension_info.version,
      resolution_attempts,
      resolution_delay_seconds,
    ) {
    if exit_code != 0 {
      println(
        "\{extension_info.extension_id} \{extension_info.version} was published despite a non-zero vsce publish exit.",
      )
    }
    return 0
  }
  if exit_code == 0 {
    @stdio.stderr.write(
      "VS Code Marketplace did not expose \{extension_info.extension_id} \{extension_info.version} before the visibility deadline.\n",
    )
    return 1
  }
  exit_code
}

///|
async fn main {
  @sys.exit(run_app())
}
