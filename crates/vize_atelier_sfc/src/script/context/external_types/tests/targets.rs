//! The generated module graphs of `differential`, imported by components that
//! compile for several targets at once: each target resolves the same types
//! and reports the same errors as a compile of that target alone.

use super::super::work::{self, Work};
use super::differential::{Rng, edges, name, write_module};
use super::sharing::write_reproduction;
use crate::compile::targets_tests::{
    BUILDS, DOM_INLINE, SSR, TARGET_LISTS, assert_targets_alike, compile_alone, compile_together,
};
use crate::types::SfcParseOptions;

/// The imported-type work of compiling each component of the reporter's
/// reproduction for a client and a server target: one call per target, then
/// one call for both. Returns `(apart, together)`.
fn reproduction_work(components: usize, modules: usize) -> (Work, Work) {
    let dir = tempfile::tempdir().unwrap();
    let reproduction = write_reproduction(dir.path(), components, modules);
    let sources: Vec<_> = reproduction
        .components
        .iter()
        .map(|(path, _)| {
            let path = path.to_string_lossy().into_owned();
            let source = std::fs::read_to_string(&path).unwrap();
            (path, source)
        })
        .collect();
    let build = BUILDS[1];
    let targets = [DOM_INLINE, SSR];
    let descriptors: Vec<_> = sources
        .iter()
        .map(|(path, source)| {
            let parse = SfcParseOptions {
                filename: path.as_str().into(),
                ..Default::default()
            };
            (path, crate::parse::parse_sfc(source, parse).unwrap())
        })
        .collect();
    work::take();
    for (path, descriptor) in &descriptors {
        for target in &targets {
            assert!(compile_alone(descriptor, path, build, target).is_ok());
        }
    }
    let apart = work::take();
    for (path, descriptor) in &descriptors {
        let compiled = compile_together(descriptor, path, build, &targets);
        assert!(compiled.dependencies_shared, "{path}");
    }
    (apart, work::take())
}

/// One call for both targets resolves and merges a component's imported
/// types once; two calls do it twice. Only the per-target emit repeats.
#[test]
fn two_targets_resolve_and_merge_imported_types_once() {
    for (components, modules) in [(1, 1), (6, 25), (3, 100)] {
        let (apart, together) = reproduction_work(components, modules);
        assert_eq!(apart.declarations, 2 * components * (modules + 1));
        assert_eq!(together.declarations, components * (modules + 1));
        assert!(together.resolutions > 0);
        assert_eq!(apart.resolutions, 2 * together.resolutions);
        assert_eq!(apart.modules, 2 * together.modules);
        assert_eq!(apart.dependencies, 2 * together.dependencies);
    }
}

/// The work table for the reporter's shapes, for a pull request description:
/// `cargo test -p vize_atelier_sfc --lib -- --ignored --nocapture
/// two_targets_work_table`.
#[test]
#[ignore = "prints the work of the reporter's full shapes"]
fn two_targets_work_table() {
    println!(
        "| components | barrel | calls | resolutions | modules walked | declarations merged | dependency names |"
    );
    println!("| --: | --: | :-- | --: | --: | --: | --: |");
    for (components, modules) in [(1000, 200), (1000, 800)] {
        let (apart, together) = reproduction_work(components, modules);
        for (calls, work) in [("one per target", apart), ("one for both", together)] {
            println!(
                "| {components} | {modules} | {calls} | {} | {} | {} | {} |",
                work.resolutions, work.modules, work.declarations, work.dependencies
            );
        }
    }
}

/// A component typing its props through the type imports of `edges`; the
/// re-exports a module may have are not allowed in `<script setup>`.
fn component(rng: &mut Rng, modules: usize) -> std::string::String {
    let mut imports = std::string::String::new();
    edges(rng, modules, &mut imports);
    let imports: std::string::String = imports
        .lines()
        .filter(|line| line.starts_with("import"))
        .flat_map(|line| [line, "\n"])
        .collect();
    let props = match rng.below(3) {
        0 => format!("{{ local: string }} & {}", name(rng)),
        1 => name(rng).to_owned(),
        _ => "{ local: string }".to_owned(),
    };
    format!(
        "<script setup lang=\"ts\">\n{imports}const props = defineProps<{props}>()\n</script>\n\n<template><p>{{{{ props }}}}</p></template>\n"
    )
}

#[test]
fn every_target_resolves_the_generated_graphs_as_it_does_alone() {
    let mut compiled = 0;
    for seed in 1..=160_u64 {
        let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15));
        let dir = tempfile::tempdir().unwrap();
        let dir = dir.path().canonicalize().unwrap();
        let modules = 2 + rng.below(9);
        for index in 0..modules {
            write_module(&mut rng, &dir, index, modules);
        }
        for index in 0..4 {
            let source = component(&mut rng, modules);
            let path = dir.join(format!("Component{index}.vue"));
            std::fs::write(&path, &source).unwrap();
            let path = path.to_string_lossy();
            let build = BUILDS[index % BUILDS.len()];
            let targets = TARGET_LISTS[(seed as usize + index) % TARGET_LISTS.len()];
            compiled += assert_targets_alike(&path, &source, build, targets);
        }
    }
    assert!(compiled > 160, "{compiled} compiled");
}
