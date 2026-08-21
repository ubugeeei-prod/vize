/// Rewrites every native binary catalog pin under the `native-binaries:` block
/// of `pnpm-workspace.yaml` to the new release version. It also keeps pnpm's
/// minimum-release-age allowlist aligned with those newly-published native
/// package versions.
/// The catalog block is recognised by the `  native-binaries:` header (two-space
/// indent) and runs until the next top-level or catalog-section header. Inside
/// that block the function replaces the version literal on every
/// `    "@vizejs/native-...": "<version>"` line. The block header itself plus
/// any comments are preserved untouched.
fn replace_native_binary_workspace_catalog(
  content : String,
  current_version : String,
  new_version : String,
) -> String {
  let updated : Array[String] = []
  let mut in_catalog = false
  let lines : Array[String] = content.split("\n").map(view => view.to_owned()).collect()
  let header = "  native-binaries:"
  let pin_prefix = "    \"@vizejs/native-"
  let minimum_age_prefix = "  - \"@vizejs/native-"
  for line in lines {
    if line.has_prefix(minimum_age_prefix) && line.contains(current_version) {
      updated.push(line.replace(old=current_version, new=new_version))
      continue
    }
    if line == header {
      in_catalog = true
      updated.push(line)
      continue
    }
    if in_catalog {
      let leaves_block = line != "" &&
        !line.has_prefix("    ") &&
        !line.has_prefix("  #")
      if leaves_block {
        in_catalog = false
      } else if line.has_prefix(pin_prefix) && line.contains(current_version) {
        updated.push(line.replace(old=current_version, new=new_version))
        continue
      }
    }
    updated.push(line)
  }
  updated.join("\n")
}

/// Rewrites every native binary catalog pin under the `native-binaries:` block
/// inside `pnpm-lock.yaml`.
/// The catalogs section of pnpm-lock.yaml mirrors `pnpm-workspace.yaml` and each
/// native package appears as a two-line entry:
/// ```
///     '@vizejs/native-darwin-arm64':
///       specifier: 0.107.0
///       version: 0.107.0
/// ```
/// We walk the file, detect entry into the `native-binaries:` block (under
/// `catalogs:` at the four-space indent), and rewrite the `specifier:` /
/// `version:` lines for every native package. The block ends when we either hit
/// a top-level YAML key or another catalog header at the same indent.
fn replace_native_binary_lockfile_catalog(
  content : String,
  current_version : String,
  new_version : String,
) -> String {
  let updated : Array[String] = []
  let mut in_block = false
  let lines : Array[String] = content.split("\n").map(view => view.to_owned()).collect()
  let header = "  native-binaries:"
  for line in lines {
    if line == header {
      in_block = true
      updated.push(line)
      continue
    }
    if in_block {
      let leaves_block = line != "" && !line.has_prefix("    ")
      if leaves_block {
        in_block = false
      } else if (
        line.has_prefix("      specifier: ") || line.has_prefix("      version: ")
      ) &&
        line.contains(current_version) {
        updated.push(line.replace(old=current_version, new=new_version))
        continue
      }
    }
    updated.push(line)
  }
  updated.join("\n")
}
