//! Existing configured checkers consume the provider's complete original File.
use super::{
    Allocator, Lang, OriginalProgramError, Path, ProjectionError, assert_diagnosing_options,
    backend, block_on, file, project, real_bridge,
};
use lsp_types::{DocumentDiagnosticReport, DocumentDiagnosticReportResult, Range};
use oxc_parser::Parser;
use oxc_span::SourceType;
use sha2::{Digest, Sha256};
use vize_l0::{SourceRoot, Span, String, cstr, line_index::LineBreaks};
use vize_l2::{file::DeclarationKind, lang::js::JsxFileProducer};
use vize_l4::targets::ts::{ProgramProjection, SourceKind};

fn uri(path: &Path) -> String {
    let mut result = String::from("file://");
    for byte in path.to_string_lossy().bytes() {
        if byte.is_ascii_alphanumeric() || b"-._~/".contains(&byte) {
            result.push(char::from(byte));
        } else {
            result.push_str(&cstr!("%{byte:02X}"));
        }
    }
    result
}

fn span(source: &str, diagnostic: &serde_json::Value) -> Span {
    let start = diagnostic["start"].as_u64().unwrap() as u32;
    let end = start + diagnostic["length"].as_u64().unwrap() as u32;
    Span::new(
        vize_l0::line_index::utf16_offset(source, start).unwrap() as u32,
        vize_l0::line_index::utf16_offset(source, end).unwrap() as u32,
    )
}

fn range(source: &str, diagnostic: &serde_json::Value) -> serde_json::Value {
    let span = span(source, diagnostic);
    let (sl, sc) = LineBreaks::Lsp.offset_to_position(source, span.start as usize);
    let (el, ec) = LineBreaks::Lsp.offset_to_position(source, span.end as usize);
    serde_json::json!({"start":{"line":sl,"character":sc},"end":{"line":el,"character":ec}})
}

fn expected(case: &serde_json::Value, uri: &str) -> serde_json::Value {
    let source = case["source"].as_str().unwrap();
    let items = case["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .map(|diagnostic| {
            assert_eq!(diagnostic["category"], 1);
            let mut raw = serde_json::json!({
                "range":range(source,diagnostic),"severity":1,"code":diagnostic["code"],
                "source":"ts","message":diagnostic["message"],
            });
            if let Some(related) = diagnostic["related"].as_array() {
                raw["relatedInformation"] = related
                    .iter()
                    .map(|related| {
                        serde_json::json!({
                            "location":{"uri":uri,"range":range(source,related)},
                            "message":related["message"],
                        })
                    })
                    .collect::<Vec<_>>()
                    .into();
            }
            raw
        })
        .collect::<Vec<_>>();
    serde_json::json!({"kind":"full","items":items})
}

fn mapped(projection: &ProgramProjection<'_, '_>, range: Range) -> Span {
    let text = projection.document().as_str();
    let byte = |at: lsp_types::Position| {
        LineBreaks::Lsp
            .position_to_offset(text, at.line, at.character)
            .unwrap() as u32
    };
    projection
        .map_span(Span::new(byte(range.start), byte(range.end)))
        .unwrap()
}

