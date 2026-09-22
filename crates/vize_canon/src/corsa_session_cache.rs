//! Cache key for Corsa project sessions (issue #699, Davinci P5-8).
//!
//! A reused `ProjectSession` must never serve another project, tsconfig,
//! Corsa build, flag set or host: a key covering less than the session's
//! inputs is a cache-corruption bug, not a performance detail. The key is
//! therefore the P5-1b ambient manifest's fingerprint for the
//! `corsa.session` artifact (`davinci-road/plan/key-manifests.md`), which
//! folds **exactly** these inputs — the fold refuses a missing or an extra
//! one:
//!
//! | Input               | Value                                                         |
//! | ------------------- | ------------------------------------------------------------- |
//! | `project-identity`  | the canonical `tsconfig.json` path                            |
//! | `tsconfig-content`  | every config of its `extends` chain, path and bytes, in order |
//! | `toolchain-version` | the Vize version that built this binary                       |
//! | `corsa-version`     | the Corsa build identity (see [`corsa_build_identity`])       |
//! | `feature-flags`     | the caller's canonical spelling of its checking flags         |
//! | `platform`          | the host's architecture and OS                                |
//!
//! The session map and its lifecycle consume this key (`vize check-server`).

use core::fmt::Write as _;
use std::path::{Path, PathBuf};

use vize_carton::String;
use vize_davinci::key::manifest::{AmbientInput, CachedArtifact, KeyManifest};

mod chain;

pub use chain::tsconfig_chain_digest;

/// The resolved value of every ambient input of a Corsa session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionInputs {
    /// `project-identity`: the canonical `tsconfig.json` path.
    pub project: PathBuf,
    /// `tsconfig-content`: the digest of the config's `extends` chain.
    pub tsconfig: String,
    /// `toolchain-version`: the Vize version.
    pub toolchain: String,
    /// `corsa-version`: the Corsa build identity.
    pub corsa: String,
    /// `feature-flags`: the caller's canonical flag spelling.
    pub flags: String,
    /// `platform`: the host's architecture and OS.
    pub platform: String,
}

impl SessionInputs {
    /// Resolve every input for the session of `tsconfig_path`, reading the
    /// config's `extends` chain from disk, with the Corsa build
    /// `corsa_version` and the checking flags `flags`.
    pub fn resolve(tsconfig_path: impl AsRef<Path>, corsa_version: &str, flags: &str) -> Self {
        let path = tsconfig_path.as_ref().to_path_buf();
        let project = std::fs::canonicalize(&path).unwrap_or(path);
        Self {
            tsconfig: tsconfig_chain_digest(&project),
            project,
            toolchain: String::from(env!("CARGO_PKG_VERSION")),
            corsa: String::from(corsa_version),
            flags: String::from(flags),
            platform: host_platform(),
        }
    }

    /// The P5-1b manifest over these inputs.
    fn manifest(&self) -> KeyManifest {
        KeyManifest::new()
            .with(
                AmbientInput::ProjectIdentity,
                &self.project.to_string_lossy(),
            )
            .with(AmbientInput::TsconfigContent, &self.tsconfig)
            .with(AmbientInput::ToolchainVersion, &self.toolchain)
            .with(AmbientInput::CorsaVersion, &self.corsa)
            .with(AmbientInput::FeatureFlags, &self.flags)
            .with(AmbientInput::Platform, &self.platform)
    }
}

/// Cache key identifying a Corsa project session: the `corsa.session`
/// manifest fingerprint over [`SessionInputs`]. Two sessions are
/// interchangeable exactly when every ambient input is equal.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CorsaSessionKey {
    tsconfig_path: PathBuf,
    fingerprint: [u8; 16],
}

impl CorsaSessionKey {
    /// The key of the session for `tsconfig_path` run by Corsa build
    /// `corsa_version` with checking flags `flags` (see
    /// [`SessionInputs::resolve`]).
    pub fn new(tsconfig_path: impl AsRef<Path>, corsa_version: &str, flags: &str) -> Self {
        Self::from_inputs(&SessionInputs::resolve(tsconfig_path, corsa_version, flags))
    }

    /// The key over already-resolved inputs.
    pub fn from_inputs(inputs: &SessionInputs) -> Self {
        let fingerprint = inputs
            .manifest()
            .fingerprint(CachedArtifact::CorsaSession)
            .expect("SessionInputs sets exactly the corsa.session inputs");
        Self {
            tsconfig_path: inputs.project.clone(),
            fingerprint,
        }
    }

    /// Canonical `tsconfig.json` path of the session's project.
    pub fn tsconfig_path(&self) -> &Path {
        &self.tsconfig_path
    }

    /// The manifest fingerprint — the key's identity.
    pub fn fingerprint(&self) -> [u8; 16] {
        self.fingerprint
    }
}

/// The identity of the Corsa build at `executable`: its canonical path, size
/// and modification time. Replacing or reinstalling the binary changes it.
pub fn corsa_build_identity(executable: impl AsRef<Path>) -> String {
    let path = executable.as_ref();
    let canonical = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let mut identity = String::from(canonical.to_string_lossy().as_ref());
    match std::fs::metadata(&canonical) {
        Ok(metadata) => {
            let modified = metadata
                .modified()
                .ok()
                .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                .map_or(0, |since| since.as_nanos());
            let len = metadata.len();
            let _infallible = write!(identity, " len={len} mtime={modified}");
        }
        Err(_) => identity.push_str(" missing"),
    }
    identity
}

/// The host platform: architecture, OS and family.
pub fn host_platform() -> String {
    let mut platform = String::default();
    append(
        &mut platform,
        &[
            std::env::consts::ARCH,
            "-",
            std::env::consts::OS,
            "-",
            std::env::consts::FAMILY,
        ],
    );
    platform
}

fn append(out: &mut String, parts: &[&str]) {
    for part in parts {
        out.push_str(part);
    }
}

#[cfg(test)]
mod tests;
