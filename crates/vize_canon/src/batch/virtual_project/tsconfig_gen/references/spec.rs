use std::path::{Path, PathBuf};

use vize_carton::{FxHashMap, FxHashSet};
pub(super) use vize_l0::typescript_ownership::OwnershipSpec;
use vize_l0::typescript_ownership::extends_entries;

use super::super::super::tsconfig_paths::resolve_extended_tsconfig_path_with_inputs;
use super::super::compiler_inputs::CompilerInputs;
use super::{graph, parse_jsonc_value, resolve_extended_tsconfig_path};

#[derive(Default)]
pub(super) struct SpecCache {
    active: FxHashSet<PathBuf>,
    completed: FxHashMap<PathBuf, Option<OwnershipSpec>>,
    /// Config spellings resolved once: the cache already freezes the configs.
    normalized: FxHashMap<PathBuf, PathBuf>,
}

impl SpecCache {
    pub(super) fn load(&mut self, config_path: &Path) -> Option<OwnershipSpec> {
        self.load_with_inputs(config_path, None)
    }

    pub(super) fn load_with_inputs(
        &mut self,
        config_path: &Path,
        inputs: Option<&CompilerInputs>,
    ) -> Option<OwnershipSpec> {
        let normalized = self.normalize(config_path);
        if let Some(inputs) = inputs {
            inputs.record_config_alias(config_path, &normalized);
        }
        if let Some(cached) = self.completed.get(&normalized) {
            return cached.clone();
        }
        if !self.active.insert(normalized.clone()) {
            return None;
        }
        let loaded = self.load_uncached(&normalized, inputs);
        self.active.remove(&normalized);
        self.completed.insert(normalized, loaded.clone());
        loaded
    }

    pub(super) fn normalize(&mut self, config_path: &Path) -> PathBuf {
        self.normalized
            .entry(config_path.to_path_buf())
            .or_insert_with_key(|path| graph::normalize_path(path))
            .clone()
    }

    fn load_uncached(
        &mut self,
        config_path: &Path,
        inputs: Option<&CompilerInputs>,
    ) -> Option<OwnershipSpec> {
        let config = if let Some(inputs) = inputs {
            inputs.read_jsonc(config_path).ok()?
        } else {
            let content = std::fs::read_to_string(config_path).ok()?;
            parse_jsonc_value(&content).ok()?
        };
        let parents = extends_entries(&config)
            .into_iter()
            .filter_map(|parent| {
                if inputs.is_some() {
                    resolve_extended_tsconfig_path_with_inputs(config_path, parent, inputs)
                } else {
                    resolve_extended_tsconfig_path(config_path, parent)
                }
            })
            .filter_map(|parent| self.load_with_inputs(&parent, inputs))
            .collect::<Vec<_>>();
        Some(OwnershipSpec::from_config(config_path, &config, parents))
    }
}
