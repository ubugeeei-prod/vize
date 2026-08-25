//! P1-11: the batch pool's escape checks, at the level a batch actually runs.
//!
//! Under the pool one arena serves file after file on a worker, so the bytes
//! that held file N's tree are handed to file N+1. Two things have to hold for
//! that to be a memory win rather than a correctness bug:
//!
//! 1. **Nothing survives the reset.** Everything a compile keeps — the
//!    generated code, diagnostics, the numbers a resident cache stores — is in
//!    its owned form before the arena goes back to the pool. The scenario test
//!    below populates a cache from one compile, runs enough further compiles on
//!    the same worker that the arena is reset and overwritten many times over,
//!    and then reads the cache back: owned data compares equal, a window into
//!    recycled arena bytes would not.
//! 2. **Reuse is invisible in the output.** A hot arena must produce the same
//!    bytes as a cold one, which is what `identical_output_*` pins: a compile
//!    on a worker whose pool has never been touched against the same compile on
//!    a worker that has already run the batch.
//!
//! The borrow checker is the primary enforcement (`Allocator::reset` takes
//! `&mut self` and the pool guard owns its arena, so an arena-backed value
//! that outlived a file is a compile error). These tests cover what it cannot
//! see: the pool's own bookkeeping and the owned-form contract.

use vize_atelier_sfc::{
    SfcCompileOptions, SfcCompileResult, SfcParseOptions, compile_sfc, parse_sfc,
};
use vize_carton::{String, pool};

/// How many other files run between populating a cache and reading it back.
///
/// Miri runs this file too (it is the only lane that reports use-after-free on
/// a recycled arena for real), and it interprets every instruction, so the
/// scenario shrinks to the smallest count that still resets the arena and
/// refills it with different bytes.
const INTERLEAVED_FILES: usize = if cfg!(miri) { 2 } else { 64 };

const SCRIPT_SETUP: &str = r#"<template>
  <ul class="list">
    <li v-for="item in items" :key="item.id" @click="select(item)">
      {{ item.label }} — {{ formatted(item) }}
    </li>
  </ul>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'

const items = ref([{ id: 1, label: 'one' }])
const active = ref<number | null>(null)
const formatted = computed(() => (item: { label: string }) => item.label.toUpperCase())

function select(item: { id: number }) {
  active.value = item.id
}
</script>

<style scoped>
.list { padding: 8px; }
</style>
"#;

const OTHER_FILE: &str = r#"<template>
  <section>
    <h1 :class="headingClass">{{ heading }}</h1>
    <p v-if="body">{{ body }}</p>
  </section>
</template>

<script setup>
import { ref } from 'vue'
const heading = ref('other file')
const headingClass = ref('title')
const body = ref('')
</script>
"#;

/// What a resident cache keeps for a compiled file: owned values only, the
/// same shape the CLI's `--format stats` cache stores.
#[derive(Clone, Debug, PartialEq, Eq)]
struct CachedFacts {
    code_len: usize,
    css: Option<String>,
    code: String,
}

fn compile(source: &str) -> SfcCompileResult {
    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("fixture must parse");
    compile_sfc(&descriptor, SfcCompileOptions::default()).expect("fixture must compile")
}

fn facts(source: &str) -> CachedFacts {
    let result = compile(source);
    CachedFacts {
        code_len: result.code.len(),
        css: result.css.clone(),
        code: result.code,
    }
}

/// The resident-cache reset scenario: populate, reset the arena many times
/// over by compiling other files on the same worker, then read the cache.
#[test]
fn cached_facts_survive_the_arena_that_produced_them() {
    let cache = [("SCRIPT_SETUP", facts(SCRIPT_SETUP))];
    let expected = cache[0].1.clone();

    // Every one of these acquires the same pooled arena this worker used for
    // the cached compile, resets it, and fills it with different bytes.
    for _ in 0..INTERLEAVED_FILES {
        let _ = compile(OTHER_FILE);
    }

    let (name, cached) = &cache[0];
    assert_eq!(*name, "SCRIPT_SETUP");
    assert_eq!(*cached, expected);
    // And the cached numbers still describe the file: recompiling it now
    // reproduces them exactly.
    assert_eq!(*cached, facts(SCRIPT_SETUP));
}

/// A compile returns its arena before its result crosses the file boundary,
/// and the arena stays on the worker for the next file instead of being freed.
#[test]
fn a_compile_returns_its_arena_to_the_worker_pool() {
    pool::clear();
    assert_eq!((pool::checked_out(), pool::idle()), (0, 0));

    let _ = compile(SCRIPT_SETUP);
    assert_eq!(pool::checked_out(), 0);
    assert_ne!(pool::idle(), 0);

    let idle_after_first = pool::idle();
    let _ = compile(OTHER_FILE);
    // Steady state: the second file reuses what the first left behind.
    assert_eq!((pool::checked_out(), pool::idle()), (0, idle_after_first));
}

/// STRICT zero-output-byte-change: a hot pool compiles to the same bytes as a
/// cold one. The comparison runs the cold side on a fresh thread, whose pool
/// starts empty — a brand-new arena, exactly what the pre-P1-11 per-file
/// `Allocator::new()` produced.
#[test]
fn identical_output_from_a_cold_and_a_hot_arena() {
    let cold = std::thread::spawn(|| {
        // Nothing has run on this worker, so its pool is empty and the compile
        // below builds its arena from scratch.
        assert_eq!((pool::checked_out(), pool::idle()), (0, 0));
        facts(SCRIPT_SETUP)
    })
    .join()
    .expect("the cold worker runs to completion");

    for _ in 0..INTERLEAVED_FILES / 2 {
        let _ = compile(OTHER_FILE);
    }
    let hot = facts(SCRIPT_SETUP);
    assert_eq!(hot, cold);
}

/// Type-level half of the contract: nothing a compile hands back can borrow
/// an arena. `'static` is exactly that property — a value holding an `&'a`
/// into arena bytes would not satisfy it, and this stops compiling.
#[test]
fn compile_results_are_owned() {
    fn assert_owned<T: 'static>() {}

    assert_owned::<SfcCompileResult>();
    assert_owned::<CachedFacts>();
    assert_owned::<vize_atelier_sfc::SfcError>();
}
