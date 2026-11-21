//! The current Canon generator and the opt-in L4 projection are checked by
//! the same published TypeScript compiler at the same module path.
#![cfg(test)]
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "these checker fixtures use standard strings for TypeScript process input and output"
)]

mod cases;
mod checker;
mod corpus;
mod input;
mod profile;

use cases::{CASES, WIDGET, source};
use checker::{Comparison, compare_checker};
use input::lower;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use std::path::Path;
use vize_atelier_jsx::JsxLang;
use vize_canon::batch::generate_jsx_document_virtual_ts;
use vize_l0::Allocator;
use vize_l1_to_l2::framework::vue::jsx::{ModuleOptions, VIZE, lower_module_def};
use vize_l2::component::Exec;
use vize_l4::targets::ts::{
    ModuleProjectionError, project_module, project_module_with_component_checks,
};

#[test]
fn published_helpers_match_existing_checker_diagnostics_and_original_bytes() {
    let mut documents = Vec::new();
    for &case in CASES {
        let source = source(case);
        let original =
            generate_jsx_document_virtual_ts(Path::new("View.tsx"), &source, JsxLang::Tsx).unwrap();
        assert!(
            original.diagnostics.is_empty(),
            "{}: {:?}",
            case.id,
            original.diagnostics
        );
        let arena = Allocator::default();
        let module = lower(&arena, &source);
        assert_eq!(
            project_module(&module).unwrap_err(),
            ModuleProjectionError::UnpublishedHelper("ui.component")
        );
        let document = project_module_with_component_checks(&module)
            .unwrap_or_else(|e| panic!("{}: {e:?}", case.id));
        documents.push(Comparison::new(case.id, source, original, document));
    }
    let results = compare_checker(documents);
    for (case, codes) in CASES.iter().zip(results) {
        assert_eq!(codes, case.codes, "{}", case.id);
    }
}

#[test]
fn unsupported_owners_and_slot_scopes_refuse_the_whole_document() {
    for root in [
        "<Unknown/>",
        "<component is={Widget}/>",
        "<Widget<number>/>",
        "<Widget /* explicit */ <number>/>",
        "<Widget v-slots={external}/>",
        "<Widget>{(p) => <div/>}</Widget>",
        "<Widget onClickOnce={external}/>",
    ] {
        let source = format!("{WIDGET} export function View() {{return <div><i/>{root}</div>}}");
        let arena = Allocator::default();
        let module = lower(&arena, &source);
        assert!(
            project_module_with_component_checks(&module).is_err(),
            "{root}"
        );
    }
}

#[test]
fn a_canon_diagnostic_endpoint_inside_a_unicode_tag_is_an_explicit_refusal() {
    let source =
        "declare const 雪山: (props: { n: number }) => unknown; export const View = () => <雪山/>;";
    let arena = Allocator::default();
    let module = lower(&arena, source);
    assert_eq!(
        project_module_with_component_checks(&module).unwrap_err(),
        ModuleProjectionError::Span
    );
}
