use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

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
fn project() -> ProjectFacts {
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
fn add(project: &mut ProjectFacts, source: u32, target: BindingTarget, global: bool) {
    let mut bindings = FxHashMap::default();
    bindings.insert(local(), target);
    project
        .add_source(Arc::new(SourceFacts {
            revision: version(source),
            bindings,
            globals: if global {
                vec![(String::from("useTemplateRef"), local())]
            } else {
                Vec::new()
            },
        }))
        .unwrap();
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
    let mut facts = project();
    for source in 1..=20_000 {
        add(
            &mut facts,
            source,
            if source == 20_000 {
                imported()
            } else {
                BindingTarget::Alias {
                    binding: BindingRef {
                        source: source + 1,
                        local: local(),
                    },
                    usage: Usage::TypeQuery,
                }
            },
            source == 1,
        );
    }
    let resolver = Imports {
        identity: identity(1_000_000),
        calls: AtomicUsize::new(0),
    };
    let snapshot = seal(facts, &resolver, identity(1_000_000), &["useTemplateRef"]).unwrap();
    assert_eq!(origin(&snapshot), BindingOrigin::Vue);
    assert_eq!(resolver.calls.load(Ordering::Relaxed), 1);
}
#[test]
fn distinct_global_declarations_keep_ambiguous_ownership() {
    let mut facts = project();
    add(&mut facts, 1, imported(), true);
    add(&mut facts, 2, BindingTarget::Opaque, true);
    let resolver = Imports {
        identity: identity(1_000_000),
        calls: AtomicUsize::new(0),
    };
    let snapshot = seal(facts, &resolver, identity(1_000_000), &["useTemplateRef"]).unwrap();
    assert_eq!(origin(&snapshot), BindingOrigin::Ambiguous);
}
#[test]
fn stale_or_foreign_resolver_images_never_authorize_a_provider() {
    for source in [
        SourceRevision {
            version: 2,
            ..version(1_000_000)
        },
        SourceRevision {
            owner: 2,
            ..version(1_000_000)
        },
    ] {
        let mut facts = project();
        add(&mut facts, 1, imported(), true);
        let resolver = Imports {
            identity: SymbolIdentity {
                source,
                local: local(),
            },
            calls: AtomicUsize::new(0),
        };
        let snapshot = seal(facts, &resolver, identity(1_000_000), &["useTemplateRef"]).unwrap();
        assert_eq!(origin(&snapshot), BindingOrigin::Unresolved);
    }
}
#[test]
fn type_only_runtime_values_do_not_become_callable_providers() {
    let mut facts = project();
    let mut target = imported();
    if let BindingTarget::Import {
        type_only, usage, ..
    } = &mut target
    {
        *type_only = true;
        *usage = Some(Usage::Runtime);
    }
    add(&mut facts, 1, target, true);
    let resolver = Imports {
        identity: identity(1_000_000),
        calls: AtomicUsize::new(0),
    };
    let snapshot = seal(facts, &resolver, identity(1_000_000), &["useTemplateRef"]).unwrap();
    assert_eq!(origin(&snapshot), BindingOrigin::Unresolved);
    assert_eq!(resolver.calls.load(Ordering::Relaxed), 0);
}

#[test]
fn requested_roots_do_not_resolve_unrelated_global_values() {
    let mut facts = project();
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
    facts
        .add_source(Arc::new(SourceFacts {
            revision: version(1),
            bindings,
            globals,
        }))
        .unwrap();
    let resolver = Imports {
        identity: identity(1_000_000),
        calls: AtomicUsize::new(0),
    };
    let snapshot = seal(facts, &resolver, identity(1_000_000), &["useTemplateRef"]).unwrap();
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
