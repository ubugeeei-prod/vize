//! Memoize facts derived from an SFC descriptor already parsed by its caller.

use std::sync::{Arc, Mutex, OnceLock};
use vize_atelier_sfc::{SfcDescriptor, SfcScriptBlock, script::SfcTypeProject};
use vize_l0::{FxHashMap, String, hash::hash_str};

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
type ScriptHashes = [Option<(u64, usize)>; 2];

/// The source inputs of a script classification, without source locations or
/// derived bindings. Facts contain only booleans, so a different template or a
/// different script offset does not change them. Keep language, external-source
/// and authored attributes as well as the exact script bytes in the key.
struct ScriptInput {
    content: String,
    lang: Option<String>,
    src: Option<String>,
    setup: bool,
    attrs: FxHashMap<String, String>,
}

impl ScriptInput {
    fn of(script: &SfcScriptBlock<'_>) -> Self {
        Self {
            content: script.content.as_ref().into(),
            lang: script.lang.as_deref().map(Into::into),
            src: script.src.as_deref().map(Into::into),
            setup: script.setup,
            attrs: script
                .attrs
                .iter()
                .map(|(name, value)| (name.as_ref().into(), value.as_ref().into()))
                .collect(),
        }
    }

    fn matches(&self, script: &SfcScriptBlock<'_>) -> bool {
        self.content == script.content.as_ref()
            && self.lang.as_deref() == script.lang.as_deref()
            && self.src.as_deref() == script.src.as_deref()
            && self.setup == script.setup
            && self.attrs.len() == script.attrs.len()
            && script.attrs.iter().all(|(name, value)| {
                self.attrs
                    .get(name.as_ref())
                    .is_some_and(|stored| stored == value.as_ref())
            })
    }
}

struct ScriptFacts {
    inputs: [Option<ScriptInput>; 2],
    facts: OnceLock<Facts>,
}

struct ScriptCache {
    buckets: FxHashMap<ScriptHashes, Vec<Arc<ScriptFacts>>>,
    bytes: usize,
    limit: usize,
}

impl Default for ScriptCache {
    fn default() -> Self {
        Self {
            buckets: FxHashMap::default(),
            bytes: 0,
            // Classification is only a boolean query. Bound its retained keys
            // within each request; large unique scripts need no extra copy.
            limit: 1 << 20,
        }
    }
}

impl ScriptFacts {
    fn matches(&self, scripts: [Option<&SfcScriptBlock<'_>>; 2]) -> bool {
        self.inputs
            .iter()
            .zip(scripts)
            .all(|(stored, script)| match (stored.as_ref(), script) {
                (Some(stored), Some(script)) => stored.matches(script),
                (None, None) => true,
                _ => false,
            })
    }
}

#[derive(Default)]
pub(crate) struct SourceFacts {
    entries: Mutex<FxHashMap<(u64, usize), Cell>>,
    /// Hashes select a bucket only; a hit compares every script input exactly.
    /// Hits borrow the current descriptor and allocate no script keys.
    scripts: Mutex<ScriptCache>,
    #[cfg(test)]
    analyses: std::sync::atomic::AtomicUsize,
}

impl SourceFacts {
    /// Classify scripts once even when their components have distinct templates
    /// and styles. The whole-source CLI lookup/record cache stays independent.
    pub(crate) fn for_descriptor(&self, descriptor: &SfcDescriptor<'_>) -> Facts {
        let scripts = [descriptor.script.as_ref(), descriptor.script_setup.as_ref()];
        let key = scripts
            .map(|script| script.map(|script| (hash_str(&script.content), script.content.len())));
        self.descriptor_with_key(descriptor, key)
    }

    fn descriptor_with_key(&self, descriptor: &SfcDescriptor<'_>, key: ScriptHashes) -> Facts {
        let scripts = [descriptor.script.as_ref(), descriptor.script_setup.as_ref()];
        let cell = {
            let mut cached = self
                .scripts
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            match cached
                .buckets
                .get(&key)
                .and_then(|bucket| bucket.iter().find(|entry| entry.matches(scripts)))
            {
                Some(entry) => Arc::clone(entry),
                None => {
                    let bytes = size_of::<ScriptFacts>()
                        + size_of::<ScriptHashes>()
                        + scripts
                            .iter()
                            .flatten()
                            .map(|script| {
                                script.content.len()
                                    + script.lang.as_ref().map_or(0, |lang| lang.len())
                                    + script.src.as_ref().map_or(0, |src| src.len())
                                    + script
                                        .attrs
                                        .iter()
                                        .map(|(name, value)| {
                                            name.len() + value.len() + 2 * size_of::<String>()
                                        })
                                        .sum::<usize>()
                            })
                            .sum::<usize>();
                    if cached.bytes.saturating_add(bytes) > cached.limit {
                        drop(cached);
                        return self.derive_descriptor(descriptor);
                    }
                    let entry = Arc::new(ScriptFacts {
                        inputs: scripts.map(|script| script.map(ScriptInput::of)),
                        facts: OnceLock::new(),
                    });
                    cached.bytes += bytes;
                    cached
                        .buckets
                        .entry(key)
                        .or_default()
                        .push(Arc::clone(&entry));
                    entry
                }
            }
        };
        *cell
            .facts
            .get_or_init(|| self.derive_descriptor(descriptor))
    }

