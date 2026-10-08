#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//! ```
//! Diagnostic custody around the unchanged original SSR command; never a repair or fallback.

#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
use std::{
    env,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::{Command, ExitCode, ExitStatus},
};

const ORIGINAL: &[&str] = &[
    "cargo",
    "test",
    "-p",
    "vize_atelier_ssr",
    "--features",
    "legacy-differential",
    "--test",
    "davinci_ssr_corpus",
    "--",
    "--nocapture",
];
const TRACE: &[&str] = &[
    "-ff",
    "-qq",
    "-ttt",
    "-yy",
    "-s",
    "8192",
    "-e",
    "trace=execve,clone,clone3,chdir,fchdir,openat,openat2,newfstatat,statx,readlink,readlinkat,read,pread64,close",
    "-e",
    "raw=read,pread64",
];

fn record(directory: &Path, name: &str, bytes: impl AsRef<[u8]>) -> bool {
    match fs::write(directory.join(name), bytes) {
        Ok(()) => true,
        Err(error) => {
            eprintln!("SSR diagnostic capture {name} failed: {error}");
            false
        }
    }
}

fn capture_error(directory: &Path, name: &str, error: impl std::fmt::Display) -> bool {
    eprintln!("SSR diagnostic capture {name} failed: {error}");
    record(directory, name, format!("{error}\n"));
    false
}

fn command(directory: &Path, name: &str, program: &str, args: &[&str]) -> bool {
    match Command::new(program).args(args).output() {
        Ok(output) => {
            let mut complete = output.status.success();
            complete &= record(directory, &format!("{name}.stdout"), output.stdout);
            complete &= record(directory, &format!("{name}.stderr"), output.stderr);
            complete &= record(
                directory,
                &format!("{name}.status"),
                format!("{}\n", output.status),
            );
            complete
        }
        Err(error) => capture_error(directory, &format!("{name}.spawn-error"), error),
    }
}

fn hash(directory: &Path, name: &str, path: &Path) -> bool {
    let Some(path) = path.to_str() else {
        return false;
    };
    command(directory, name, "sha256sum", &[path])
}

fn capture_before(directory: &Path) -> bool {
    let mut complete = true;
    for (name, program, args) in [
        ("git-head", "git", vec!["rev-parse", "HEAD"]),
        ("git-tree", "git", vec!["rev-parse", "HEAD^{tree}"]),
        (
            "git-clean",
            "git",
            vec!["diff", "--quiet", "--ignore-submodules=all", "HEAD", "--"],
        ),
        ("kernel", "uname", vec!["-a"]),
        ("rustc", "rustc", vec!["-vV"]),
        ("cargo", "cargo", vec!["-vV"]),
        ("strace", "strace", vec!["--version"]),
    ] {
        complete &= command(directory, name, program, &args);
    }
    let pid = std::process::id();
    complete &= record(directory, "wrapper-pid.txt", format!("{pid}\n"));
    for name in ["mountinfo", "status", "maps", "limits"] {
        match fs::read(format!("/proc/{pid}/{name}")) {
            Ok(bytes) => complete &= record(directory, &format!("process-{name}"), bytes),
            Err(error) => {
                complete &= capture_error(directory, &format!("process-{name}.error"), error);
            }
        }
    }
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        for (name, path) in [
            ("cwd", env::current_dir()),
            ("exe", env::current_exe()),
            ("root", fs::read_link(format!("/proc/{pid}/root"))),
        ] {
            match path {
                Ok(path) => {
                    complete &= record(
                        directory,
                        &format!("process-{name}.bytes"),
                        path.as_os_str().as_bytes(),
                    )
                }
                Err(error) => {
                    complete &= capture_error(directory, &format!("process-{name}.error"), error);
                }
            }
        }
    }
    // Explicit build/run custody only. Never dump the inherited environment or credentials.
    let values = [
        "GITHUB_SHA",
        "GITHUB_REPOSITORY",
        "GITHUB_RUN_ID",
        "GITHUB_RUN_ATTEMPT",
        "GITHUB_EVENT_NAME",
        "VIZE_DAVINCI_DIFFERENTIAL_CORPUS",
        "CARGO_TARGET_DIR",
        "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
        "RUSTC",
        "RUSTC_WRAPPER",
        "RUSTC_WORKSPACE_WRAPPER",
        "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER",
    ];
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        for key in values {
            if let Some(value) = env::var_os(key) {
                complete &= record(
                    directory,
                    &format!("environment-{key}.bytes"),
                    value.as_bytes(),
                );
            }
        }
    }
    for (name, path) in [
        (
            "workspace-cargo-config",
            PathBuf::from(".cargo/config.toml"),
        ),
        ("toolchain", PathBuf::from("rust-toolchain.toml")),
        ("cargo-lock", PathBuf::from("Cargo.lock")),
    ] {
        match fs::read(&path) {
            Ok(bytes) => {
                complete &= record(directory, &format!("{name}.bytes"), bytes);
                complete &= hash(directory, &format!("{name}-sha256"), &path);
            }
            Err(error) => {
                complete &= capture_error(directory, &format!("{name}.error"), error);
            }
        }
    }
    // The Wild setup writes linker/rustflags to this file; preserve build ownership without credentials.toml.
    if let Some(home) = env::var_os("CARGO_HOME").or_else(|| {
        env::var_os("HOME").map(|home| PathBuf::from(home).join(".cargo").into_os_string())
    }) {
        let path = PathBuf::from(home).join("config.toml");
        if path.exists() {
            if let Err(error) = fs::copy(&path, directory.join("cargo-home-config.toml")) {
                complete &= capture_error(directory, "cargo-home-config.copy-error", error);
            }
            complete &= hash(directory, "cargo-home-config-sha256", &path);
        }
    }
    complete
}

