//! The opt-in Canon consumer checks the original completed Module and identity.
#![expect(clippy::unwrap_used, reason = "fixture assertions panic")]

use std::path::{Path, PathBuf};

use corsa::runtime::block_on;
use lsp_types::{DocumentDiagnosticReport, DocumentDiagnosticReportResult};
use vize_canon::corsa_bridge::{CorsaBridge, CorsaBridgeConfig, OriginalProgramError};
use vize_l0::{Allocator, SourceRoot, line_index::LineBreaks};
use vize_l1::embed::{
    EmbedSource, Lang,
    syntax::{ProgramOptions, parse_program_once},
};
use vize_l2::{
    file::FileArtifact,
    lang::js::{FileProducer, ProgramInput, ProgramScope},
};
use vize_l4::targets::ts::ProjectionError;

#[path = "original_program_check/unused_history.rs"]
mod unused_history;

fn file<'a>(arena: &'a Allocator, source: &'a str, lang: Lang) -> FileArtifact<'a> {
    file_options(arena, source, ProgramOptions::module(lang))
}

fn file_options<'a>(
    arena: &'a Allocator,
    source: &'a str,
    options: ProgramOptions,
) -> FileArtifact<'a> {
    let block = SourceRoot::new(source).unwrap().whole_block();
    let syntax = parse_program_once(
        arena,
        EmbedSource::authored(source, block.span()).unwrap(),
        options,
    );
    let mut producer = FileProducer::new(arena, source).unwrap();
    producer
        .program(
            ProgramInput::checked(syntax.admitted_program().unwrap(), block, 17).unwrap(),
            ProgramScope::Module,
        )
        .unwrap();
    producer.finish().unwrap()
}

fn project(root: &Path, force: bool) -> CorsaBridge {
    std::fs::write(root.join("tsconfig.json"), serde_json::to_vec(&serde_json::json!({
        "compilerOptions": {"noEmit":true,"strict":true,"types":[],"target":"ESNext","module":"ESNext","moduleResolution":"Bundler","moduleDetection":if force {"force"} else {"auto"},"skipLibCheck":true,"allowJs":true,"checkJs":true},
        "include":["**/*"]
    })).unwrap()).unwrap();
    CorsaBridge::with_config(CorsaBridgeConfig {
        working_dir: Some(root.to_path_buf()),
        ..Default::default()
    })
}

fn backend() -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    vize_carton::corsa_resolver::resolve_corsa_executable(
        vize_carton::corsa_resolver::CorsaResolveRequest {
            explicit_path: std::env::var_os("CORSA_PATH").as_deref().map(Path::new),
            project_root: Some(root),
        },
    )
    .unwrap()
}

fn real_bridge(root: &Path) -> CorsaBridge {
    CorsaBridge::with_config(CorsaBridgeConfig {
        corsa_path: Some(backend()),
        working_dir: Some(root.to_path_buf()),
        ..Default::default()
    })
}