    fn derive_descriptor(&self, descriptor: &SfcDescriptor<'_>) -> Facts {
        #[cfg(test)]
        self.analyses
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Facts::for_descriptor(descriptor)
    }

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
    use vize_l0::cstr;

    const SCRIPT: &str = "import type { Props } from './props';defineProps<Props>();";

    #[test]
    fn distinct_templates_and_script_offsets_share_one_classification() {
        let cache = SourceFacts::default();
        let sources: Vec<_> = (0..16)
            .map(|index| {
                cstr!("<template><div>{index}</div></template><script setup lang=\"ts\">{SCRIPT}</script><style>.n{index} {{}}</style>")
            })
            .collect();
        let start = std::sync::Barrier::new(sources.len());
        std::thread::scope(|scope| {
            for source in &sources {
                let (cache, start) = (&cache, &start);
                scope.spawn(move || {
                    let descriptor = vize_atelier_sfc::parse_sfc(source, Default::default())
                        .expect("component with a distinct template and style");
                    start.wait();
                    let facts = cache.for_descriptor(&descriptor);
                    assert!(facts.needs_project);
                    assert!(facts.importer_sensitive);
                });
            }
        });
        assert_eq!(cache.analyses.load(std::sync::atomic::Ordering::Relaxed), 1);
    }

    #[test]
    fn script_presence_language_source_and_attributes_are_distinct_inputs() {
        let source = cstr!("<script setup lang=\"ts\">{SCRIPT}</script>");
        let original = vize_atelier_sfc::parse_sfc(&source, Default::default()).unwrap();
        let cache = SourceFacts::default();
        assert!(cache.for_descriptor(&original).needs_project);

        let mut changed = original.clone();
        changed.script_setup.as_mut().unwrap().lang = Some("js".into());
        assert!(!cache.for_descriptor(&changed).needs_project);

        let mut changed = original.clone();
        changed.script = changed.script_setup.take();
        assert!(!cache.for_descriptor(&changed).needs_project);

        let mut changed = original.clone();
        changed.script_setup.as_mut().unwrap().src = Some("./script.ts".into());
        assert!(cache.for_descriptor(&changed).needs_project);

        let mut changed = original.clone();
        changed
            .script_setup
            .as_mut()
            .unwrap()
            .attrs
            .insert("generic".into(), "T extends object".into());
        assert!(cache.for_descriptor(&changed).needs_project);
        assert_eq!(cache.analyses.load(std::sync::atomic::Ordering::Relaxed), 5);
    }

    #[test]
    fn colliding_script_digests_still_compare_the_exact_source() {
        let imported = cstr!("<script setup lang=\"ts\">{SCRIPT}</script>");
        let inline = "<script setup lang=\"ts\">defineProps<{ label: string }>();</script>";
        let imported = vize_atelier_sfc::parse_sfc(&imported, Default::default()).unwrap();
        let inline = vize_atelier_sfc::parse_sfc(inline, Default::default()).unwrap();
        let cache = SourceFacts::default();
        let collision = [None, Some((0, 0))];
        assert!(
            cache
                .descriptor_with_key(&imported, collision)
                .needs_project
        );
        assert!(!cache.descriptor_with_key(&inline, collision).needs_project);
        assert!(
            cache
                .descriptor_with_key(&imported, collision)
                .needs_project
        );
        assert_eq!(cache.analyses.load(std::sync::atomic::Ordering::Relaxed), 2);
    }

    #[test]
    fn script_classification_keeps_its_result_when_key_storage_is_full() {
        let source = cstr!("<script setup lang=\"ts\">{SCRIPT}</script>");
        let descriptor = vize_atelier_sfc::parse_sfc(&source, Default::default()).unwrap();
        let cache = SourceFacts::default();
        cache.scripts.lock().unwrap().limit = 0;
        for _ in 0..2 {
            let facts = cache.for_descriptor(&descriptor);
            assert!(facts.needs_project);
            assert!(facts.importer_sensitive);
        }
        assert_eq!(cache.analyses.load(std::sync::atomic::Ordering::Relaxed), 2);
        assert!(cache.scripts.lock().unwrap().buckets.is_empty());
    }

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
