//! Memoize facts derived from an SFC descriptor already parsed by its caller.

use std::sync::{Arc, Mutex, OnceLock};
use vize_atelier_sfc::{SfcDescriptor, script::SfcTypeProject};
use vize_l0::{FxHashMap, hash::hash_str};

#[derive(Clone, Copy)]
pub(crate) struct Facts {
    pub needs_project: bool,
    pub importer_sensitive: bool,
}

impl Facts {
    pub(crate) const PARSE_FAILURE: Self = Self {
        needs_project: false,
        importer_sensitive: true,
    };

    pub(crate) fn for_descriptor(descriptor: &SfcDescriptor<'_>) -> Self {
        let needs_project = SfcTypeProject::is_needed_by(descriptor);
        Self {
            needs_project,
            importer_sensitive: needs_project,
        }
    }
}

type Cell = Arc<OnceLock<Facts>>;

#[derive(Default)]
pub(crate) struct SourceFacts {
    entries: Mutex<FxHashMap<(u64, usize), Cell>>,
    #[cfg(test)]
    analyses: std::sync::atomic::AtomicUsize,
}

impl SourceFacts {
    pub(crate) fn lookup(&self, source: &str) -> Option<Facts> {
        self.entries
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&(hash_str(source), source.len()))?
            .get()
            .copied()
    }

    pub(crate) fn record(&self, source: &str, derive: impl FnOnce() -> Facts) -> Facts {
        let cell = self
            .entries
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .entry((hash_str(source), source.len()))
            .or_default()
            .clone();
        *cell.get_or_init(|| {
            #[cfg(test)]
            self.analyses
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            derive()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_runtime_and_type_import_bodies_are_classified_once_each() {
        let cache = SourceFacts::default();
        let runtime =
            "<script setup>import { ref } from 'vue';const type = ref('import type');</script>";
        let typed = "<script setup lang=\"ts\">import type { Props } from '@/props';defineProps<Props>();</script>";
        let normal =
            "<script lang=\"ts\">import type { Props } from '@/props';export default {};</script>";
        let ordinary_descriptor =
            vize_atelier_sfc::parse_sfc(runtime, Default::default()).expect("runtime source");
        let typed_descriptor =
            vize_atelier_sfc::parse_sfc(typed, Default::default()).expect("typed source");
        let normal_descriptor =
            vize_atelier_sfc::parse_sfc(normal, Default::default()).expect("normal script source");
        for _ in 0..512 {
            let ordinary = cache.record(runtime, || Facts::for_descriptor(&ordinary_descriptor));
            assert!(!ordinary.needs_project);
            assert!(!ordinary.importer_sensitive);
            let imported = cache.record(typed, || Facts::for_descriptor(&typed_descriptor));
            assert!(imported.needs_project);
            assert!(imported.importer_sensitive);
            let standalone = cache.record(normal, || Facts::for_descriptor(&normal_descriptor));
            assert!(!standalone.needs_project);
            assert!(!standalone.importer_sensitive);
        }
        assert_eq!(cache.analyses.load(std::sync::atomic::Ordering::Relaxed), 3);
    }
}
