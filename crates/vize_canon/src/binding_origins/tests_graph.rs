use super::{
    BindingOrigin, BindingRef, BindingTarget, ImportResolution, LocalSymbol, Namespace,
    OriginSnapshot, ProjectFacts, ProjectRevision, Resolver, SourceFacts, SourceRevision, SymbolId,
    SymbolIdentity, Usage,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use vize_carton::{FxHashMap, String};

fn version(source: u32) -> SourceRevision {
    SourceRevision {
        owner: 1,
        source,
        version: 1,
    }
}
fn local() -> LocalSymbol {
    LocalSymbol {
        symbol: SymbolId::new(0),
        namespace: Namespace::Value,
    }
}
fn identity(source: u32) -> SymbolIdentity {
    SymbolIdentity {
        source: version(source),
        local: local(),
    }
}
fn project<'a>() -> ProjectFacts<'a> {
    let mut facts = ProjectFacts::new(ProjectRevision {
        owner: 1,
        generation: 1,
    });
    facts.register_revision(version(1_000_000)).unwrap();
    facts
}
fn origin(snapshot: &OriginSnapshot) -> BindingOrigin {
    snapshot
        .get(
            "useTemplateRef",
            ProjectRevision {
                owner: 1,
                generation: 1,
            },
        )
        .unwrap()
}
fn source(source: u32, target: BindingTarget, global: bool) -> SourceFacts {
    let mut bindings = FxHashMap::default();
    bindings.insert(local(), target);
    SourceFacts {
        revision: version(source),
        bindings,
        globals: if global {
            vec![(String::from("useTemplateRef"), local())]
        } else {
            Vec::new()
        },
    }
}
struct Imports {
    identity: SymbolIdentity,
    calls: AtomicUsize,
}
impl Resolver for Imports {
    fn resolve_import(
        &self,
        _: SourceRevision,
        _: &str,
        _: &str,
        _: Namespace,
        _: Usage,
    ) -> ImportResolution {
        self.calls.fetch_add(1, Ordering::Relaxed);
        ImportResolution::External(self.identity)
    }
}
fn imported() -> BindingTarget {
    BindingTarget::Import {
        module: String::from("vue"),
        exported: String::from("useTemplateRef"),
        namespace: Namespace::Value,
        type_only: false,
        usage: Some(Usage::TypeQuery),
    }
}
#[test]
fn twenty_thousand_aliases_expand_without_recursion_or_repeated_resolution() {
    let sources = (1..=20_000)
        .map(|id| {
            source(
                id,
                if id == 20_000 {
                    imported()
                } else {
                    BindingTarget::Alias {
                        binding: BindingRef {
                            source: id + 1,
                            local: local(),
                        },
                        usage: Usage::TypeQuery,
                    }
                },
                id == 1,
            )
        })
        .collect::<Vec<_>>();
    let mut facts = project();
    for source in &sources {
        facts.add_source(source).unwrap();
    }
    let resolver = Imports {
        identity: identity(1_000_000),
        calls: AtomicUsize::new(0),
    };
    let snapshot = super::seal(facts, &resolver, identity(1_000_000), &["useTemplateRef"]).unwrap();
    assert_eq!(origin(&snapshot), BindingOrigin::Vue);
    assert_eq!(resolver.calls.load(Ordering::Relaxed), 1);
}
#[test]
fn distinct_global_declarations_keep_ambiguous_ownership() {
    let first = source(1, imported(), true);
    let second = source(2, BindingTarget::Opaque, true);
    let mut facts = project();
    facts.add_source(&first).unwrap();
    facts.add_source(&second).unwrap();
    let resolver = Imports {
        identity: identity(1_000_000),
        calls: AtomicUsize::new(0),
    };
    let snapshot = super::seal(facts, &resolver, identity(1_000_000), &["useTemplateRef"]).unwrap();
    assert_eq!(origin(&snapshot), BindingOrigin::Ambiguous);
}
#[test]
fn stale_or_foreign_resolver_images_never_authorize_a_provider() {
    for revision in [
        SourceRevision {
            version: 2,
            ..version(1_000_000)
        },
        SourceRevision {
            owner: 2,
            ..version(1_000_000)
        },
    ] {
        let input = source(1, imported(), true);
        let mut facts = project();
        facts.add_source(&input).unwrap();
        let resolver = Imports {
            identity: SymbolIdentity {
                source: revision,
                local: local(),
            },
            calls: AtomicUsize::new(0),
        };
        let snapshot =
            super::seal(facts, &resolver, identity(1_000_000), &["useTemplateRef"]).unwrap();
        assert_eq!(origin(&snapshot), BindingOrigin::Unresolved);
    }
}
#[test]
fn type_only_runtime_values_do_not_become_callable_providers() {
    let mut target = imported();
    if let BindingTarget::Import {
        type_only, usage, ..
    } = &mut target
    {
        *type_only = true;
        *usage = Some(Usage::Runtime);
    }
    let input = source(1, target, true);
    let mut facts = project();
    facts.add_source(&input).unwrap();
    let resolver = Imports {
        identity: identity(1_000_000),
        calls: AtomicUsize::new(0),
    };
    let snapshot = super::seal(facts, &resolver, identity(1_000_000), &["useTemplateRef"]).unwrap();
    assert_eq!(origin(&snapshot), BindingOrigin::Unresolved);
    assert_eq!(resolver.calls.load(Ordering::Relaxed), 0);
}
#[test]
fn requested_roots_do_not_resolve_unrelated_global_values() {
    let mut bindings = FxHashMap::default();
    let mut globals = Vec::new();
    for index in 0..10_000 {
        let local = LocalSymbol {
            symbol: SymbolId::new(index),
            namespace: Namespace::Value,
        };
        let name = if index == 0 {
            String::from("useTemplateRef")
        } else {
            String::from(format!("unrelated{index}"))
        };
        bindings.insert(local, imported());
        globals.push((name, local));
    }
    let input = SourceFacts {
        revision: version(1),
        bindings,
        globals,
    };
    let mut facts = project();
    facts.add_source(&input).unwrap();
    let resolver = Imports {
        identity: identity(1_000_000),
        calls: AtomicUsize::new(0),
    };
    let snapshot = super::seal(facts, &resolver, identity(1_000_000), &["useTemplateRef"]).unwrap();
    assert_eq!(origin(&snapshot), BindingOrigin::Vue);
    assert_eq!(resolver.calls.load(Ordering::Relaxed), 1);
    assert_eq!(
        snapshot
            .get(
                "unrelated1",
                ProjectRevision {
                    owner: 1,
                    generation: 1
                }
            )
            .unwrap(),
        BindingOrigin::Unresolved
    );
}

struct ContextImports {
    calls: AtomicUsize,
}
impl Resolver for ContextImports {
    fn resolve_import(
        &self,
        from: SourceRevision,
        _: &str,
        _: &str,
        _: Namespace,
        usage: Usage,
    ) -> ImportResolution {
        self.calls.fetch_add(1, Ordering::Relaxed);
        ImportResolution::External(identity(if usage == Usage::TypeQuery || from.source == 2 {
            2_000_000
        } else {
            1_000_000
        }))
    }
}

#[test]
fn import_memoization_preserves_distinct_effective_usage() {
    let mut target = imported();
    if let BindingTarget::Import { usage, .. } = &mut target {
        *usage = None;
    }
    let imported_source = source(3, target, false);
    let root = source(
        1,
        BindingTarget::Candidates(
            [Usage::Runtime, Usage::TypeQuery]
                .into_iter()
                .map(|usage| BindingTarget::Alias {
                    binding: BindingRef {
                        source: 3,
                        local: local(),
                    },
                    usage,
                })
                .collect(),
        ),
        true,
    );
    let mut facts = project();
    facts.register_revision(version(2_000_000)).unwrap();
    facts.add_source(&root).unwrap();
    facts.add_source(&imported_source).unwrap();
    let resolver = ContextImports {
        calls: AtomicUsize::new(0),
    };
    let snapshot = super::seal(facts, &resolver, identity(1_000_000), &["useTemplateRef"]).unwrap();
    assert_eq!(origin(&snapshot), BindingOrigin::Ambiguous);
    assert_eq!(resolver.calls.load(Ordering::Relaxed), 2);
}

#[test]
fn import_memoization_preserves_distinct_source_owners() {
    let mut target = imported();
    if let BindingTarget::Import { usage, .. } = &mut target {
        *usage = None;
    }
    let first = source(1, target.clone(), true);
    let second = source(2, target, true);
    let mut facts = project();
    facts.register_revision(version(2_000_000)).unwrap();
    facts.add_source(&first).unwrap();
    facts.add_source(&second).unwrap();
    let resolver = ContextImports {
        calls: AtomicUsize::new(0),
    };
    let snapshot = super::seal(facts, &resolver, identity(1_000_000), &["useTemplateRef"]).unwrap();
    assert_eq!(origin(&snapshot), BindingOrigin::Ambiguous);
    assert_eq!(resolver.calls.load(Ordering::Relaxed), 2);
}
