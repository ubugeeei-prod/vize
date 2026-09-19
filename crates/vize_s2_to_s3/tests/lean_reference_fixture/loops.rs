#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

use std::path::Path;
use vize_davinci::folio::{Folio, FolioMode};
use vize_s0::Allocator;
use vize_s3::folio::S3Folio;
use vize_s3::trace::{TraceBackend, backend_trace_text, reference_trace_text};
use vize_s3::values_folio::S3ValuesFolio;
use vize_s3::verify::verify;

#[test]
fn loop_reference_graph_and_values_are_rust_lowered() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../formal/impeto/fixtures");
    for name in ["keyed", "unkeyed", "nested"] {
        let stem = fixtures.join(format!("rust-lowered-loop-{name}"));
        let source = std::fs::read_to_string(stem.with_extension("template.txt")).unwrap();
        let allocator = Allocator::default();
        let (tree, errors) = vize_s1::parse(&allocator, source.trim_end());
        assert!(errors.is_empty(), "{errors:?}");
        let s2 = vize_s1_to_s2::lower(&allocator, &tree, &errors);
        assert!(s2.diagnostics.is_empty(), "{:?}", s2.diagnostics);
        let lowered = vize_s2_to_s3::lower(&allocator, &s2.root);
        assert_eq!(verify(&lowered.program), []);
        for (extension, actual) in [
            (
                "s3.folio",
                S3Folio::of(&lowered.program)
                    .print_to_string(FolioMode::Full)
                    .to_string(),
            ),
            (
                "values.folio",
                S3ValuesFolio::of(&lowered.program)
                    .print_to_string(FolioMode::Full)
                    .to_string(),
            ),
            ("trace", reference_trace_text(&lowered.program).to_string()),
            (
                "vdom.trace",
                backend_trace_text(TraceBackend::Vdom, &lowered.program).to_string(),
            ),
            (
                "vapor.trace",
                backend_trace_text(TraceBackend::Vapor, &lowered.program).to_string(),
            ),
        ] {
            let path = stem.with_extension(extension);
            if std::env::var("VIZE_UPDATE_LOOP_REFERENCE_FIXTURES").as_deref() == Ok("1") {
                std::fs::write(&path, format!("{}\n", actual.trim_end())).unwrap();
            }
            let expected = std::fs::read_to_string(&path).unwrap();
            assert_eq!(actual.trim_end(), expected.trim_end(), "{}", path.display());
        }
    }
}
