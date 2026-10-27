use super::{
    BindingOrigin, BindingRef, BindingTarget, ImportResolution, LocalSymbol, Namespace,
    OriginError, ProjectFacts, ProjectRevision, Resolver, SourceFacts, SourceRevision,
    SymbolIdentity, Usage, collect, seal,
};
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use std::sync::atomic::{AtomicUsize, Ordering};
use vize_carton::String;

fn revision(source: u32) -> SourceRevision {
    SourceRevision {
        owner: 1,
        source,
        version: 1,
    }
}
fn source(text: &str, id: u32) -> (SourceFacts, LocalSymbol) {
    source_as(text, id, SourceType::d_ts(), false)
}
fn source_as(
    text: &str,
    id: u32,
    source_type: SourceType,
    global_script: bool,
) -> (SourceFacts, LocalSymbol) {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, text, source_type).parse();
    assert!(!parsed.panicked);
    assert_eq!(parsed.diagnostics.len(), 0);
    let built = SemanticBuilder::new()
        .with_build_nodes(false)
        .build(&parsed.program);
    assert_eq!(built.diagnostics.len(), 0);
    assert!(
        built.semantic.nodes().is_empty(),
        "origin collection must not require the optional node store"
    );
    let symbol = built
        .semantic
        .scoping()
        .symbol_ids()
        .find(|symbol| built.semantic.scoping().symbol_name(*symbol) == "useTemplateRef")
        .unwrap();
    let local = LocalSymbol {
        symbol,
        namespace: Namespace::Value,
    };
    (
        collect(
            &parsed.program,
            &built.semantic,
            revision(id),
            global_script,
            &[],
        )
        .expect("the original Program and semantic snapshot are paired"),
        local,
    )
}
fn provider() -> SymbolIdentity {
    let (_, local) = source(
        "export declare function useTemplateRef<T = unknown>(key: string): Readonly<{ value: T | null }>;",
        900,
    );
    SymbolIdentity {
        source: revision(900),
        local,
    }
}
struct Imports {
    identity: SymbolIdentity,
    calls: AtomicUsize,
}
impl Resolver for Imports {
    fn resolve_import(
        &self,
        _from: SourceRevision,
        _module: &str,
        exported: &str,
        _namespace: Namespace,
        _usage: Usage,
    ) -> ImportResolution {
        self.calls.fetch_add(1, Ordering::Relaxed);
        assert_eq!(exported, "useTemplateRef");
        ImportResolution::External(self.identity)
    }
}
fn project<'a>(reference: SymbolIdentity) -> ProjectFacts<'a> {
    let mut project = ProjectFacts::new(ProjectRevision {
        owner: 1,
        generation: 10,
    });
    project.register_revision(reference.source).unwrap();
    project
}

#[test]
fn source_callable_shape_never_authorizes_vue_identity() {
    let text = "export {}; declare global { const useTemplateRef: <T = unknown, K extends string = string>(key: K) => Readonly<import('vue').ShallowRef<T | null>>; }";
    let (facts, local) = source(text, 1);
    let own = SymbolIdentity {
        source: facts.revision,
        local,
    };
    let reference = provider();
    let resolver = Imports {
        identity: reference,
        calls: AtomicUsize::new(0),
    };
    let mut project = project(reference);
    project.add_source(&facts).unwrap();
    let snapshot = seal(project, &resolver, reference, &["useTemplateRef"]).unwrap();
    assert_eq!(
        snapshot.get(
            "useTemplateRef",
            ProjectRevision {
                owner: 1,
                generation: 10
            }
        ),
        Ok(BindingOrigin::Custom(own))
    );
    assert_eq!(resolver.calls.load(Ordering::Relaxed), 0);
}

#[test]
fn actual_type_queries_and_type_only_aliases_retain_provider_identity() {
    for text in [
        "export {}; declare global { const useTemplateRef: typeof import('vue').useTemplateRef; }",
        "export {}; declare global { const useTemplateRef: typeof import('vue')['useTemplateRef']; }",
        "import type { useTemplateRef as VueRef } from 'vue'; type RefApi = typeof VueRef; declare global { const useTemplateRef: RefApi; }",
        "import type * as Vue from 'vue'; declare global { const useTemplateRef: typeof Vue.useTemplateRef; }",
    ] {
        let (facts, _) = source(text, 1);
        let reference = provider();
        let resolver = Imports {
            identity: reference,
            calls: AtomicUsize::new(0),
        };
        let mut project = project(reference);
        project.add_source(&facts).unwrap();
        let snapshot = seal(project, &resolver, reference, &["useTemplateRef"]).unwrap();
        assert_eq!(
            snapshot.get(
                "useTemplateRef",
                ProjectRevision {
                    owner: 1,
                    generation: 10
                }
            ),
            Ok(BindingOrigin::Vue)
        );
        assert_eq!(resolver.calls.load(Ordering::Relaxed), 1);
    }
}

