#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

use std::{
    env, fs, fs::OpenOptions, io::Write, os::unix::fs::PermissionsExt, path::Path,
    process::ExitCode,
};

fn main() -> ExitCode {
    match configure() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}

fn configure() -> Result<(), String> {
    if let Some(extra) = env::args_os().nth(1) {
        return Err(format!(
            "Usage: rust-script tools/commands/ci/github/configure-zig-musl-linkers.rs; unexpected argument {}",
            extra.to_string_lossy()
        ));
    }

    let runner_temp = env::var("RUNNER_TEMP").map_err(|_| "RUNNER_TEMP is required".to_string())?;
    let github_env = env::var("GITHUB_ENV").map_err(|_| "GITHUB_ENV is required".to_string())?;
    let linker_dir = Path::new(&runner_temp).join("vize-zig-linkers");
    fs::create_dir_all(&linker_dir)
        .map_err(|error| format!("failed to create {}: {error}", linker_dir.display()))?;

    let zig_ar = linker_dir.join("zig-ar");
    write_executable(&zig_ar, "#!/usr/bin/env bash\nexec zig ar \"$@\"\n")?;

    let mut env_file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(&github_env)
        .map_err(|error| format!("failed to open {github_env}: {error}"))?;
    write_cc(
        &linker_dir,
        &zig_ar,
        &mut env_file,
        "X86_64_UNKNOWN_LINUX_MUSL",
        "x86_64-linux-musl",
    )?;
    write_cc(
        &linker_dir,
        &zig_ar,
        &mut env_file,
        "AARCH64_UNKNOWN_LINUX_MUSL",
        "aarch64-linux-musl",
    )
}

fn write_cc(
    linker_dir: &Path,
    zig_ar: &Path,
    env_file: &mut fs::File,
    rust_target: &str,
    zig_target: &str,
) -> Result<(), String> {
    let cc = linker_dir.join(format!("zig-cc-{zig_target}"));
    let script = format!(
        r#"#!/usr/bin/env bash
set -euo pipefail
args=()
skip_next=0
for arg in "$@"; do
  if (( skip_next )); then
    skip_next=0
    continue
  fi
  case "$arg" in
    --target=*)
      ;;
    --target)
      skip_next=1
      ;;
    *)
      args+=("$arg")
      ;;
  esac
done
exec zig cc -target {zig_target} "${{args[@]}}"
"#
    );
    write_executable(&cc, &script)?;

    writeln!(env_file, "CARGO_TARGET_{rust_target}_LINKER=rust-lld")
        .map_err(|error| format!("failed to write GitHub env: {error}"))?;
    writeln!(
        env_file,
        "CC_{}={}",
        rust_target.to_lowercase(),
        cc.display()
    )
    .map_err(|error| format!("failed to write GitHub env: {error}"))?;
    writeln!(
        env_file,
        "AR_{}={}",
        rust_target.to_lowercase(),
        zig_ar.display()
    )
    .map_err(|error| format!("failed to write GitHub env: {error}"))
}

fn write_executable(path: &Path, contents: &str) -> Result<(), String> {
    fs::write(path, contents)
        .map_err(|error| format!("failed to write {}: {error}", path.display()))?;
    let mut permissions = fs::metadata(path)
        .map_err(|error| format!("failed to stat {}: {error}", path.display()))?
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)
        .map_err(|error| format!("failed to chmod {}: {error}", path.display()))
}
