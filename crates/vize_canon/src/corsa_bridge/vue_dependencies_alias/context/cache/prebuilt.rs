//! One workspace revision built outside the native editor transaction.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use vize_carton::FxHashMap;

use super::{ContextFingerprint, SessionCache};
use crate::corsa_bridge::vue_dependencies_alias::context::AliasContext;

/// A prepared revision only serves the request that asked for it. An
/// abandoned one must not keep a whole workspace projection alive.
const PREBUILT_LIFETIME: Duration = Duration::from_secs(120);

pub(in super::super) struct PrebuiltContext {
    source_path: PathBuf,
    fingerprint: ContextFingerprint,
    context: AliasContext,
    built_at: Instant,
}

impl SessionCache {
    /// Changes whenever the session forgets its mirrors, so a build that
    /// started before the reset cannot be adopted after it.
    pub(in super::super) fn generation(&self) -> u64 {
        self.generation
    }

    pub(super) fn reset_prebuilt(&mut self) {
        self.prebuilt = None;
        self.generation = self.generation.wrapping_add(1);
    }

    /// Live members whose unsaved text a rebuilt revision registers again.
    /// The build output depends on this list, so adoption compares it.
    pub(in super::super) fn overlay_sources_to_refresh(
        &self,
        virtual_root: &Path,
        overlay_identity: u64,
        host: &Path,
        overlays: &FxHashMap<PathBuf, &str>,
    ) -> Vec<PathBuf> {
        let mut sources = self.project_sources_to_refresh(virtual_root, overlay_identity);
        sources.retain(|path| path != host && overlays.contains_key(path));
        sources
    }

    /// Keep one stamped, unmaterialized revision until its request re-enters
    /// the native transaction. Materialization stays inside that transaction.
    pub(in super::super) fn store_prebuilt(
        &mut self,
        generation: u64,
        source_path: PathBuf,
        fingerprint: ContextFingerprint,
        context: AliasContext,
    ) {
        if generation != self.generation {
            return;
        }
        self.prebuilt = Some(PrebuiltContext {
            source_path,
            fingerprint,
            context,
            built_at: Instant::now(),
        });
    }

    pub(in super::super) fn has_prebuilt(
        &mut self,
        source_path: &Path,
        fingerprint: &ContextFingerprint,
        overlays: &FxHashMap<PathBuf, &str>,
    ) -> bool {
        self.expire_prebuilt();
        self.prebuilt.as_ref().is_some_and(|prebuilt| {
            self.prebuilt_is_current(prebuilt, source_path, fingerprint, overlays)
        })
    }

    /// Adopt the prepared revision only while it is exactly what an inline
    /// build of the same request would produce; otherwise the caller builds.
    pub(in super::super) fn take_prebuilt(
        &mut self,
        source_path: &Path,
        fingerprint: &ContextFingerprint,
        overlays: &FxHashMap<PathBuf, &str>,
    ) -> Option<(AliasContext, ContextFingerprint)> {
        // Another revision of the same host (a hover without the workspace)
        // must leave the prepared one for the request that asked for it.
        if !self.has_prebuilt(source_path, fingerprint, overlays) {
            return None;
        }
        let prebuilt = self.prebuilt.take()?;
        Some((prebuilt.context, prebuilt.fingerprint))
    }

    fn expire_prebuilt(&mut self) {
        if self
            .prebuilt
            .as_ref()
            .is_some_and(|prebuilt| prebuilt.built_at.elapsed() > PREBUILT_LIFETIME)
        {
            self.prebuilt = None;
        }
    }

    fn prebuilt_is_current(
        &self,
        prebuilt: &PrebuiltContext,
        source_path: &Path,
        fingerprint: &ContextFingerprint,
        overlays: &FxHashMap<PathBuf, &str>,
    ) -> bool {
        if prebuilt.source_path != source_path || prebuilt.fingerprint != *fingerprint {
            return false;
        }
        let mut observed = crate::package_route::stamp::InputStampCache::default();
        if !prebuilt.fingerprint.stamps_still_valid(&mut observed) {
            return false;
        }
        let host = vize_carton::path::canonicalize_non_verbatim(source_path);
        let refreshed = prebuilt
            .context
            .mirror
            .as_ref()
            .map_or_else(Vec::new, |mirror| {
                self.overlay_sources_to_refresh(
                    mirror.virtual_root(),
                    fingerprint.overlay_identity(),
                    &host,
                    overlays,
                )
            });
        refreshed == prebuilt.context.refreshed_sources
    }
}

#[cfg(test)]
mod tests;
