//! Explicit absence of the reviewed loader's ambient RC and environment inputs.

use std::path::{Path, PathBuf};

use vize_l0::FxHashSet;

use super::io::{PathMetadata, ProbeIo};

const MAX_RC_ANCESTORS: usize = 64;

#[derive(Clone)]
pub(crate) struct Environment {
    pub(crate) home: Option<PathBuf>,
    pub(crate) xdg: Option<PathBuf>,
    pub(crate) opaque: bool,
}

impl Environment {
    pub(crate) fn capture() -> Self {
        let home = std::env::var_os("HOME")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from);
        let xdg = std::env::var_os("XDG_CONFIG_HOME")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from);
        let opaque = std::env::vars_os().any(|(key, value)| opaque_entry(&key, &value));
        Self { home, xdg, opaque }
    }

    pub(crate) fn absent(&self, root: &Path, io: &mut ProbeIo) -> Option<()> {
        if self.opaque || cfg!(windows) {
            return None;
        }
        let home = self.home.as_deref()?;
        if !home.is_absolute()
            || home.to_str().is_none()
            || io.canonical_directory(home, false)?.as_path() != home
        {
            return None;
        }
        let user_config = self.xdg.clone().unwrap_or_else(|| home.join(".config"));
        if !user_config.is_absolute() || user_config.to_str().is_none() {
            return None;
        }
        let mut probes = Vec::new();
        let mut seen = FxHashSet::default();
        // Any workspace chosen by bundled pkg-types is an ancestor. Probe all
        // capped ancestors instead of reparsing workspace manifests to select it.
        for (index, ancestor) in root.ancestors().enumerate() {
            if index == MAX_RC_ANCESTORS {
                return None;
            }
            push_probe(&mut probes, &mut seen, ancestor.join(".nuxtrc"));
        }
        push_probe(&mut probes, &mut seen, home.join(".nuxtrc"));
        match io.metadata(&user_config) {
            PathMetadata::Missing => {}
            PathMetadata::Present(metadata)
                if metadata.is_dir() && !metadata.file_type().is_symlink() =>
            {
                if io.canonical_directory(&user_config, false)?.as_path() != user_config.as_path() {
                    return None;
                }
                push_probe(&mut probes, &mut seen, user_config.join(".nuxtrc"));
            }
            _ => return None,
        }
        for path in probes {
            if !matches!(io.metadata(&path), PathMetadata::Missing) {
                return None;
            }
        }
        Some(())
    }
}

fn push_probe(probes: &mut Vec<PathBuf>, seen: &mut FxHashSet<PathBuf>, path: PathBuf) {
    if seen.insert(path.clone()) {
        probes.push(path);
    }
}

fn opaque_entry(key: &std::ffi::OsStr, value: &std::ffi::OsStr) -> bool {
    let Some(key) = key.to_str() else { return true };
    if matches!(
        key,
        "NODE_OPTIONS" | "NODE_PATH" | "NODE_PRESERVE_SYMLINKS" | "NODE_PRESERVE_SYMLINKS_MAIN"
    ) {
        return value.to_str().is_none_or(|value| !value.trim().is_empty());
    }
    (key.starts_with("NUXT_") && !(key == "NUXT_TELEMETRY_DISABLED" && value == "1"))
        || key.starts_with("C12_")
        || key.starts_with("DOTENV_")
        || key.starts_with("JITI_")
}

#[cfg(test)]
mod tests {
    use super::opaque_entry;
    use std::ffi::OsStr;

    #[test]
    fn loader_and_node_resolution_overrides_are_unknown_without_mutating_process_environment() {
        for key in [
            "NUXT_APP_BASE_URL",
            "C12_CONFIG",
            "DOTENV_CONFIG_PATH",
            "JITI_ESM_RESOLVE",
            "NODE_OPTIONS",
            "NODE_PATH",
            "NODE_PRESERVE_SYMLINKS",
            "NODE_PRESERVE_SYMLINKS_MAIN",
        ] {
            assert!(opaque_entry(OsStr::new(key), OsStr::new("1")), "{key}");
        }
        assert!(!opaque_entry(
            OsStr::new("NUXT_TELEMETRY_DISABLED"),
            OsStr::new("1")
        ));
        assert!(!opaque_entry(OsStr::new("NODE_OPTIONS"), OsStr::new("")));
    }
}
