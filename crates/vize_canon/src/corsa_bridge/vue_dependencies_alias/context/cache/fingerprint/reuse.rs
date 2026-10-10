//! Only authored-source revisions may take ownership of a cached graph.

use std::path::PathBuf;

use vize_carton::{FxHashMap, FxHashSet};

use super::ContextFingerprint;
use crate::corsa_bridge::vue_dependencies_alias::AliasContext;

#[derive(Debug, PartialEq, Eq)]
pub(in crate::corsa_bridge::vue_dependencies_alias::context) enum SourceGuard {
    Settings,
    RequestedSources,
    ClosedOverlay,
    AddedOverlay,
    Configuration,
    PackageInput,
    ResolutionInput,
}

impl SourceGuard {
    pub(in crate::corsa_bridge::vue_dependencies_alias::context) fn code(&self) -> u8 {
        match self {
            Self::Settings => 1,
            Self::RequestedSources => 2,
            Self::ClosedOverlay => 3,
            Self::AddedOverlay => 4,
            Self::Configuration => 5,
            Self::PackageInput => 6,
            Self::ResolutionInput => 7,
        }
    }
}

impl ContextFingerprint {
    pub(in crate::corsa_bridge::vue_dependencies_alias::context) fn source_changes(
        &self,
        current: &Self,
        context: &AliasContext,
        overlays: &FxHashMap<PathBuf, &str>,
    ) -> Result<Vec<PathBuf>, SourceGuard> {
        if self.generation_options != current.generation_options {
            return Err(SourceGuard::Settings);
        }
        if self.requested_paths != current.requested_paths {
            return Err(SourceGuard::RequestedSources);
        }
        let mirror = context.mirror.as_ref().ok_or(SourceGuard::Settings)?;
        let known = mirror
            .registered_original_paths_sorted()
            .into_iter()
            .collect::<FxHashSet<_>>();
        let configurations = mirror
            .governing_config_paths()
            .into_iter()
            .collect::<FxHashSet<_>>();
        // A closed existing buffer must return to disk through the cold path.
        // A deleted known source, however, can release its old graph owners.
        for path in &self.overlay_paths {
            if !overlays.contains_key(path) && (path.is_file() || !known.contains(path)) {
                return Err(SourceGuard::ClosedOverlay);
            }
        }
        if current
            .overlay_paths
            .iter()
            .any(|path| !self.overlay_paths.contains(path) && !known.contains(path))
        {
            return Err(SourceGuard::AddedOverlay);
        }
        let mut observed = crate::package_route::stamp::InputStampCache::default();
        let mut changed = Vec::new();
        for stamp in &self.stamps {
            if stamp.is_current_with_cache(&mut observed) {
                continue;
            }
            let path = stamp.path();
            // Missing probes, higher-priority companions, configs and package
            // inputs still invalidate the complete producer. Never turn a
            // newly created resolution target into a silently reused graph.
            if configurations.contains(path) {
                return Err(SourceGuard::Configuration);
            }
            if context
                .route_inputs
                .iter()
                .any(|input| input.as_path() == path)
            {
                return Err(SourceGuard::PackageInput);
            }
            if !known.contains(path) {
                return Err(SourceGuard::ResolutionInput);
            }
            changed.push(path.to_path_buf());
        }
        Ok(changed)
    }
}
