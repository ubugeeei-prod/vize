//! P1-11 resident-cache reset scenario for the batch's stats cache.
//!
//! The batch pool resets one arena per worker between files, so the cache is
//! the structure most exposed to a lifetime mistake: it is populated during
//! file N's compile and read long after that arena has been reset and refilled
//! by other files. These tests pin that the entries are owned — once by
//! construction (`'static`), once by running the scenario against the real
//! compiler and comparing the read-back exactly.

use vize_atelier_sfc::{SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};
use vize_s0::hash::hash_str;

use super::{StatsCompileCache, StatsCompileCacheEntry, StatsCompileCacheKey};

const CACHED_FILE: &str = r#"<template>
  <button class="counter" @click="increment">{{ label }}: {{ count }}</button>
</template>

<script setup>
import { ref } from 'vue'
const count = ref(0)
const label = ref('clicks')
function increment() { count.value += 1 }
</script>
"#;

const OTHER_FILE: &str = r#"<template>
  <section><p v-for="row in rows" :key="row">{{ row }}</p></section>
</template>

<script setup>
import { ref } from 'vue'
const rows = ref(['a', 'b', 'c'])
</script>
"#;

/// Compiles a source the way the stats path does and returns the numbers it
/// caches. Each call takes this worker's pooled arena and returns it — reset —
/// before the numbers come back.
fn stats_facts(source: &str) -> (usize, usize, usize, usize) {
    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("fixture must parse");
    let template_size = descriptor.template.as_ref().map_or(0, |t| t.content.len());
    let script_size = descriptor.script.as_ref().map_or(0, |s| s.content.len())
        + descriptor
            .script_setup
            .as_ref()
            .map_or(0, |s| s.content.len());
    let style_count = descriptor.styles.len();
    let result = compile_sfc(&descriptor, SfcCompileOptions::default()).expect("must compile");
    (result.code.len(), template_size, script_size, style_count)
}

fn key_for(source: &str) -> StatsCompileCacheKey {
    StatsCompileCacheKey {
        source_hash: hash_str(source),
        source_len: source.len(),
        component_name_len: 0,
        settings: 0,
        custom_elements_hash: 0,
    }
}

fn success_entry(facts: (usize, usize, usize, usize)) -> StatsCompileCacheEntry {
    StatsCompileCacheEntry::Success {
        output_bytes: facts.0,
        template_size: facts.1,
        script_size: facts.2,
        style_count: facts.3,
    }
}

fn success_facts(entry: &StatsCompileCacheEntry) -> (usize, usize, usize, usize) {
    match entry {
        StatsCompileCacheEntry::Success {
            output_bytes,
            template_size,
            script_size,
            style_count,
        } => (*output_bytes, *template_size, *script_size, *style_count),
        StatsCompileCacheEntry::Failure { .. } => {
            unreachable!("the fixture compiles, so its entry is a success")
        }
    }
}

/// Cache populated → arena reset (many times) → cache read.
#[test]
fn cached_stats_survive_the_arena_that_produced_them() {
    let cache = StatsCompileCache::default();
    let key = key_for(CACHED_FILE);
    let facts = stats_facts(CACHED_FILE);

    cache
        .entries
        .lock()
        .expect("fresh cache mutex is not poisoned")
        .insert(key, success_entry(facts));

    // Each of these acquires the same pooled arena, resets it, and overwrites
    // the bytes the cached compile used.
    for _ in 0..64 {
        let _ = stats_facts(OTHER_FILE);
    }

    let read = cache
        .entries
        .lock()
        .expect("fresh cache mutex is not poisoned")
        .get(&key)
        .cloned()
        .expect("the entry stays in the cache for the whole batch");
    assert_eq!(success_facts(&read), facts);
    // And the entry still describes the file: recompiling reproduces it.
    assert_eq!(success_facts(&read), stats_facts(CACHED_FILE));
    // The compile that filled it returned its arena before the entry was
    // stored, which is what the per-file assertion in `compile_stats` reads.
    assert_eq!(vize_s0::pool::checked_out(), 0);
}

/// Type-level half of the arena/cache contract: a cache entry that borrowed
/// arena bytes could not be `'static`, so this stops compiling if one ever
/// does.
#[test]
fn cache_entries_cannot_borrow_an_arena() {
    fn assert_owned<T: 'static>() {}

    assert_owned::<StatsCompileCacheKey>();
    assert_owned::<StatsCompileCacheEntry>();
    assert_owned::<StatsCompileCache>();
    assert_owned::<crate::commands::build::config::CompileOutput>();
    assert_owned::<crate::commands::build::config::CompileError>();
    assert_owned::<crate::commands::build::config::FileProfile>();
}
