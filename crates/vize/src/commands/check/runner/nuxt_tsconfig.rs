//! Nuxt path-alias fallback `tsconfig` synthesis for the `check` runner.
//!
//! When Nuxt auto-imports contribute path aliases, the runner publishes an
//! immutable wrapper `tsconfig` in Vize's cache. Canon flattens the authored
//! config chain into the wrapper, so its meaning never depends on a live
//! `extends` target, shared dependency tree, or cache spelling.

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use vize_s0::String;

mod cache;
use cache::{
    ConfigLease, acquire_config_lease, collect_cache_projects, collect_project_cache,
    config_cache_root, encode_digest, publish_config_atomically, update_digest_path,
};
#[cfg(test)]
use cache::{publish_config_atomically_with_hook, validate_config_cache_root};

#[cfg(all(test, unix))]
mod isolation_tests;
#[cfg(test)]
mod legacy_options_tests;
#[cfg(test)]
mod path_options_tests;

const CONFIG_IDENTITY_SCHEMA: &[u8] = b"vize-nuxt-fallback-config:v2\0";

/// The immutable config selected for one Corsa program.
pub(super) struct PreparedCheckerTsconfig {
    path: Option<PathBuf>,
    _lease: Option<ConfigLease>,
}

impl PreparedCheckerTsconfig {
    pub(super) fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    fn is_generated(&self) -> bool {
        self._lease.is_some()
    }
}

pub(super) fn resolve_checker_tsconfig_path(
    program_tsconfig_path: Option<&Path>,
    project_root: &Path,
    nuxt_alias_base_dir: &Path,
    nuxt_path_aliases: &[super::super::nuxt::NuxtPathAlias],
) -> Result<PreparedCheckerTsconfig, std::io::Error> {
    if nuxt_path_aliases.is_empty() {
        return Ok(PreparedCheckerTsconfig {
            path: program_tsconfig_path.map(Path::to_path_buf),
            _lease: None,
        });
    }

    write_nuxt_fallback_tsconfig(
        program_tsconfig_path,
        project_root,
        nuxt_alias_base_dir,
        nuxt_path_aliases,
    )
}

/// Test-only process barrier at the exact publication-to-consumption boundary.
/// Normal invocations never set these environment variables and take the
/// zero-cost early return.
pub(super) fn wait_for_prepared_config_test_barrier(
    prepared: &PreparedCheckerTsconfig,
) -> Result<(), std::io::Error> {
    wait_for_config_test_barrier(
        prepared,
        "VIZE_TEST_NUXT_CONFIG_PREPARED_BARRIER",
        "prepared-config",
    )
    .map(|_| ())
}

pub(super) fn wait_for_active_config_test_barrier(
    prepared: &PreparedCheckerTsconfig,
) -> Result<bool, std::io::Error> {
    wait_for_config_test_barrier(
        prepared,
        "VIZE_TEST_NUXT_CONFIG_ACTIVE_BARRIER",
        "active-config",
    )
}

fn wait_for_config_test_barrier(
    prepared: &PreparedCheckerTsconfig,
    variable: &str,
    phase: &str,
) -> Result<bool, std::io::Error> {
    if !cfg!(debug_assertions) {
        return Ok(false);
    }
    let Some(barrier) = std::env::var_os(variable) else {
        return Ok(false);
    };
    if !prepared.is_generated() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("{phase} barrier requires a generated Nuxt checker config"),
        ));
    }
    let participant = std::env::var_os("VIZE_TEST_NUXT_CONFIG_PARTICIPANT").ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "VIZE_TEST_NUXT_CONFIG_PARTICIPANT is required with the prepared-config barrier",
        )
    })?;
    #[cfg(unix)]
    {
        use std::io::{Read as _, Write as _};

        let barrier = PathBuf::from(barrier);
        fs::OpenOptions::new()
            .write(true)
            .open(barrier.join("ready"))?
            .write_all(b"x")?;
        let mut release = [0_u8; 3];
        fs::OpenOptions::new()
            .read(true)
            .open(barrier.join(format!("release-{}", participant.to_string_lossy())))?
            .read_exact(&mut release)?;
        if release != *b"go\n" {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "prepared-config barrier received an invalid release token",
            ));
        }
        Ok(true)
    }
    #[cfg(not(unix))]
    {
        let _ = (barrier, participant);
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            format!("the {phase} test barrier requires Unix FIFOs"),
        ))
    }
}

