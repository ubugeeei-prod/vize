use super::{
    AssembledDiagnostic, AssembledOrigin, AssemblyPolicy, AuthoredSource, FinishedDiagnostic,
    ProjectedDocument, assemble_diagnostics, is_reportable,
};
use crate::batch::ImportSourceMap;
use crate::template_diagnostic_directives::UNUSED_EXPECT_ERROR_MESSAGE;
use crate::virtual_ts::{ProjectionMapping, VizeMapping};

const SFC: &str = "<script setup lang=\"ts\">\nconst count = 1\n</script>\n<template><p>{{ missing }}</p></template>\n";
const GENERATED: &str = "const count = 1\nvoid (missing);\nconst __vize_match_0_unreachable: __VizePatterns.Reachable<T> = true;\n";
const REPORT_UNUSED: AssemblyPolicy = AssemblyPolicy {
    report_unused: true,
};

fn mapping(source: &str) -> ProjectionMapping {
    let script = source.find("const count").unwrap_or(0);
    let missing = source.find("missing").unwrap_or(0);
    let assertion = GENERATED.find("__vize_match").unwrap();
    ProjectionMapping::from_spans(vec![
        VizeMapping::new(0..15, script..script + 15),
        VizeMapping::new(22..29, missing..missing + 7),
        VizeMapping::new(assertion..assertion + 32, missing..missing + 7),
    ])
}

fn finished(needle: &str, code: u32, message: &str) -> FinishedDiagnostic<u32> {
    let start = GENERATED.find(needle).unwrap();
    FinishedDiagnostic {
        document: 0,
        start,
        end: start + needle.len(),
        code: Some(code),
        severity: Some(1),
        message: message.into(),
        payload: code,
    }
}

fn assemble(
    source: &str,
    finished: Vec<FinishedDiagnostic<u32>>,
    policy: AssemblyPolicy,
) -> Vec<AssembledDiagnostic<u32>> {
    let mapping = mapping(source);
    let import_map = ImportSourceMap::empty();
    let documents = [ProjectedDocument {
        generated: GENERATED,
        import_map: &import_map,
        mapping: Some(&mapping),
        tsx: false,
    }];
    assemble_diagnostics(&AuthoredSource::vue(source), &documents, finished, policy)
}

fn at(
    source: &str,
    needle: &str,
    code: u32,
    severity: u8,
    message: &str,
) -> AssembledDiagnostic<u32> {
    let start = source.find(needle).unwrap();
    AssembledDiagnostic {
        start,
        end: start + needle.len(),
        code: Some(code),
        severity: Some(severity),
        message: message.into(),
        origin: AssembledOrigin::Checker(code),
    }
}

#[test]
fn template_lookup_failures_become_instance_diagnostics_once() {
    let diagnostic = finished("missing", 2304, "Cannot find name 'missing'.");
    let assembled = assemble(SFC, vec![diagnostic.clone(), diagnostic], REPORT_UNUSED);
    let mut expected = at(
        SFC,
        "missing",
        2339,
        1,
        "Property 'missing' does not exist on the component instance.",
    );
    expected.origin = AssembledOrigin::Checker(2304);
    assert_eq!(assembled, [expected]);
}

#[test]
fn script_diagnostics_keep_their_checker_code() {
    let assembled = assemble(
        SFC,
        vec![finished(
            "count",
            2322,
            "Type 'number' is not assignable to type 'string'.",
        )],
        REPORT_UNUSED,
    );
    assert_eq!(
        assembled,
        [at(
            SFC,
            "count",
            2322,
            1,
            "Type 'number' is not assignable to type 'string'."
        )]
    );
}

#[test]
fn unmapped_and_unreportable_diagnostics_are_dropped() {
    let mut unmapped = finished("void", 2345, "Unmapped.");
    unmapped.end = unmapped.start + 1;
    let assembled = assemble(
        SFC,
        vec![
            unmapped,
            finished("count", 2666, "Helper export."),
            finished(
                "count",
                6133,
                "'count' is declared but its value is never read.",
            ),
        ],
        AssemblyPolicy {
            report_unused: false,
        },
    );
    assert_eq!(assembled, []);
}