#[test]
fn real_configured_original_programs_preserve_complete_diagnostics_and_source() {
    let fixtures: serde_json::Value = serde_json::from_str(include_str!(
        "../../../davinci/vize_l4/tests/fixtures/program-checker.json"
    ))
    .unwrap();
    for case in fixtures["cases"].as_array().unwrap() {
        let arena = Allocator::default();
        let root = tempfile::TempDir::new().unwrap();
        drop(project(root.path(), true));
        let bridge = CorsaBridge::with_config(CorsaBridgeConfig {
            corsa_path: Some(backend()),
            working_dir: Some(root.path().to_path_buf()),
            ..Default::default()
        });
        let source = case["source"].as_str().unwrap();
        let js = case["kind"] == "js";
        let source_path = root
            .path()
            .join(if js { "source.mjs" } else { "source.ts" });
        std::fs::write(&source_path, source).unwrap();
        let original = file(&arena, source, if js { Lang::Js } else { Lang::Ts });
        assert!(
            original.is_complete(),
            "{}: {:?}",
            case["id"],
            original.issues()
        );
        block_on(bridge.spawn()).unwrap();
        let result = block_on(bridge.check_original_program(&original, &source_path)).unwrap();
        assert_diagnosing_options(&result);
        assert!(std::ptr::eq(result.projection().file(), &original));
        assert_eq!(result.source_path(), source_path.canonicalize().unwrap());
        assert_eq!(
            result.source_uri(),
            vize_l0::cstr!("file://{}", source_path.canonicalize().unwrap().display())
        );
        assert_eq!(result.source_digest().len(), 64);
        assert_eq!(
            result.diagnostic_configuration_path(),
            root.path().join("tsconfig.json").canonicalize().unwrap()
        );
        let DocumentDiagnosticReportResult::Report(DocumentDiagnosticReport::Full(report)) =
            result.report()
        else {
            panic!("complete report required")
        };
        let diagnostics = &report.full_document_diagnostic_report.items;
        let mut expected = case["diagnostics"].as_array().unwrap().iter().map(|expected| {
            let start = u32::try_from(expected["start"].as_u64().unwrap()).unwrap();
            let end = start + u32::try_from(expected["length"].as_u64().unwrap()).unwrap();
            let start_byte = vize_l0::line_index::utf16_offset(source,start).unwrap();
            let end_byte = vize_l0::line_index::utf16_offset(source,end).unwrap();
            let (sl, sc) = LineBreaks::Lsp.offset_to_position(source,start_byte);
            let (el, ec) = LineBreaks::Lsp.offset_to_position(source,end_byte);
            serde_json::json!({"range":{"start":{"line":sl,"character":sc},"end":{"line":el,"character":ec}},"severity":1,"code":expected["code"],"source":"ts","message":expected["message"]})
        }).collect::<Vec<_>>();
        if matches!(case["id"].as_str(), Some("js-jsdoc" | "ts-jsdoc-ignored")) {
            expected.push(serde_json::json!({"range":{"start":{"line":0,"character":26},"end":{"line":0,"character":29}},"severity":4,"code":6133,"source":"ts","message":"'msg' is declared but its value is never read."}));
        }
        assert_eq!(
            serde_json::to_value(diagnostics).unwrap(),
            serde_json::json!(expected),
            "{}: entire raw vector",
            case["id"]
        );
        assert_eq!(result.authored_spans().len(), diagnostics.len());
        for (span, expected) in result
            .authored_spans()
            .iter()
            .zip(case["diagnostics"].as_array().unwrap())
        {
            assert_eq!(
                source.get(span.unwrap().start as usize..span.unwrap().end as usize),
                expected["authored"].as_str()
            );
        }
        assert_eq!(std::fs::read(&source_path).unwrap(), source.as_bytes());
        block_on(bridge.shutdown()).unwrap();
    }
}

#[test]
fn source_identity_and_module_goal_refuse_before_backend() {
    let arena = Allocator::default();
    let source = "const x=1; x.missing;";
    let original = file(&arena, source, Lang::Ts);
    let root = tempfile::TempDir::new().unwrap();
    let bridge = project(root.path(), true);
    let path = root.path().join("source.ts");
    std::fs::write(&path, "const x=2;").unwrap();
    assert!(matches!(
        block_on(bridge.check_original_program(&original, &path)),
        Err(OriginalProgramError::SourceChanged)
    ));
    std::fs::write(&path, source).unwrap();
    assert!(matches!(
        block_on(bridge.check_original_program(&original, Path::new("source.ts"))),
        Err(OriginalProgramError::InvalidSourcePath)
    ));
    let wrong = root.path().join("source.js");
    std::fs::write(&wrong, source).unwrap();
    assert!(matches!(
        block_on(bridge.check_original_program(&original, &wrong)),
        Err(OriginalProgramError::SourceKindMismatch)
    ));
    let outside = tempfile::TempDir::new().unwrap();
    let foreign = outside.path().join("source.ts");
    std::fs::write(&foreign, source).unwrap();
    assert!(matches!(
        block_on(bridge.check_original_program(&original, &foreign)),
        Err(OriginalProgramError::OutsideProject)
    ));
    drop(project(root.path(), false));
    let bridge = real_bridge(root.path());
    block_on(bridge.spawn()).unwrap();
    assert!(matches!(
        block_on(bridge.check_original_program(&original, &path)),
        Err(OriginalProgramError::UnsupportedModuleGoal)
    ));
    let mts = root.path().join("source.mts");
    std::fs::write(&mts, source).unwrap();
    assert!(block_on(bridge.check_original_program(&original, &mts)).is_ok());
    let jsx = file_options(
        &arena,
        "const x=<div/>;",
        ProgramOptions {
            jsx: true,
            ..ProgramOptions::module(Lang::Js)
        },
    );
    assert!(matches!(
        block_on(bridge.check_original_program(&jsx, &path)),
        Err(OriginalProgramError::Projection(
            ProjectionError::UnsupportedProfile
        ))
    ));
    for name in ["source.d.ts", "source.d.mts"] {
        let declaration = root.path().join(name);
        std::fs::write(&declaration, source).unwrap();
        assert!(matches!(
            block_on(bridge.check_original_program(&original, &declaration)),
            Err(OriginalProgramError::SourceKindMismatch)
        ));
    }
    block_on(bridge.shutdown()).unwrap();
}