#[test]
fn configured_original_typed_parameter_ts_tsx_preserve_whole_raw_reports_and_related_ranges() {
    let pack: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../davinci/vize_l4/tests/fixtures/typed-parameter-program-checker.json"
    ))
    .unwrap();
    let root = tempfile::TempDir::new().unwrap();
    drop(project(root.path(), true));
    let config = root.path().join("tsconfig.json");
    let mut configured: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&config).unwrap()).unwrap();
    configured["compilerOptions"]["jsx"] = serde_json::json!("preserve");
    std::fs::write(&config, serde_json::to_vec(&configured).unwrap()).unwrap();
    let config_bytes = std::fs::read(&config).unwrap();
    let backend = backend();
    let backend_hash = Sha256::digest(std::fs::read(&backend).unwrap())
        .iter()
        .map(|byte| cstr!("{byte:02x}"))
        .collect::<String>();
    let bridge = real_bridge(root.path());
    block_on(bridge.spawn()).unwrap();
    let mut evidence = Vec::new();
    for case in pack["cases"].as_array().unwrap() {
        let arena = Allocator::default();
        let source = case["source"].as_str().unwrap();
        let original = file(&arena, source, Lang::Ts);
        assert!(original.is_complete(), "{:?}", original.issues());
        let mut tsx_case = case.clone();
        tsx_case["source"] = case["tsxSource"].clone();
        let tsx_source = tsx_case["source"].as_str().unwrap();
        let block = SourceRoot::new(tsx_source).unwrap().whole_block();
        let observed =
            Parser::new(&arena, tsx_source, SourceType::tsx().with_module(true)).parse_observed();
        let body = observed.admitted().unwrap().program().body.as_ptr();
        let mut producer = JsxFileProducer::new(&arena, observed, block, 17)
            .unwrap_or_else(|rejected| panic!("original TSX: {:?}", rejected.error()));
        producer.walk().unwrap();
        let owner = Box::new(
            producer
                .finish()
                .unwrap_or_else(|rejected| panic!("complete TSX: {:?}", rejected.error())),
        );
        assert!(owner.nodes().any(|node| node.source() == Some("<></>")));
        for tsx in [false, true] {
            let case = if tsx { &tsx_case } else { case };
            let source = case["source"].as_str().unwrap();
            // Actual distinct basenames avoid real .ts shadowing of .tsx.
            let path = root.path().join(if tsx {
                "OriginalParameterTsx@+雪.tsx"
            } else {
                "OriginalParameterTs@+雪.ts"
            });
            std::fs::write(&path, source.as_bytes()).unwrap();
            let checked = if tsx {
                block_on(bridge.check_original_jsx(&owner, &path))
            } else {
                block_on(bridge.check_original_program(&original, &path))
            }
            .unwrap();
            assert_diagnosing_options(&checked);
            let file = if tsx { owner.file() } else { &original };
            let projection = checked.projection();
            assert!(core::ptr::eq(projection.file(), file));
            assert!(core::ptr::eq(file.artifact().source(), source));
            assert_eq!(
                projection.source_kind(),
                if tsx {
                    SourceKind::Tsx
                } else {
                    SourceKind::TypeScript
                }
            );
            assert_eq!(
                projection.document().as_str(),
                cstr!("{source}\n;\nexport {{}};\n")
            );
            let name = case["binding"].as_str().unwrap();
            let binding = file
                .binding_at_offset(source.rfind(name).unwrap() as u32)
                .unwrap()
                .unwrap();
            assert!(core::ptr::eq(binding.file(), file));
            assert_eq!(
                binding.declaration().unwrap().kind,
                DeclarationKind::Parameter
            );
            assert_eq!(checked.source_path(), path.canonicalize().unwrap());
            assert_eq!(checked.source_uri(), uri(&path.canonicalize().unwrap()));
            assert_eq!(
                checked.source_digest(),
                case[if tsx {
                    "tsxSourceSha256"
                } else {
                    "sourceSha256"
                }]
                .as_str()
                .unwrap()
            );
            assert_eq!(
                serde_json::to_value(checked.report()).unwrap(),
                expected(case, checked.source_uri()),
                "{}: complete unfiltered raw report",
                case["id"]
            );
            let DocumentDiagnosticReportResult::Report(DocumentDiagnosticReport::Full(report)) =
                checked.report()
            else {
                panic!("complete Full report")
            };
            let diagnostics = &report.full_document_diagnostic_report.items;
            let mut spans = Vec::new();
            for (actual, primary) in diagnostics
                .iter()
                .zip(case["diagnostics"].as_array().unwrap())
            {
                let authored = span(source, primary);
                assert_eq!(
                    authored.slice(source),
                    primary["authored"].as_str().unwrap()
                );
                assert_eq!(mapped(projection, actual.range), authored);
                spans.push(Ok(authored));
                for (actual, primary) in actual
                    .related_information
                    .as_ref()
                    .into_iter()
                    .flatten()
                    .zip(primary["related"].as_array().into_iter().flatten())
                {
                    assert_eq!(actual.location.uri.as_str(), checked.source_uri());
                    let authored = span(source, primary);
                    assert_eq!(
                        authored.slice(source),
                        primary["authored"].as_str().unwrap()
                    );
                    assert_eq!(mapped(projection, actual.location.range), authored);
                }
            }
            assert_eq!(checked.authored_spans(), spans);
            assert_eq!(std::fs::read(&path).unwrap(), source.as_bytes());
            assert_eq!(std::fs::read(&config).unwrap(), config_bytes);
            let custody = checked.diagnosing_configuration();
            let snapshots = [custody.before(), custody.after()].map(|snapshot| {
                serde_json::json!({
                    "handle":snapshot.handle(),"projects":snapshot.projects(),
                    "changes":snapshot.changes(),"selectedProject":snapshot.project(),
                })
            });
            evidence.push(serde_json::json!({
                "id":case["id"],"extension":projection.source_kind().extension(),
                "source":source,"projected":projection.document().as_str(),
                "sourcePath":checked.source_path(),"sourceUri":checked.source_uri(),
                "sourceDigest":checked.source_digest(),"report":checked.report(),
                "configuration":checked.configuration(),
                "configurationPath":checked.diagnostic_configuration_path(),
                "session":custody.session(),"snapshots":snapshots,
                "authoredSpans":spans.iter().map(|span| {
                    let span = span.as_ref().unwrap();
                    serde_json::json!({"start":span.start,"end":span.end})
                }).collect::<Vec<_>>(),
            }));
        }
        assert_eq!(
            owner
                .observation()
                .admitted()
                .unwrap()
                .program()
                .body
                .as_ptr(),
            body
        );
        assert_eq!(owner.observation().comments().len(), 1);
    }
    block_on(bridge.shutdown()).unwrap();
    assert_eq!(evidence.len(), 12);
    if let Ok(profile) = std::env::var("NEXTEST_PROFILE") {
        assert!(matches!(profile.as_str(), "pr" | "full"));
        let path = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .parent()
            .unwrap()
            .join("nextest")
            .join(profile)
            .join("original-typed-parameter-canon.json");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(
            path,
            serde_json::to_vec_pretty(&serde_json::json!({
                "backendPath":backend,"backendSha256":backend_hash,
                "authoredConfiguration":configured,"cases":evidence,
            }))
            .unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn unsupported_original_function_fields_refuse_without_backend_or_source_rewriting() {
    let arena = Allocator::default();
    let root = tempfile::TempDir::new().unwrap();
    let bridge = project(root.path(), true);
    for source in [
        "function f(value:number[]){return value;}f([]);",
        "function f(value:any){return value;}f(1);",
        "export function f(value:number){return value;}f(1);",
    ] {
        let original = file(&arena, source, Lang::Ts);
        assert!(!original.is_complete());
        let path = root.path().join("OriginalParameter.ts");
        std::fs::write(&path, source).unwrap();
        assert!(matches!(
            block_on(bridge.check_original_program(&original, &path)),
            Err(OriginalProgramError::Projection(
                ProjectionError::IncompleteFile
            ))
        ));
        assert!(core::ptr::eq(original.artifact().source(), source));
        assert_eq!(std::fs::read(&path).unwrap(), source.as_bytes());
    }
}
