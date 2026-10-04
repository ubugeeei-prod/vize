//! The shared provider unlocks actual unchanged source checks, not setup erasure.
use super::{
    Allocator, Lang, OriginalProgramError, Path, ProjectionError, assert_diagnosing_options,
    block_on, file, project, real_bridge,
};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::{SourceRoot, Span, String, cstr, line_index::LineBreaks};
use vize_l2::lang::js::JsxFileProducer;
use vize_l4::targets::ts::SourceKind;

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

#[test]
fn configured_original_array_ts_tsx_preserve_complete_reports_and_related_authored_ranges() {
    let pack: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../davinci/vize_l4/tests/fixtures/array-program-checker.json"
    ))
    .unwrap();
    let root = tempfile::TempDir::new().unwrap();
    drop(project(root.path(), true));
    let config = root.path().join("tsconfig.json");
    let mut configured: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&config).unwrap()).unwrap();
    configured["compilerOptions"]["jsx"] = serde_json::json!("preserve");
    std::fs::write(config, serde_json::to_vec(&configured).unwrap()).unwrap();
    let bridge = real_bridge(root.path());
    block_on(bridge.spawn()).unwrap();
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
        for tsx in [false, true] {
            let case = if tsx { &tsx_case } else { case };
            let source = case["source"].as_str().unwrap();
            let path = root.path().join(if tsx {
                "OriginalTsx@+雪.tsx"
            } else {
                "OriginalTs@+雪.ts"
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
            assert!(core::ptr::eq(checked.projection().file(), file));
            assert_eq!(
                checked.projection().source_kind(),
                if tsx {
                    SourceKind::Tsx
                } else {
                    SourceKind::TypeScript
                }
            );
            assert_eq!(
                checked.projection().document().as_str(),
                cstr!("{source}\n;\nexport {{}};\n")
            );
            assert_eq!(checked.source_path(), path.canonicalize().unwrap());
            assert_eq!(checked.source_uri(), uri(&path.canonicalize().unwrap()));
            assert_eq!(
                serde_json::to_value(checked.report()).unwrap(),
                expected(case, checked.source_uri()),
                "{}: whole raw report",
                case["id"]
            );
            let spans = case["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .map(|diagnostic| {
                    let span = span(source, diagnostic);
                    assert_eq!(span.slice(source), diagnostic["authored"].as_str().unwrap());
                    Ok(span)
                })
                .collect::<Vec<_>>();
            assert_eq!(checked.authored_spans(), spans);
            assert_eq!(std::fs::read(&path).unwrap(), source.as_bytes());
            if tsx {
                assert!(matches!(
                    block_on(bridge.check_original_program(owner.file(), &path)),
                    Err(OriginalProgramError::Projection(
                        ProjectionError::UnsupportedProfile
                    ))
                ));
            }
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
}

#[test]
fn nonflat_array_types_and_typed_functions_keep_original_checker_refusals() {
    let arena = Allocator::default();
    let root = tempfile::TempDir::new().unwrap();
    let bridge = project(root.path(), true);
    for source in [
        "const items:number[][]=[];items;",
        "const items:{id?:number}[]=[];items;",
        "function read(items:number[]){return items;}",
    ] {
        let original = file(&arena, source, Lang::Ts);
        assert!(!original.is_complete());
        let path = root.path().join("Original.ts");
        std::fs::write(&path, source).unwrap();
        assert!(matches!(
            block_on(bridge.check_original_program(&original, &path)),
            Err(OriginalProgramError::Projection(
                ProjectionError::IncompleteFile
            ))
        ));
        assert_eq!(original.artifact().source(), source);
        assert_eq!(std::fs::read(&path).unwrap(), source.as_bytes());
    }
}
