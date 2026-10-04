//! Same-owner whole array sources, namespace reads and independent full vectors.
#[cfg(test)]
mod cases {
    use oxc_parser::Parser;
    use oxc_span::SourceType;
    use std::io::Write;
    use std::process::{Command, Stdio};
    use vize_l0::{Allocator, SourceRoot, Span, cstr};
    use vize_l1::embed::{
        EmbedSource, Lang,
        syntax::{ProgramOptions, parse_program_once},
    };
    use vize_l2::lang::js::{FileProducer, JsxFileProducer, ProgramInput, ProgramScope};
    use vize_l4::targets::ts::{
        ProgramProjection, SourceKind, project_jsx_program, project_program,
    };

    fn packet(
        case: &serde_json::Value,
        projection: &ProgramProjection<'_, '_>,
    ) -> serde_json::Value {
        let source = case["source"].as_str().unwrap();
        assert_eq!(
            projection.document().as_str(),
            cstr!("{source}\n;\nexport {{}};\n")
        );
        let [link] = projection.document().links() else {
            panic!("one original whole-source link");
        };
        assert_eq!(link.authored, Span::new(0, source.len() as u32));
        assert_eq!(projection.map_span(link.generated), Ok(link.authored));
        let at = source
            .rfind(if source.contains("日本語") {
                "日本語"
            } else {
                "items"
            })
            .unwrap() as u32;
        let binding = projection.file().binding_at_offset(at).unwrap().unwrap();
        assert!(core::ptr::eq(binding.file(), projection.file()));
        for diagnostic in case["diagnostics"].as_array().unwrap() {
            let authored = projection
                .map_utf16(
                    diagnostic["start"].as_u64().unwrap() as u32,
                    diagnostic["length"].as_u64().unwrap() as u32,
                )
                .unwrap();
            assert_eq!(
                authored.slice(source),
                diagnostic["authored"].as_str().unwrap()
            );
            for related in diagnostic["related"].as_array().into_iter().flatten() {
                let authored = projection
                    .map_utf16(
                        related["start"].as_u64().unwrap() as u32,
                        related["length"].as_u64().unwrap() as u32,
                    )
                    .unwrap();
                assert_eq!(
                    authored.slice(source),
                    related["authored"].as_str().unwrap()
                );
            }
        }
        serde_json::json!({
            "case":case,
            "extension":projection.source_kind().extension(),
            "projected":projection.document().as_str(),
        })
    }

    #[test]
    fn actual_array_ts_and_tsx_projections_preserve_primary_full_vectors_and_value_navigation() {
        let pack: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/array-program-checker.json")).unwrap();
        let mut packets = Vec::new();
        for case in pack["cases"].as_array().unwrap() {
            let arena = Allocator::default();
            let source = case["source"].as_str().unwrap();
            let block = SourceRoot::new(source).unwrap().whole_block();
            let syntax = parse_program_once(
                &arena,
                EmbedSource::authored(source, block.span()).unwrap(),
                ProgramOptions::module(Lang::Ts),
            );
            let body = syntax.program().unwrap().body.as_ptr();
            let mut producer = FileProducer::new(&arena, source).unwrap();
            producer
                .program(
                    ProgramInput::checked(syntax.admitted_program().unwrap(), block, 17).unwrap(),
                    ProgramScope::Module,
                )
                .unwrap();
            let file = producer.finish().unwrap();
            let projection = project_program(&file).unwrap();
            assert_eq!(projection.source_kind(), SourceKind::TypeScript);
            assert!(core::ptr::eq(projection.file(), &file));
            packets.push(packet(case, &projection));
            assert_eq!(syntax.program().unwrap().body.as_ptr(), body);
            assert_eq!(syntax.comments().count(), 1);

            let mut case = case.clone();
            case["source"] = case["tsxSource"].clone();
            let source = case["source"].as_str().unwrap();
            let block = SourceRoot::new(source).unwrap().whole_block();
            let observed =
                Parser::new(&arena, source, SourceType::tsx().with_module(true)).parse_observed();
            let body = observed.admitted().unwrap().program().body.as_ptr();
            let mut producer = JsxFileProducer::new(&arena, observed, block, 17)
                .unwrap_or_else(|rejected| panic!("original TSX: {:?}", rejected.error()));
            producer.walk().unwrap();
            let owner = Box::new(
                producer
                    .finish()
                    .unwrap_or_else(|rejected| panic!("whole TSX: {:?}", rejected.error())),
            );
            let projection = project_jsx_program(&owner).unwrap();
            assert_eq!(projection.source_kind(), SourceKind::Tsx);
            assert!(core::ptr::eq(projection.file(), owner.file()));
            packets.push(packet(&case, &projection));
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
        let mut child = Command::new("node")
            .args([
                "--input-type=module",
                "-e",
                include_str!("../../../tests/tooling/support/native-array-typescript.ts"),
            ])
            .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(&serde_json::to_vec(&packets).unwrap())
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            std::str::from_utf8(&output.stderr).unwrap()
        );
        let evidence: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(evidence["version"], "6.0.3");
        assert_eq!(evidence["cases"].as_array().unwrap().len(), 10);
        if let Ok(path) = std::env::var("VIZE_L4_ARRAY_CHECK_CAPTURE") {
            std::fs::write(path, serde_json::to_vec_pretty(&evidence).unwrap()).unwrap();
        }
    }
}
