//! Whole original required named keyword exports retain their actual owner and primary vector.
use oxc_parser::Parser;
use oxc_span::SourceType;
use std::io::Write;
use std::process::{Command, Stdio};
use vize_l0::{Allocator, SourceRoot, Span, cstr};
use vize_l1::embed::{
    EmbedSource, Lang,
    syntax::{ProgramOptions, parse_program_once},
};
use vize_l2::{
    file::{DeclarationKind, Namespace},
    lang::js::{FileProducer, JsxFileProducer, ProgramInput, ProgramScope},
};
use vize_l4::targets::ts::{
    MappingError, ProgramProjection, ProjectionError, SourceKind, project_jsx_program,
    project_jsx_program_no_links, project_program, project_program_no_links,
};

fn packet(case: &serde_json::Value, projection: &ProgramProjection<'_, '_>) -> serde_json::Value {
    let source = case["source"].as_str().unwrap();
    let file = projection.file();
    assert!(file.is_complete(), "{:?}", file.issues());
    assert!(core::ptr::eq(file.artifact().source(), source));
    assert!(core::ptr::eq(projection.unit(), &file.units()[0]));
    assert!(projection.unit().profile.module);
    assert!(projection.unit().profile.typescript);
    assert_eq!(
        projection.unit().profile.jsx,
        projection.source_kind() == SourceKind::Tsx
    );
    assert_eq!(
        projection.document().as_str(),
        cstr!("{source}\n;\nexport {{}};\n")
    );
    let [link] = projection.document().links() else {
        panic!("one whole original source link")
    };
    assert_eq!(link.authored, Span::new(0, source.len() as u32));
    assert_eq!(projection.map_span(link.generated), Ok(link.authored));
    let names: &[&str] = match case["id"].as_str().unwrap() {
        "named-export-seven-keywords" => &["big", "bool", "nul", "num", "text", "sym", "undef"],
        "named-export-unicode-order" => &["表示"],
        _ => &["echo"],
    };
    assert_eq!(file.exports().len(), names.len());
    for (export, name) in file.exports().iter().zip(names) {
        let local = file
            .lookup(projection.unit().scope, name, Namespace::Value)
            .unwrap();
        let declaration = local.declaration().unwrap();
        assert_eq!(declaration.kind, DeclarationKind::Function);
        assert_eq!(export.name.as_str(), *name);
        assert_eq!(export.namespace, Namespace::Value);
        assert_eq!(export.local, Some(local.id()));
        assert_eq!(export.unit, projection.unit().id);
        assert_eq!(export.span, declaration.span);
        assert_eq!(export.span.slice(source), *name);
        assert!(export.source.is_none());
        assert!(core::ptr::eq(local.file(), file));
        assert!(
            file.lookup(projection.unit().scope, name, Namespace::Type)
                .is_none()
        );
    }
    let name = case["binding"].as_str().unwrap();
    let at = source.rfind(name).unwrap() as u32;
    let binding = file.binding_at_offset(at).unwrap().unwrap();
    assert!(core::ptr::eq(binding.file(), file));
    let declaration = binding.declaration().unwrap();
    assert_eq!(declaration.kind, DeclarationKind::Parameter);
    assert_eq!(declaration.namespace, Namespace::Value);
    assert_eq!(declaration.unit, projection.unit().id);
    assert_ne!(declaration.scope, projection.unit().scope);
    assert_eq!(declaration.span.slice(source), name);
    assert!(
        file.lookup(projection.unit().scope, name, Namespace::Value)
            .is_none()
    );
    for diagnostic in case["diagnostics"].as_array().unwrap() {
        for record in core::iter::once(diagnostic)
            .chain(diagnostic["related"].as_array().into_iter().flatten())
        {
            let authored = projection
                .map_utf16(
                    record["start"].as_u64().unwrap() as u32,
                    record["length"].as_u64().unwrap() as u32,
                )
                .unwrap();
            assert_eq!(authored.slice(source), record["authored"].as_str().unwrap());
        }
    }
    serde_json::json!({
        "case":case,
        "extension":projection.source_kind().extension(),
        "projected":projection.document().as_str(),
    })
}

#[test]
fn original_required_named_export_ts_tsx_projection_preserves_full_primary_vectors_and_navigation()
{
    let pack: serde_json::Value = serde_json::from_str(include_str!(
        "fixtures/required-named-export-program-checker.json"
    ))
    .unwrap();
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
        packets.push(packet(case, &projection));
        let unrecorded = project_program_no_links(&file).unwrap();
        assert_eq!(
            unrecorded.document().as_str(),
            projection.document().as_str()
        );
        assert_eq!(
            unrecorded.map_span(block.span()),
            Err(MappingError::Unrecorded)
        );
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
                .unwrap_or_else(|rejected| panic!("complete TSX: {:?}", rejected.error())),
        );
        assert!(owner.nodes().any(|node| node.source() == Some("<></>")));
        let projection = project_jsx_program(&owner).unwrap();
        assert_eq!(projection.source_kind(), SourceKind::Tsx);
        assert!(core::ptr::eq(projection.file(), owner.file()));
        packets.push(packet(&case, &projection));
        assert!(matches!(
            project_program(owner.file()),
            Err(ProjectionError::UnsupportedProfile)
        ));
        let unrecorded = project_jsx_program_no_links(&owner).unwrap();
        assert_eq!(
            unrecorded.document().as_str(),
            projection.document().as_str()
        );
        assert_eq!(
            unrecorded.map_span(block.span()),
            Err(MappingError::Unrecorded)
        );
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
    // The unchanged helper's actual filename/options/schema are also used by
    // the independently frozen original TS6 observation, before Rust execution.
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
    assert_eq!(evidence["cases"].as_array().unwrap().len(), 12);
    if let Ok(profile) = std::env::var("NEXTEST_PROFILE") {
        assert!(matches!(profile.as_str(), "pr" | "full"));
        let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
            .parent()
            .unwrap()
            .join("nextest")
            .join(profile)
            .join("original-required-named-export-primary.json");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, serde_json::to_vec_pretty(&evidence).unwrap()).unwrap();
    }
}
