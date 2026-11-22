use super::*;
use crate::module_paths::normalize_logical_path;
use std::{fs, path::PathBuf};

/// Register each original caller and the real Vue targets available in the same fixture manifest.
pub(super) fn requests(
    mut visit: impl FnMut(&Path, &ModuleRegistry, &SetupRenderInput<'_, '_, '_>),
) {
    let manifest =
        fs::read_to_string(std::env::var_os("VIZE_EXPRESSION_QUERY_MANIFEST").unwrap()).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let originals: Vec<_> = manifest
        .split_terminator('\0')
        .map(|name| {
            let physical = normalize_logical_path(root.join(name));
            let logical = if physical.extension().is_some_and(|ext| ext == "txt") {
                physical.with_extension("")
            } else {
                physical.clone()
            };
            (logical, physical)
        })
        .collect();
    let mut available = FxHashMap::<&Path, &Path>::default();
    for (logical, physical) in &originals {
        if logical == physical || !available.contains_key(logical.as_path()) {
            available.insert(logical, physical);
        }
    }
    let (mut invalid, mut refused, mut callers, mut registered) = (0, 0, 0, 0);
    for (caller, input) in &originals {
        let bytes = fs::read(input).unwrap();
        let Ok(source) = std::str::from_utf8(&bytes) else {
            invalid += 1;
            continue;
        };
        let arena = Allocator::new();
        let owner = observe(&arena, source);
        let Ok(imports) = owner.setup_component_imports() else {
            refused += 1;
            continue;
        };
        callers += 1;
        let mut registry = ModuleRegistry::new();
        let (_, input) = registry
            .register_setup_caller(caller, analysis(caller, source), &imports)
            .unwrap();
        let mut targets = FxHashSet::<PathBuf>::default();
        for site in imports.sites() {
            for target in crate::logical_import_candidates(site.module(), caller.parent()) {
                if target != *caller
                    && let Some(physical) = available.get(target.as_path())
                    && targets.insert(target.clone())
                {
                    let target_bytes = fs::read(physical).unwrap();
                    if let Ok(target_source) = std::str::from_utf8(&target_bytes) {
                        register(&mut registry, &target, target_source);
                        registered += 1;
                    }
                }
            }
        }
        visit(caller, &registry, &input);
    }
    assert_eq!(originals.len(), invalid + refused + callers);
    assert!(callers > 0);
    eprintln!(
        "setup_render_corpus files={} invalid={invalid} unavailable={refused} callers={callers} registered_targets={registered}",
        originals.len()
    );
}

#[test]
#[ignore = "original Vue callers and project import targets"]
fn original_project_results() {
    let (mut uses, mut edges, mut sites, mut targets) = (0, 0, 0, 0);
    requests(|path, registry, imports| {
        let actual = registry
            .setup_render_products(path, imports)
            .unwrap_or_else(|reason| panic!("{path:?}: {reason:?}"));
        assert_eq!(actual, original(registry, path), "{path:?}");
        uses += imports.imports.sites().len();
        edges += actual.edges.len();
        sites += actual.edges.values().map(Vec::len).sum::<usize>();
        targets += actual.targets.len();
    });
    assert!(uses > 0);
    eprintln!(
        "setup_render_results uses={uses} edges={edges} sites={sites} ordered_targets={targets}"
    );
}
