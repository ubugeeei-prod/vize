//! The P6-4a hosting choice: the pinned native `moonc`, one process per
//! check, behind [`MooncHost`].
//!
//! Measured in the spike (record: `davinci-road/plan/phase-6-records/p6-4a.md`):
//! `moonc` is a native OCaml binary, the toolchain CI already installs at
//! the `.moonbit-version` pin; no wasm build of it is published, so there
//! is nothing for wasmtime to load. The native binary takes the virtual
//! file from its command line (`-replace-name` / `-replace-content`), so
//! no source file is ever written; it does write `<pkg>.ast` and
//! `<pkg>.typechecked` beside `-o` even with `-no-mi`, so each check runs
//! in a private scratch directory that is removed afterwards.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use vize_s0::{String, ToCompactString, cstr};

use crate::host::{CheckUnit, HostError, MooncHost, RawCheck};

/// Checks started by this process: each gets its own scratch directory.
static RUNS: AtomicU64 = AtomicU64::new(0);

/// The largest virtual file one call carries: Linux caps a single argv
/// entry at 32 pages (`MAX_ARG_STRLEN`, 131072 bytes with its NUL).
pub const ARGUMENT_LIMIT: usize = 128 * 1024 - 1;

/// Warnings the projection disables: `missing_priv` (4) asks for `priv` on
/// types absent from the package's public signature, and the virtual
/// package has no public signature.
const WARNINGS: &str = "-4";

/// The native `moonc` of one MoonBit installation.
#[derive(Debug)]
pub struct NativeMoonc {
    moonc: PathBuf,
    std_path: PathBuf,
    toolchain: String,
}

impl NativeMoonc {
    /// The installation at `$MOON_HOME`, else `$HOME/.moon`.
    ///
    /// # Errors
    ///
    /// [`HostError::Unavailable`] when neither is set or the installation
    /// is incomplete.
    pub fn discover() -> Result<Self, HostError> {
        let home = std::env::var_os("MOON_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| Path::new(&home).join(".moon")))
            .ok_or_else(|| HostError::Unavailable("neither MOON_HOME nor HOME is set".into()))?;
        Self::at(&home)
    }

    /// The installation at `moon_home`.
    ///
    /// # Errors
    ///
    /// [`HostError::Unavailable`] when `moonc` does not run or the core
    /// bundle for `wasm-gc` is missing.
    pub fn at(moon_home: &Path) -> Result<Self, HostError> {
        let moonc = moon_home
            .join("bin")
            .join(if cfg!(windows) { "moonc.exe" } else { "moonc" });
        let std_path = moon_home.join("lib/core/_build/wasm-gc/release/bundle");
        let output = Command::new(&moonc)
            .arg("-v")
            .output()
            .map_err(|error| HostError::Unavailable(cstr!("{}: {error}", moonc.display())))?;
        // `moonc -v` prints `v<version> (<date>)`.
        let toolchain = core::str::from_utf8(&output.stdout)
            .ok()
            .and_then(|text| text.split_whitespace().next())
            .map(|word| word.trim_start_matches('v').to_compact_string())
            .filter(|_| output.status.success())
            .ok_or_else(|| HostError::Unavailable(cstr!("`{} -v` failed", moonc.display())))?;
        if !std_path.join("prelude/prelude.mi").is_file() {
            return Err(HostError::Unavailable(cstr!(
                "no wasm-gc core bundle at {}",
                std_path.display()
            )));
        }
        Ok(Self {
            moonc,
            std_path,
            toolchain,
        })
    }
}

impl MooncHost for NativeMoonc {
    fn toolchain(&self) -> &str {
        &self.toolchain
    }

    fn check(&mut self, unit: &CheckUnit<'_>) -> Result<RawCheck, HostError> {
        if unit.source.len() > ARGUMENT_LIMIT {
            return Err(HostError::TooLarge {
                bytes: unit.source.len(),
                limit: ARGUMENT_LIMIT,
            });
        }
        let run = RUNS.fetch_add(1, Ordering::Relaxed);
        let scratch =
            std::env::temp_dir().join(cstr!("vize-moonc-{}-{run}", std::process::id()).as_str());
        std::fs::create_dir_all(&scratch)
            .map_err(|error| HostError::Unavailable(cstr!("{}: {error}", scratch.display())))?;
        let stem = unit.package.rsplit('/').next().unwrap_or(unit.package);
        let mut prelude = OsString::from(self.std_path.join("prelude/prelude.mi"));
        prelude.push(":prelude");
        let output = Command::new(&self.moonc)
            .current_dir(&scratch)
            .arg("check")
            .arg("-o")
            .arg(scratch.join(cstr!("{stem}.mi").as_str()))
            .args([
                "-no-mi",
                "-w",
                WARNINGS,
                "-pkg",
                unit.package,
                "-pkg-type",
                "library",
            ])
            .arg("-std-path")
            .arg(&self.std_path)
            .arg("-i")
            .arg(prelude)
            .args(["-target", "wasm-gc", "-error-format", "json"])
            .args([
                "-replace-name",
                unit.file_name,
                "-replace-content",
                unit.source,
            ])
            .arg(unit.file_name)
            .output();
        let removed = std::fs::remove_dir_all(&scratch);
        let output = output.map_err(|error| HostError::Unavailable(cstr!("{error}")))?;
        removed.map_err(|error| HostError::Failed(cstr!("scratch cleanup: {error}")))?;
        // Diagnostics are the JSON lines of stderr; exit 0 means none were
        // errors, 2 means some were. Anything else is a toolchain failure.
        let stderr = core::str::from_utf8(&output.stderr).unwrap_or_default();
        let lines: Vec<String> = stderr
            .lines()
            .filter(|line| line.starts_with('{'))
            .map(ToCompactString::to_compact_string)
            .collect();
        match output.status.code() {
            Some(0) => Ok(RawCheck {
                toolchain: self.toolchain.clone(),
                lines,
            }),
            Some(2) if !lines.is_empty() => Ok(RawCheck {
                toolchain: self.toolchain.clone(),
                lines,
            }),
            status => Err(HostError::Failed(cstr!(
                "exit {status:?}: {}",
                stderr.trim()
            ))),
        }
    }
}
