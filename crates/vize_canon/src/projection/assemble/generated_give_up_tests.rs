//! Checker give-ups in unmapped generated code surface once per file (#9016).

use super::{
    AssembledDiagnostic, AssembledOrigin, AssemblyPolicy, AuthoredSource, FinishedDiagnostic,
    ProjectedDocument, assemble_diagnostics,
};
use crate::batch::ImportSourceMap;
use crate::virtual_ts::{ProjectionMapping, VizeMapping};

const SFC: &str = "<script setup lang=\"ts\">\nconst count = 1\n</script>\n<template><p>{{ count }}</p></template>\n";
const GENERATED: &str =
    "const count = 1\nfunction __vize_render() { if (a) {} else if (b) {} }\nvoid (count);\n";
const POLICY: AssemblyPolicy = AssemblyPolicy {
    report_unused: true,
};

fn diagnostic(needle: &str, code: u32) -> FinishedDiagnostic<u32> {
    let start = GENERATED.find(needle).unwrap();
    FinishedDiagnostic {
        document: 0,
        start,
        end: start + needle.len(),
        code: Some(code),
        severity: Some(1),
        message: "checker message".into(),
        payload: code,
    }
}

fn assemble(finished: Vec<FinishedDiagnostic<u32>>) -> Vec<AssembledDiagnostic<u32>> {
    let script = SFC.find("const count").unwrap();
    // Only the authored script statement is mapped; the render function is
    // generated code with no authored range.
    let mapping = ProjectionMapping::from_spans(vec![VizeMapping::new(0..15, script..script + 15)]);
    let import_map = ImportSourceMap::empty();
    let documents = [ProjectedDocument {
        generated: GENERATED,
        import_map: &import_map,
        mapping: Some(&mapping),
        tsx: false,
    }];
    assemble_diagnostics(&AuthoredSource::vue(SFC), &documents, finished, POLICY)
}

#[test]
fn control_flow_give_up_in_generated_code_is_an_internal_error_on_the_template() {
    let assembled = assemble(vec![diagnostic("__vize_render", 2563)]);
    // The template block range starts at its content, after the opening tag.
    let template = SFC.find("<template>").unwrap() + "<template>".len();
    assert_eq!(assembled.len(), 1);
    let only = &assembled[0];
    assert_eq!(only.origin, AssembledOrigin::GeneratedGiveUp(2563));
    assert_eq!((only.start, only.end), (template, template));
    assert_eq!(only.code, Some(2563));
    assert_eq!(only.severity, Some(1));
    assert_eq!(
        only.message.as_str(),
        "Vize internal error: the type-check code generated for this file made the checker give up (TS2563). Types in this and other files may be degraded. Please report this file to Vize."
    );
}

#[test]
fn repeated_give_ups_in_one_file_are_reported_once() {
    let assembled = assemble(vec![
        diagnostic("__vize_render", 2563),
        diagnostic("if (a)", 2589),
        diagnostic("else if (b)", 2590),
    ]);
    let give_ups: Vec<_> = assembled
        .iter()
        .filter(|diagnostic| matches!(diagnostic.origin, AssembledOrigin::GeneratedGiveUp(_)))
        .collect();
    assert_eq!(give_ups.len(), 1);
    assert_eq!(give_ups[0].code, Some(2563));
}

#[test]
fn other_unmapped_generated_diagnostics_stay_dropped() {
    assert_eq!(assemble(vec![diagnostic("__vize_render", 2493)]), []);
}

#[test]
fn a_give_up_with_an_authored_range_is_an_ordinary_diagnostic() {
    let assembled = assemble(vec![diagnostic("const count", 2589)]);
    assert_eq!(assembled.len(), 1);
    assert!(matches!(
        assembled[0].origin,
        AssembledOrigin::Checker(2589)
    ));
}

const CALL_SFC: &str = "<template><CountTo v-bind=\"props\" /></template>\n";
const CALL: &str = "(undefined as unknown as Check)({\n  ...props,\n});\nvoid x;\n";

fn assemble_call(start: usize, end: usize) -> Vec<AssembledDiagnostic<u32>> {
    let tag = CALL_SFC.find("CountTo").unwrap();
    let literal = CALL.find('{').unwrap()..CALL.find('}').unwrap() + 1;
    let mapping = ProjectionMapping::from_spans(vec![VizeMapping::new(literal, tag..tag + 7)]);
    let import_map = ImportSourceMap::empty();
    let documents = [ProjectedDocument {
        generated: CALL,
        import_map: &import_map,
        mapping: Some(&mapping),
        tsx: false,
    }];
    let finished = vec![FinishedDiagnostic {
        document: 0,
        start,
        end,
        code: Some(2589),
        severity: Some(1),
        message: "Type instantiation is excessively deep and possibly infinite.".into(),
        payload: 2589,
    }];
    assemble_diagnostics(&AuthoredSource::vue(CALL_SFC), &documents, finished, POLICY)
}

/// `check({ ...props })` with a deep spread is the user's own TS2589, so it is
/// reported on the component tag the props literal belongs to.
#[test]
fn a_give_up_over_a_mapped_call_lands_on_its_authored_owner() {
    let tag = CALL_SFC.find("CountTo").unwrap();
    for end in [0, CALL.find(';').unwrap() + 1] {
        assert_eq!(
            assemble_call(0, end),
            [AssembledDiagnostic {
                start: tag,
                end: tag + 7,
                code: Some(2589),
                severity: Some(1),
                message: "Type instantiation is excessively deep and possibly infinite.".into(),
                origin: AssembledOrigin::Checker(2589),
            }],
            "end {end}"
        );
    }
    // The following statement holds no authored row.
    let after = CALL.find("void").unwrap();
    assert_eq!(
        assemble_call(after, after)[0].origin,
        AssembledOrigin::GeneratedGiveUp(2589)
    );
}