#[test]
fn module_spelling_and_stale_provider_revisions_cannot_authorize_identity() {
    let (facts, _) = source(
        "export {}; declare global { const useTemplateRef: typeof import('vue').useTemplateRef; }",
        1,
    );
    let reference = provider();
    let custom = SymbolIdentity {
        source: revision(901),
        local: reference.local,
    };
    let mut project = project(reference);
    project.register_revision(custom.source).unwrap();
    project.add_source(&facts).unwrap();
    let resolver = Imports {
        identity: custom,
        calls: AtomicUsize::new(0),
    };
    let snapshot = seal(project, &resolver, reference, &["useTemplateRef"]).unwrap();
    assert_eq!(
        snapshot.get(
            "useTemplateRef",
            ProjectRevision {
                owner: 1,
                generation: 10
            }
        ),
        Ok(BindingOrigin::Custom(custom))
    );
    assert_eq!(
        snapshot.get(
            "useTemplateRef",
            ProjectRevision {
                owner: 1,
                generation: 11
            }
        ),
        Err(OriginError::StaleProjectRevision)
    );
}

#[test]
fn source_and_project_owners_are_explicit_snapshot_inputs() {
    let reference = provider();
    let mut project = project(reference);
    assert_eq!(
        project.register_revision(SourceRevision {
            owner: 2,
            ..revision(1)
        }),
        Err(OriginError::ForeignOwner)
    );
    assert_eq!(
        project.register_revision(SourceRevision {
            version: 2,
            ..reference.source
        }),
        Err(OriginError::ConflictingSourceRevision)
    );
}

#[test]
fn alias_cycles_do_not_require_a_recursion_limit_or_fabricate_an_origin() {
    let reference = provider();
    let (mut facts, local) = source(
        "export {}; declare global { const useTemplateRef: typeof import('vue').useTemplateRef; }",
        1,
    );
    let binding = BindingRef { source: 1, local };
    facts.bindings.insert(
        local,
        BindingTarget::Alias {
            binding,
            usage: Usage::TypeQuery,
        },
    );
    let mut project = project(reference);
    project.add_source(&facts).unwrap();
    let resolver = Imports {
        identity: reference,
        calls: AtomicUsize::new(0),
    };
    let snapshot = seal(project, &resolver, reference, &["useTemplateRef"]).unwrap();
    assert_eq!(
        snapshot.get(
            "useTemplateRef",
            ProjectRevision {
                owner: 1,
                generation: 10
            }
        ),
        Ok(BindingOrigin::Unresolved)
    );
    assert_eq!(resolver.calls.load(Ordering::Relaxed), 0);
}

#[test]
fn cyclic_export_evidence_can_reach_one_provider_without_guessing_missing_edges() {
    let reference = provider();
    let (mut facts, local) = source(
        "export {}; declare global { const useTemplateRef: typeof import('vue').useTemplateRef; }",
        1,
    );
    let binding = BindingRef { source: 1, local };
    facts.bindings.insert(
        local,
        BindingTarget::Candidates(vec![
            BindingTarget::Alias {
                binding,
                usage: Usage::TypeQuery,
            },
            BindingTarget::Import {
                module: String::from("vue"),
                exported: String::from("useTemplateRef"),
                namespace: Namespace::Value,
                type_only: false,
                usage: Some(Usage::TypeQuery),
            },
        ]),
    );
    let mut project = project(reference);
    project.add_source(&facts).unwrap();
    let resolver = Imports {
        identity: reference,
        calls: AtomicUsize::new(0),
    };
    let snapshot = seal(project, &resolver, reference, &["useTemplateRef"]).unwrap();
    assert_eq!(
        snapshot.get(
            "useTemplateRef",
            ProjectRevision {
                owner: 1,
                generation: 10
            }
        ),
        Ok(BindingOrigin::Vue)
    );
    assert_eq!(resolver.calls.load(Ordering::Relaxed), 1);
}

pub(super) fn observe(
    text: &str,
    source_type: SourceType,
    global_script: bool,
) -> (BindingOrigin, usize) {
    let (facts, _) = source_as(text, 1, source_type, global_script);
    let reference = provider();
    let resolver = Imports {
        identity: reference,
        calls: AtomicUsize::new(0),
    };
    let mut project = project(reference);
    project.add_source(&facts).unwrap();
    let snapshot = seal(project, &resolver, reference, &["useTemplateRef"]).unwrap();
    (
        snapshot
            .get(
                "useTemplateRef",
                ProjectRevision {
                    owner: 1,
                    generation: 10,
                },
            )
            .unwrap(),
        resolver.calls.load(Ordering::Relaxed),
    )
}