#[test]
fn actual_project_membership_inherited_options_and_relative_imports_are_retained() {
    let root = tempfile::TempDir::new().unwrap();
    std::fs::write(root.path().join("base.json"), r#"{"compilerOptions":{"allowJs":true,"checkJs":true,"moduleDetection":"force","strict":false,"types":[],"noEmit":true,"module":"ESNext","moduleResolution":"Bundler","target":"ESNext"}}"#).unwrap();
    std::fs::write(
        root.path().join("tsconfig.json"),
        r#"{"extends":"./base.json","include":["source.mjs","source.ts"]}"#,
    )
    .unwrap();
    std::fs::write(root.path().join("dep.mjs"), "export const value=1;").unwrap();
    let js =
        "import { value } from './dep.mjs'; value.missing; function local() { return 1; } local();";
    let ts = "const value=null; value.toFixed();";
    std::fs::write(root.path().join("source.mjs"), js).unwrap();
    std::fs::write(root.path().join("source.ts"), ts).unwrap();
    std::fs::write(root.path().join("excluded.ts"), ts).unwrap();
    let bridge = real_bridge(root.path());
    block_on(bridge.spawn()).unwrap();
    let arena = Allocator::default();
    let original = file(&arena, js, Lang::Js);
    assert!(original.is_complete(), "{:?}", original.issues());
    let result =
        block_on(bridge.check_original_program(&original, &root.path().join("source.mjs")))
            .unwrap();
    let report = serde_json::to_value(result.report()).unwrap();
    assert_eq!(report["items"].as_array().unwrap().len(), 1);
    assert_eq!(report["items"][0]["code"], 2339);
    assert_eq!(
        report["items"][0]["message"],
        "Property 'missing' does not exist on type '1'."
    );
    assert_eq!(
        js.get(
            result.authored_spans()[0].unwrap().start as usize
                ..result.authored_spans()[0].unwrap().end as usize
        ),
        Some("missing")
    );
    assert_eq!(result.configuration().options["strict"], false);
    assert_diagnosing_options(&result);
    let original = file(&arena, ts, Lang::Ts);
    let result =
        block_on(bridge.check_original_program(&original, &root.path().join("source.ts"))).unwrap();
    assert!(
        result.authored_spans().is_empty(),
        "strict:false must come from the real inherited config"
    );
    assert!(matches!(
        block_on(bridge.check_original_program(&original, &root.path().join("excluded.ts"))),
        Err(OriginalProgramError::UnconfiguredSource)
    ));
    // Reuse the same bridge after a real inherited-configuration revision.
    let base_path = root.path().join("base.json");
    let strict = std::fs::read_to_string(&base_path)
        .unwrap()
        .replace("\"strict\":false", "\"strict\":true");
    std::fs::write(&base_path, strict).unwrap();
    let revised =
        block_on(bridge.check_original_program(&original, &root.path().join("source.ts"))).unwrap();
    assert_eq!(revised.configuration().options["strict"], true);
    assert_diagnosing_options(&revised);
    assert_eq!(
        revised.diagnostic_configuration_path(),
        root.path().join("tsconfig.json").canonicalize().unwrap()
    );
    let report = serde_json::to_value(revised.report()).unwrap();
    assert_eq!(report["items"].as_array().unwrap().len(), 1);
    assert_eq!(report["items"][0]["code"], 18047);
    assert_eq!(report["items"][0]["message"], "'value' is possibly 'null'.");
    assert_eq!(
        std::fs::read_to_string(root.path().join("source.mjs")).unwrap(),
        js
    );
    assert_eq!(
        std::fs::read_to_string(root.path().join("source.ts")).unwrap(),
        ts
    );
    block_on(bridge.shutdown()).unwrap();
}

fn assert_diagnosing_options(result: &vize_canon::OriginalProgramCheck<'_, '_>) {
    let custody = result.diagnosing_configuration();
    assert!(!custody.session().session_id.is_empty());
    for observed in [custody.before(), custody.after()] {
        assert_eq!(
            observed.project().compiler_options,
            result.configuration().options
        );
        assert_eq!(
            Path::new(&observed.project().config_file_name),
            result.diagnostic_configuration_path()
        );
        assert!(
            observed
                .projects()
                .iter()
                .any(|project| project.id == observed.project().id)
        );
    }
    // The pinned API allocates a new actual snapshot even for a no-op refresh.
    assert_ne!(custody.before().handle(), custody.after().handle());
}
