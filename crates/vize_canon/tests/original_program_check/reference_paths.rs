//! The unchanged reported program agrees with the native and batch checkers.
use super::{Allocator, Lang, assert_diagnosing_options, block_on, file, real_bridge};
use lsp_types::{DocumentDiagnosticReport, DocumentDiagnosticReportResult};
use sha2::{Digest, Sha256};
use vize_canon::{BatchTypeChecker, BatchTypeCheckerTrait};

#[path = "../support/reference_path_project.rs"]
mod project;

#[test]
fn path_references_retain_original_sources_native_reports_and_configuration() {
    for case in project::CASES {
        for invalid in [false, true] {
            let directory = tempfile::tempdir().unwrap();
            let root = directory.path().canonicalize().unwrap();
            let paths = project::prepare(&root, case, invalid);
            let originals = paths
                .iter()
                .map(|path| std::fs::read(path).unwrap())
                .collect::<Vec<_>>();
            let source_path = root.join("src/a.ts");
            let source = std::fs::read_to_string(&source_path).unwrap();
            let arena = Allocator::default();
            let original = file(&arena, &source, Lang::Ts);
            assert!(original.is_complete(), "{:?}", original.issues());
            let bridge = real_bridge(&root);
            block_on(bridge.spawn()).unwrap();
            let result = block_on(bridge.check_original_program(&original, &source_path)).unwrap();
            assert_diagnosing_options(&result);
            assert!(std::ptr::eq(result.projection().file(), &original));
            assert_eq!(result.source_path(), source_path);
            assert_eq!(
                result.source_digest(),
                vize_l0::cstr!("{:x}", Sha256::digest(source.as_bytes())).as_str()
            );
            assert_eq!(
                result.diagnostic_configuration_path(),
                root.join("tsconfig.json")
            );
            assert_eq!(result.configuration().options["strict"], true);
            assert_eq!(
                result.configuration().options["skipLibCheck"]
                    .as_bool()
                    .unwrap_or(false),
                case.4
            );
            let DocumentDiagnosticReportResult::Report(DocumentDiagnosticReport::Full(report)) =
                result.report()
            else {
                panic!("complete native report required")
            };
            let expected = if invalid {
                serde_json::json!([{
                    "range":{"start":{"line":1,"character":13},"end":{"line":1,"character":17}},
                    "severity":1,"code":2322,"source":"ts",
                    "message":"Type 'string' is not assignable to type 'number'."
                }])
            } else {
                serde_json::json!([])
            };
            assert_eq!(
                serde_json::to_value(&report.full_document_diagnostic_report.items).unwrap(),
                expected,
                "{}: full native vector, invalid={invalid}",
                case.0
            );
            block_on(bridge.shutdown()).unwrap();
            let mut checker = BatchTypeChecker::new(&root).unwrap();
            checker.scan_project().unwrap();
            let checked = checker.check_project().unwrap();
            let diagnostics = checked
                .diagnostics
                .iter()
                .map(|diagnostic| {
                    serde_json::json!({"file":diagnostic.file.strip_prefix(&root).unwrap(),
                    "line":diagnostic.line,"column":diagnostic.column,
                    "severity":diagnostic.severity,"code":diagnostic.code,
                    "message":diagnostic.message.as_str()})
                })
                .collect::<Vec<_>>();
            assert_eq!(
                serde_json::json!(diagnostics),
                if invalid {
                    serde_json::json!([{"file":"src/a.ts","line":1,"column":13,
                    "severity":1,"code":2322,"message":"Type 'string' is not assignable to type 'number'."}])
                } else {
                    serde_json::json!([])
                }
            );
            assert_eq!(checked.success, !invalid);
            for (path, original) in paths.iter().zip(originals) {
                assert_eq!(std::fs::read(path).unwrap(), original);
            }
        }
    }
}