pub(super) fn write_nuxt_fallback_tsconfig(
    program_tsconfig_path: Option<&Path>,
    project_root: &Path,
    nuxt_alias_base_dir: &Path,
    nuxt_path_aliases: &[super::super::nuxt::NuxtPathAlias],
) -> Result<PreparedCheckerTsconfig, std::io::Error> {
    let cache_root = config_cache_root(project_root)?;
    write_nuxt_fallback_tsconfig_in_cache(
        program_tsconfig_path,
        project_root,
        nuxt_alias_base_dir,
        nuxt_path_aliases,
        &cache_root,
    )
}

fn write_nuxt_fallback_tsconfig_in_cache(
    program_tsconfig_path: Option<&Path>,
    project_root: &Path,
    nuxt_alias_base_dir: &Path,
    nuxt_path_aliases: &[super::super::nuxt::NuxtPathAlias],
    cache_root: &Path,
) -> Result<PreparedCheckerTsconfig, std::io::Error> {
    fs::create_dir_all(cache_root)?;
    let mut compiler_options = if let Some(tsconfig_path) = program_tsconfig_path {
        vize_canon::snapshot_tsconfig_compiler_options(project_root, tsconfig_path)
            .map_err(std::io::Error::other)?
    } else {
        Map::new()
    };
    let paths = compiler_options
        .entry("paths")
        .or_insert_with(|| Value::Object(Map::new()));
    if let Some(paths) = paths.as_object_mut() {
        for alias in nuxt_path_aliases {
            paths
                .entry(alias.pattern.as_str().to_owned())
                .or_insert_with(|| {
                    Value::Array(
                        alias
                            .targets
                            .iter()
                            .map(|target| {
                                Value::String(
                                    absolute_tsconfig_path_target(
                                        nuxt_alias_base_dir,
                                        target.as_str(),
                                    )
                                    .into(),
                                )
                            })
                            .collect(),
                    )
                });
        }
    }

    if program_tsconfig_path.is_some() {
        // Nuxt 2 and Bridge projects commonly remain on TypeScript 5 configs
        // that require `baseUrl` and legacy Node resolution. This wrapper is
        // Vize's compatibility layer, so acknowledge TypeScript 6 deprecations
        // here instead of letting the native TypeScript 7 option probe turn
        // them into errors on this generated file. Unrelated config errors are
        // still reported by the probe.
        compiler_options
            .entry("ignoreDeprecations")
            .or_insert_with(|| Value::String("6.0".into()));
    }

    let mut config = Map::new();
    config.insert("compilerOptions".into(), Value::Object(compiler_options));

    let content =
        serde_json::to_vec_pretty(&Value::Object(config)).map_err(std::io::Error::other)?;
    let mut project_identity = Sha256::new();
    project_identity.update(CONFIG_IDENTITY_SCHEMA);
    update_digest_path(&mut project_identity, project_root);
    let project_identity = encode_digest(project_identity.finalize());
    let mut config_identity = Sha256::new();
    config_identity.update(CONFIG_IDENTITY_SCHEMA);
    config_identity.update(&content);
    let config_identity = encode_digest(config_identity.finalize());
    let (project_cache, entry, lease) = acquire_config_lease(
        cache_root,
        project_identity.as_str(),
        config_identity.as_str(),
    )?;
    let wrapper_path = entry.join("tsconfig.nuxt-fallback.json");
    publish_config_atomically(&wrapper_path, &content)?;
    collect_project_cache(&project_cache, &entry)?;
    collect_cache_projects(cache_root, &project_cache)?;
    Ok(PreparedCheckerTsconfig {
        path: Some(wrapper_path),
        _lease: Some(lease),
    })
}

fn absolute_tsconfig_path_target(source_base_dir: &Path, target: &str) -> String {
    path_to_tsconfig_target(&absolute_tsconfig_path(source_base_dir, target))
}

fn absolute_tsconfig_path(source_base_dir: &Path, target: &str) -> PathBuf {
    let target_path = Path::new(target);
    let target_path = if target_path.is_absolute() {
        target_path.to_path_buf()
    } else {
        vize_s0::path::canonicalize_non_verbatim(source_base_dir).join(target_path)
    };
    normalize_path_lexically(&target_path)
}

fn path_to_tsconfig_target(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/").into()
}

fn normalize_path_lexically(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if !normalized.pop() {
                    normalized.push(component.as_os_str());
                }
            }
            _ => normalized.push(component.as_os_str()),
        }
    }
    normalized
}
