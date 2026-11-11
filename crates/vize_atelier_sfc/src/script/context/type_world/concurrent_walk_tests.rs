use super::super::{ScriptCompileContext, TypeSourceSnapshot};
use std::path::Path;
use std::sync::{Arc, Barrier};
use vize_carton::cstr;
use vize_croquis::types::{ResolvedTypeWorld, TypeLookup};

/// A barrel chain seven levels deep, eight models per level: every walk
/// reaches the same 64 modules in the same order.
fn write_library(dir: &Path) {
    std::fs::write(dir.join("api.ts"), "export * from './l0';\n").unwrap();
    for level in 0..7 {
        let mut barrel = vize_carton::String::default();
        for model in 0..8 {
            std::fs::write(
                dir.join(cstr!("m{level}_{model}.ts").as_str()),
                cstr!(
                    "import type {{ Ref }} from './shared'; export interface Model{level}_{model} {{ id: Ref<{level}> }}"
                )
                .as_str(),
            )
            .unwrap();
            barrel.push_str(&cstr!("export * from './m{level}_{model}';\n"));
        }
        if level < 6 {
            barrel.push_str(&cstr!("export * from './l{}';\n", level + 1));
        }
        std::fs::write(dir.join(cstr!("l{level}.ts").as_str()), barrel.as_str()).unwrap();
    }
    std::fs::write(dir.join("shared.ts"), "export type Ref<T> = { value: T }").unwrap();
}

fn world(dir: &Path, index: usize, sources: &TypeSourceSnapshot) -> ResolvedTypeWorld {
    let level = index % 7;
    let source = cstr!(
        "import type {{ Model{level}_{index} }} from './api'; defineProps<{{ item: Model{level}_{index} }}>()",
        index = index % 8,
    );
    ScriptCompileContext::new(&source).resolve_type_world_with_sources(
        dir.join(cstr!("Host{index}.vue").as_str())
            .to_str()
            .unwrap(),
        None,
        false,
        sources,
    )
}

fn dependency_signature(world: &ResolvedTypeWorld) -> Vec<vize_carton::String> {
    let mut rows: Vec<_> = world
        .modules
        .iter()
        .filter(|(key, _)| **key != world.root_module)
        .map(|(key, module)| cstr!("{key}: {module:?}"))
        .collect();
    rows.sort();
    rows
}

#[test]
fn concurrent_walks_resolve_each_dependency_once_into_the_sequential_worlds() {
    let dir = tempfile::tempdir().unwrap();
    write_library(dir.path());
    let walks = 16;
    for _ in 0..8 {
        let sources = TypeSourceSnapshot::default();
        let start = Barrier::new(walks);
        let worlds: Vec<_> = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..walks)
                .map(|index| {
                    let (sources, start, dir) = (&sources, &start, dir.path());
                    scope.spawn(move || {
                        start.wait();
                        world(dir, index, sources)
                    })
                })
                .collect();
            handles
                .into_iter()
                .map(|handle| handle.join().unwrap())
                .collect()
        });

        // Walks that took up a module another walk had queued built the
        // worlds a lone walk builds, each module resolved once.
        let resolved = sources.resolved.lock().unwrap();
        assert_eq!(resolved.len(), 65);
        assert!(
            resolved
                .values()
                .all(|slot| slot.get().is_some_and(Option::is_some)),
            "every queued module is resolved by the walk that visits it"
        );
        drop(resolved);
        for (index, concurrent) in worlds.iter().enumerate() {
            let alone = world(dir.path(), index, &TypeSourceSnapshot::default());
            assert_eq!(concurrent.modules.len(), 66);
            assert_eq!(
                dependency_signature(concurrent),
                dependency_signature(&alone)
            );
            let name = cstr!("Model{}_{}", index % 7, index % 8);
            assert!(matches!(
                concurrent.resolve(&concurrent.root_module, &name),
                TypeLookup::Found(_)
            ));
            for (key, module) in &worlds[0].modules {
                if *key != worlds[0].root_module {
                    assert!(Arc::ptr_eq(module, &concurrent.modules[key]), "{key}");
                }
            }
        }
    }
}
