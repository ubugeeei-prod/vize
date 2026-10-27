//! Original-source coverage and bounded lowering; no host origin admission.
use super::{
    BindingTarget, SourceFacts, SourceRevision, collect_requested_globals, tests::observe,
};
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use vize_carton::{String, append};

fn capture(text: &str, global_script: bool, names: &[&str]) -> SourceFacts {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, text, SourceType::ts()).parse();
    assert!(!parsed.panicked);
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let built = SemanticBuilder::new()
        .with_build_nodes(false)
        .build(&parsed.program);
    assert!(built.diagnostics.is_empty(), "{:?}", built.diagnostics);
    assert!(built.semantic.nodes().is_empty());
    collect_requested_globals(
        &parsed.program,
        &built.semantic,
        SourceRevision {
            owner: 1,
            source: 1,
            version: 1,
        },
        global_script,
        &[],
        names,
    )
    .expect("the original Program and semantic snapshot are paired")
}

#[test]
fn requested_global_names_do_not_lower_ten_thousand_unrelated_values() {
    let mut text = String::default();
    for index in 0..10_000 {
        append!(
            text,
            "declare const unrelated{index}: typeof import('vue').useTemplateRef;\n"
        );
    }
    text.push_str("declare const useTemplateRef: typeof import('vue').useTemplateRef;\n");
    let facts = capture(&text, true, &["useTemplateRef"]);
    assert_eq!(facts.globals.len(), 1);
    assert_eq!(facts.globals[0].0, "useTemplateRef");
    assert_eq!(
        facts.bindings.len(),
        1,
        "only the requested origin dependency was lowered"
    );
    assert!(matches!(
        facts.bindings.values().next(),
        Some(BindingTarget::Import { .. })
    ));
    let empty = capture(&text, true, &[]);
    assert!(empty.globals.is_empty());
    assert!(empty.bindings.is_empty());
}

#[test]
fn existing_root_scope_maps_preserve_opaque_value_declaration_forms() {
    let text = r#"
class RootClass {}
enum RootEnum { One }
namespace RootNamespace { export const marker = 1; }
const { RootDestructure } = { RootDestructure: 1 };
for (var RootLoop of []) {}
{ var RootBlock = 1; let BlockOnly = 1; }
function nested() { const LocalOnly = 1; }
"#;
    let wanted = [
        "RootClass",
        "RootEnum",
        "RootNamespace",
        "RootDestructure",
        "RootLoop",
        "RootBlock",
    ];
    let facts = capture(text, true, &wanted);
    assert_eq!(facts.globals.len(), wanted.len());
    assert_eq!(facts.bindings.len(), wanted.len());
    let mut actual = facts
        .globals
        .iter()
        .map(|(name, _)| name.as_str())
        .collect::<Vec<_>>();
    let mut expected = wanted.to_vec();
    actual.sort_unstable();
    expected.sort_unstable();
    assert_eq!(actual, expected);
    for (_, local) in &facts.globals {
        assert!(matches!(
            facts.bindings.get(local),
            Some(BindingTarget::Opaque)
        ));
    }
    let excluded = capture(text, true, &["BlockOnly", "LocalOnly"]);
    assert!(excluded.globals.is_empty());
    assert!(excluded.bindings.is_empty());
    let module = capture(text, false, &wanted);
    assert!(
        module.globals.is_empty(),
        "host module classification is authoritative"
    );
}

#[test]
fn existing_global_augmentation_scope_retains_unsupported_values() {
    let text = r#"export {};
declare global {
  class RootClass {}
  enum RootEnum { One }
  namespace RootNamespace { const marker: number; }
}
"#;
    let wanted = ["RootClass", "RootEnum", "RootNamespace"];
    let facts = capture(text, false, &wanted);
    assert_eq!(facts.globals.len(), wanted.len());
    assert_eq!(facts.bindings.len(), wanted.len());
    for (_, local) in &facts.globals {
        assert!(matches!(
            facts.bindings.get(local),
            Some(BindingTarget::Opaque)
        ));
    }
}

#[test]
fn ambient_runtime_writes_cannot_borrow_provider_identity() {
    for suffix in [
        "useTemplateRef = (() => ({value:null})) as typeof useTemplateRef;",
        "for (useTemplateRef of [] as Array<typeof useTemplateRef>) {}",
        "for (var useTemplateRef of [] as Array<typeof useTemplateRef>) {}",
        "{ var useTemplateRef = (() => ({value:null})) as typeof useTemplateRef; }",
    ] {
        let text =
            format!("declare var useTemplateRef: typeof import('vue').useTemplateRef; {suffix}");
        let (origin, resolutions) = observe(&text, SourceType::ts(), true);
        assert!(
            matches!(origin, super::BindingOrigin::Custom(_)),
            "{text}: {origin:?}"
        );
        assert_eq!(resolutions, 0);
    }
}

#[test]
fn requested_global_roots_retain_actual_alias_dependencies() {
    let text =
        "declare const Actual: typeof import('vue').useTemplateRef; const useTemplateRef = Actual;";
    let facts = capture(text, true, &["useTemplateRef"]);
    assert_eq!(facts.globals.len(), 1);
    assert_eq!(facts.bindings.len(), 2);
    let local = facts.globals[0].1;
    let Some(BindingTarget::Alias { binding, .. }) = facts.bindings.get(&local) else {
        panic!("the original immutable value alias was lost");
    };
    assert!(matches!(
        facts.bindings.get(&binding.local),
        Some(BindingTarget::Import { .. })
    ));
}

#[test]
fn missing_original_semantic_scopes_refuse_the_whole_source() {
    let allocator = Allocator::default();
    let parsed = Parser::new(
        &allocator,
        "export {}; declare global {const useTemplateRef: typeof import('vue').useTemplateRef;}",
        SourceType::ts(),
    )
    .parse();
    assert!(!parsed.panicked);
    assert!(parsed.diagnostics.is_empty());
    let built = SemanticBuilder::new()
        .with_build_nodes(false)
        .build(&parsed.program);
    assert!(built.diagnostics.is_empty());
    let declaration = parsed
        .program
        .body
        .iter()
        .find_map(|statement| {
            if let oxc_ast::ast::Statement::TSGlobalDeclaration(declaration) = statement {
                Some(declaration)
            } else {
                None
            }
        })
        .unwrap();
    let original = declaration.scope_id.get();
    declaration.scope_id.set(None);
    let capture = || {
        collect_requested_globals(
            &parsed.program,
            &built.semantic,
            SourceRevision {
                owner: 1,
                source: 1,
                version: 1,
            },
            false,
            &[],
            &["useTemplateRef"],
        )
    };
    assert_eq!(
        capture().unwrap_err(),
        super::OriginError::MissingSemanticScope
    );
    declaration.scope_id.set(original);
    assert_eq!(capture().unwrap().globals.len(), 1);
    parsed.program.scope_id.set(None);
    assert_eq!(
        capture().unwrap_err(),
        super::OriginError::MissingSemanticScope
    );
}
