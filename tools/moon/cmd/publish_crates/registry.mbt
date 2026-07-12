/// Result of querying crates.io for one exact crate version.
enum CrateVersionState {
  Published
  Missing
  QueryFailed(String)
}

/// Parses the crates.io response without turning protocol failures into a
/// false "not published" result.
fn published_version_state(value : Json, crate_name : String, version : String) -> CrateVersionState {
  guard value is Object(root) else {
    return QueryFailed("crates.io returned a non-object response for \{crate_name}@\{version}")
  }
  guard root.get("version") is Some(Object(version_obj)) else {
    return QueryFailed("crates.io response for \{crate_name}@\{version} has no version object")
  }
  guard version_obj.get("crate") is Some(String(returned_crate)) else {
    return QueryFailed("crates.io returned a version without crate for \{crate_name}@\{version}")
  }
  guard version_obj.get("num") is Some(String(returned_version)) else {
    return QueryFailed("crates.io returned a version without num for \{crate_name}@\{version}")
  }
  if returned_crate != crate_name || returned_version != version {
    return QueryFailed("crates.io returned mismatched identity for \{crate_name}@\{version}")
  }
  Published
}

/// Queries crates.io for an exact version. Dry-run callers consume the full
/// state so transport and schema failures stop the release preflight.
async fn query_crate_version(crate_name : String, version : String) -> CrateVersionState {
  let (exit_code, body, stderr) = @process.collect_output(
    "curl",
    [
      "-sS",
      "--fail-with-body",
      "--connect-timeout",
      "5",
      "--max-time",
      "15",
      "--retry",
      "2",
      "--retry-delay",
      "1",
      "--retry-connrefused",
      "--write-out",
      "VIZE_HTTP_STATUS:%{http_code}",
      "-H",
      "User-Agent: vize-release-publish (https://github.com/ubugeeei-prod/vize)",
      "https://crates.io/api/v1/crates/" + crate_name + "/" + version,
    ],
  )
  let response_parts : Array[String] = body
    .text()
    .split("VIZE_HTTP_STATUS:")
    .map(view => view.to_owned())
    .collect()
  if response_parts.length() == 2 && response_parts[1] == "404" {
    return Missing
  }
  if exit_code != 0 {
    let detail = stderr.text().trim().to_owned()
    let suffix = if detail == "" { "" } else { ": " + detail }
    if response_parts.length() == 2 && response_parts[1] != "000" {
      return QueryFailed(
        "crates.io query for \{crate_name}@\{version} returned HTTP \{response_parts[1]} (curl exit \{exit_code})\{suffix}",
      )
    }
    return QueryFailed(
      "crates.io query for \{crate_name}@\{version} failed with curl exit \{exit_code}\{suffix}",
    )
  }
  if response_parts.length() != 2 {
    return QueryFailed(
      "crates.io query for \{crate_name}@\{version} returned no HTTP status",
    )
  }
  let status = response_parts[1]
  if status != "200" {
    return QueryFailed(
      "crates.io query for \{crate_name}@\{version} returned HTTP \{status}",
    )
  }
  let parsed = try {
    @json.parse(response_parts[0])
  } catch {
    _ =>
      return QueryFailed(
        "crates.io returned invalid JSON for \{crate_name}@\{version}",
      )
  }
  published_version_state(parsed, crate_name, version)
}

/// Actual publication preserves its conservative fallback: an inconclusive
/// API query is treated as unpublished and delegated to cargo's idempotent
/// publish/re-resolution path.
async fn crate_is_published(crate_name : String, version : String) -> Bool {
  match query_crate_version(crate_name, version) {
    Published => true
    Missing | QueryFailed(_) => false
  }
}

/// Returns an actionable error when Cargo cannot resolve an allegedly
/// published version from the registry index.
async fn crate_resolution_error(crate_name : String, version : String) -> String? {
  let (exit_code, stdout, stderr) = @process.collect_output(
    "cargo",
    ["info", "--registry", "crates-io", crate_name + "@" + version],
  )
  if exit_code == 0 {
    return None
  }
  let detail = [stdout.text().trim().to_owned(), stderr.text().trim().to_owned()]
    .filter(text => text != "")
    .join("\n")
  let suffix = if detail == "" { "" } else { "\n" + detail }
  Some(
    "Cargo could not resolve published prerequisite \{crate_name}@\{version} (exit \{exit_code})\{suffix}",
  )
}

async fn crate_is_resolvable(crate_name : String, version : String) -> Bool {
  match crate_resolution_error(crate_name, version) {
    None => true
    Some(_) => false
  }
}

/// Waits until `cargo info crate@version` succeeds or a timeout is reached.
async fn wait_for_resolution(
  crate_name : String,
  version : String,
  max_attempts : Int,
  delay_seconds : Int,
) -> Int {
  let mut attempt = 1
  while attempt <= max_attempts {
    if crate_is_resolvable(crate_name, version) {
      println("\{crate_name} v\{version} is resolvable from crates.io.")
      return 0
    }
    if attempt >= max_attempts {
      @stdio.stderr.write(
        "Timed out waiting for \{crate_name} v\{version} to become resolvable from crates.io.\n",
      )
      return 1
    }
    println(
      "Waiting for \{crate_name} v\{version} to become resolvable from crates.io... (\{attempt}/\{max_attempts})",
    )
    @async.sleep(delay_seconds * 1000)
    attempt = attempt + 1
  }
  1
}