#[test]
fn reportability_names_every_rule() {
    let policy = AssemblyPolicy {
        report_unused: false,
    };
    assert!(!is_reportable(Some(2666), Some(1), "", policy));
    assert!(!is_reportable(
        Some(2322),
        Some(1),
        "Type 'ArrayBuffer | SharedArrayBuffer' is not assignable to type 'ArrayBuffer'.",
        policy
    ));
    assert!(!is_reportable(Some(7044), Some(4), "", policy));
    assert!(is_reportable(Some(7044), Some(1), "", policy));
    assert!(!is_reportable(Some(6133), Some(1), "", policy));
    assert!(is_reportable(
        Some(6133),
        Some(4),
        "'x' is declared but its value is never read.",
        REPORT_UNUSED
    ));
    assert!(!is_reportable(
        Some(6133),
        Some(4),
        "'__vize_ctx' is declared but its value is never read.",
        REPORT_UNUSED
    ));
    assert!(is_reportable(Some(2322), Some(1), "Type mismatch.", policy));
    // TS2307 is never skipped globally, whatever specifier it quotes, and the
    // noImplicitAny family surfaces as errors (#966).
    for (code, message) in [
        (
            2307,
            "Cannot find module './app.vue' or its corresponding type declarations.",
        ),
        (
            2307,
            "Cannot find module 'lodash-es' or its corresponding type declarations.",
        ),
        (7006, "any message"),
        (7043, "any message"),
        (7044, "any message"),
    ] {
        assert!(
            is_reportable(Some(code), Some(1), message, policy),
            "{code}"
        );
    }
    assert!(is_reportable(None, Some(1), "any message", policy));
}

#[test]
fn reachability_assertions_are_warnings_unless_elaborated_otherwise() {
    let exhaustive = "Type 'true' is not assignable to type 'never'.\n  Type 'x' is not assignable to type 'never'.";
    let other = "Type 'true' is not assignable to type 'never'.\nSomething else went wrong.";
    let assembled = assemble(
        SFC,
        vec![
            finished("__vize_match", 2322, exhaustive),
            finished("__vize_match", 2322, other),
        ],
        REPORT_UNUSED,
    );
    let severities: Vec<_> = assembled
        .iter()
        .map(|diagnostic| diagnostic.severity)
        .collect();
    assert_eq!(severities, [Some(2), Some(1)]);
}

#[test]
fn widened_literal_twins_collapse_onto_the_widened_spelling() {
    let assembled = assemble(
        SFC,
        vec![
            finished(
                "count",
                2322,
                "Type '1' is not assignable to type 'string'.",
            ),
            finished(
                "count",
                2322,
                "Type 'number' is not assignable to type 'string'.",
            ),
        ],
        REPORT_UNUSED,
    );
    assert_eq!(
        assembled,
        [at(
            SFC,
            "count",
            2322,
            1,
            "Type 'number' is not assignable to type 'string'."
        )]
    );
}

#[test]
fn template_directives_suppress_and_report_unused_expectations() {
    let source = "<script setup lang=\"ts\">\nconst count = 1\n</script>\n<template><p><!-- @vue-expect-error -->{{ missing }}</p></template>\n";
    let consumed = assemble(
        source,
        vec![finished("missing", 2304, "Cannot find name 'missing'.")],
        REPORT_UNUSED,
    );
    assert_eq!(consumed, []);

    let comment = "<!-- @vue-expect-error -->";
    let start = source.find(comment).unwrap();
    assert_eq!(
        assemble(source, Vec::new(), REPORT_UNUSED),
        [AssembledDiagnostic {
            start,
            end: start + comment.len(),
            code: Some(2578),
            severity: Some(1),
            message: UNUSED_EXPECT_ERROR_MESSAGE.into(),
            origin: AssembledOrigin::UnusedExpectation,
        }]
    );
}

#[test]
fn missing_vue_imports_land_on_the_authored_specifier() {
    let source = "<script setup lang=\"ts\">\nimport Missing from './Missing.vue'\n</script>\n";
    let message = "Cannot find module './Missing.vue.ts/__vize_missing_vue_import__' or its corresponding type declarations.";
    let mut diagnostic = finished("void", 2307, message);
    diagnostic.end = diagnostic.start + 1;
    let assembled = assemble(source, vec![diagnostic], REPORT_UNUSED);
    let start = source.find("./Missing.vue").unwrap();
    assert_eq!(
        assembled,
        [AssembledDiagnostic {
            start,
            end: start + "./Missing.vue".len(),
            code: Some(2307),
            severity: Some(1),
            message: "Cannot find module './Missing.vue' or its corresponding type declarations."
                .into(),
            origin: AssembledOrigin::MissingVueImport(2307),
        }]
    );
}

#[test]
fn script_documents_project_byte_for_byte() {
    let source = "const value: string = 1;\n";
    let import_map = ImportSourceMap::empty();
    let documents = [ProjectedDocument {
        generated: source,
        import_map: &import_map,
        mapping: None,
        tsx: false,
    }];
    let assembled = assemble_diagnostics(
        &AuthoredSource::script(source),
        &documents,
        vec![FinishedDiagnostic {
            document: 0,
            start: 6,
            end: 11,
            code: Some(2322),
            severity: Some(1),
            message: "Type 'number' is not assignable to type 'string'.".into(),
            payload: (),
        }],
        REPORT_UNUSED,
    );
    assert_eq!(
        assembled,
        [AssembledDiagnostic {
            start: 6,
            end: 11,
            code: Some(2322),
            severity: Some(1),
            message: "Type 'number' is not assignable to type 'string'.".into(),
            origin: AssembledOrigin::Checker(()),
        }]
    );
}