fn capture_after(directory: &Path) -> bool {
    let mut complete = command(
        directory,
        "cargo-metadata-after",
        "cargo",
        &["metadata", "--locked", "--no-deps", "--format-version", "1"],
    );
    let target = env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target"));
    let deps = target.join("debug/deps");
    let mut retained = 0;
    match fs::read_dir(&deps) {
        Ok(entries) => {
            for entry in entries {
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(error) => {
                        complete &= capture_error(directory, "executable-entry.error", error);
                        continue;
                    }
                };
                let path = entry.path();
                let name = entry.file_name();
                let Some(name) = name.to_str() else {
                    complete = false;
                    continue;
                };
                if !name.starts_with("davinci_ssr_corpus-") || name.contains('.') || !path.is_file()
                {
                    continue;
                }
                let output = directory.join(name);
                if let Err(error) = fs::copy(&path, &output) {
                    complete &= capture_error(directory, &format!("{name}.copy-error"), error);
                    continue;
                }
                complete &= hash(directory, &format!("{name}-sha256"), &output);
                if let Some(output) = output.to_str() {
                    complete &= command(
                        directory,
                        &format!("{name}-elf"),
                        "readelf",
                        &["-h", "-n", output],
                    );
                    complete &= command(directory, &format!("{name}-link"), "ldd", &[output]);
                } else {
                    complete = false;
                }
                retained += 1;
            }
        }
        Err(error) => complete &= capture_error(directory, "executable-directory.error", error),
    }
    complete &= record(
        directory,
        "retained-executable-count.txt",
        format!("{retained}\n"),
    );
    complete && retained > 0
}

fn status_code(status: ExitStatus) -> u8 {
    if let Some(code) = status.code() {
        return u8::try_from(code).unwrap_or(1);
    }
    #[cfg(unix)]
    {
        return u8::try_from(128 + status.signal().unwrap_or(1)).unwrap_or(1);
    }
    #[cfg(not(unix))]
    {
        1
    }
}

fn main() -> ExitCode {
    assert_eq!(
        env::consts::OS,
        "linux",
        "Original syscall qualification requires Linux"
    );
    let mut args: Vec<OsString> = env::args_os().skip(1).collect();
    if args.first().is_some_and(|arg| arg == "--") {
        args.remove(0);
    }
    assert_eq!(
        args,
        ORIGINAL.iter().map(OsString::from).collect::<Vec<_>>(),
        "Original SSR argv must be unchanged"
    );
    assert_eq!(
        env::var("VIZE_DAVINCI_DIFFERENTIAL_CORPUS").unwrap(),
        "tests/_fixtures/_git"
    );
    let directory = env::current_dir()
        .unwrap()
        .join("real-project-davinci-dom-corpus/ssr-syscalls");
    if let Err(error) = fs::create_dir_all(&directory) {
        eprintln!("SSR diagnostic directory creation failed: {error}");
    }
    record(&directory, "original-argv.nul", ORIGINAL.join("\0") + "\0");
    record(&directory, "trace-arguments.nul", TRACE.join("\0") + "\0");
    let before = capture_before(&directory);
    // Observational driver hooks write vectors only after original collection, never pre-read sources.
    let result = Command::new("strace")
        .args(TRACE)
        .arg("-o")
        .arg(directory.join("trace"))
        .arg("--")
        .args(&args)
        .env("VIZE_DAVINCI_SSR_SOURCE_IO_TRACE", &directory)
        .status();
    let original_code = match result {
        Ok(status) => status_code(status),
        Err(error) => {
            capture_error(&directory, "original-spawn-error.txt", error);
            1
        }
    };
    record(
        &directory,
        "original-exit-code.txt",
        format!("{original_code}\n"),
    );
    let after = capture_after(&directory);
    record(
        &directory,
        "custody-complete.txt",
        format!("{}\n", before && after),
    );
    // Custody completeness is evidence metadata, not an additional acceptance layer.
    // Preserve the actual original command status, including its failure and signal.
    ExitCode::from(original_code)
}
