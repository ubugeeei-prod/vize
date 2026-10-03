//! Actual configured JSX/TSX checks borrow the complete original owning File.
#![expect(clippy::unwrap_used, reason = "fixture assertions panic")]

use std::path::{Path, PathBuf};

use corsa::runtime::block_on;
use lsp_types::{DocumentDiagnosticReport, DocumentDiagnosticReportResult};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_canon::{CorsaBridge, CorsaBridgeConfig, OriginalProgramError};
use vize_l0::{Allocator, SourceRoot, Span, cstr, line_index::LineBreaks};
use vize_l2::lang::js::{JsxFile, JsxFileProducer};
use vize_l4::targets::ts::{ProjectionError, SourceKind};

fn owner<'a>(arena: &'a Allocator, source: &'a str, profile: SourceType) -> JsxFile<'a> {
    let block = SourceRoot::new(source).unwrap().whole_block();
    let original = Parser::new(arena, source, profile).parse_observed();
    let mut producer = JsxFileProducer::new(arena, original, block, 17)
        .unwrap_or_else(|rejected| panic!("original admission: {:?}", rejected.error()));
    producer.walk().unwrap();
    producer
        .finish()
        .unwrap_or_else(|rejected| panic!("complete original File: {:?}", rejected.error()))
}

fn repo() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
}

fn backend() -> PathBuf {
    vize_carton::corsa_resolver::resolve_corsa_executable(
        vize_carton::corsa_resolver::CorsaResolveRequest {
            explicit_path: std::env::var_os("CORSA_PATH").as_deref().map(Path::new),
            project_root: Some(repo()),
        },
    )
    .unwrap()
}

fn configuration(root: &Path, force: bool, excluded: bool) {
    std::fs::write(root.join("tsconfig.json"), serde_json::to_vec(&serde_json::json!({
        "compilerOptions":{"noEmit":true,"strict":true,"types":[],"target":"ESNext","module":"ESNext","moduleResolution":"Bundler","moduleDetection":if force {"force"} else {"auto"},"skipLibCheck":true,"allowJs":true,"checkJs":true,"jsx":"preserve"},
        "include":if excluded {vec!["included.*"]} else {vec!["**/*"]}
    })).unwrap()).unwrap();
}

fn bridge(root: &Path) -> CorsaBridge {
    CorsaBridge::with_config(CorsaBridgeConfig {
        corsa_path: Some(backend()),
        working_dir: Some(root.to_path_buf()),
        ..Default::default()
    })
}

fn range(source: &str, start: usize, length: usize) -> serde_json::Value {
    let (sl, sc) = LineBreaks::Lsp.offset_to_position(source, start);
    let (el, ec) = LineBreaks::Lsp.offset_to_position(source, start + length);
    serde_json::json!({"start":{"line":sl,"character":sc},"end":{"line":el,"character":ec}})
}

fn file_uri(path: &Path) -> vize_l0::String {
    let mut uri = vize_l0::String::from("file://");
    for byte in path.to_string_lossy().bytes() {
        if byte.is_ascii_alphanumeric() || b"-._~/".contains(&byte) {
            uri.push(char::from(byte));
        } else {
            uri.push_str(&cstr!("%{byte:02X}"));
        }
    }
    uri
}

#[test]
#[cfg(unix)]
fn real_vue_jsx_and_tsx_keep_complete_raw_diagnostics_types_and_authored_owners() {
    let dependencies = repo().join("docs/node_modules");
    let package: serde_json::Value =
        serde_json::from_slice(&std::fs::read(dependencies.join("vue/package.json")).unwrap())
            .unwrap();
    assert_eq!(package["version"], "3.5.35");
    let vue = dependencies.join("vue").canonicalize().unwrap();
    let declaration = vue
        .parent()
        .unwrap()
        .join("@vue/runtime-dom/dist/runtime-dom.d.ts")
        .canonicalize()
        .unwrap();
    let declaration_text = std::fs::read_to_string(&declaration).unwrap();
    let declaration_id = declaration_text.find("\n    id?: string").unwrap() + 5;
    let related = serde_json::json!([{
        "location": {
            "uri": file_uri(&declaration).as_str(),
            "range": range(&declaration_text, declaration_id, 2),
        },
        "message": "The expected type comes from property 'id' which is declared here on type 'HTMLAttributes & ReservedProps'",
    }]);
    for (annotation, profile, kind, missing_type) in [
        ("", SourceType::jsx(), SourceKind::Jsx, "1"),
        (
            ": number",
            SourceType::tsx().with_module(true),
            SourceKind::Tsx,
            "number",
        ),
    ] {
        let root = tempfile::TempDir::new().unwrap();
        std::os::unix::fs::symlink(&dependencies, root.path().join("node_modules")).unwrap();
        configuration(root.path(), true, false);
        let source = cstr!(
            "/*😀*/ import 'vue/jsx'; const value{annotation}=1;\r\nexport function render() {{ return <div id={{1}}>{{value.missing}}</div>; }}"
        );
        let path = root
            .path()
            .join(cstr!("Original.{}", kind.extension()).as_str());
        std::fs::write(&path, source.as_bytes()).unwrap();
        let arena = Allocator::default();
        let original = owner(&arena, &source, profile);
        let body = original
            .observation()
            .admitted()
            .unwrap()
            .program()
            .body
            .as_ptr();
        let bridge = bridge(root.path());
        block_on(bridge.spawn()).unwrap();
        let result = block_on(bridge.check_original_jsx(&original, &path)).unwrap();
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
                    .response()
                    .projects
                    .iter()
                    .any(|project| project.id == observed.project().id)
            );
        }
        assert!(std::ptr::eq(result.projection().file(), original.file()));
        assert_eq!(result.projection().source_kind(), kind);
        assert_eq!(
            result.projection().document().as_str(),
            cstr!("{source}\n;\nexport {{}};\n")
        );
        assert_eq!(result.source_path(), path.canonicalize().unwrap());
        assert_eq!(
            result.source_uri(),
            cstr!("file://{}", path.canonicalize().unwrap().display())
        );
        assert_eq!(result.source_digest().len(), 64);
        assert_eq!(
            result.diagnostic_configuration_path(),
            root.path().join("tsconfig.json").canonicalize().unwrap()
        );
        let id = source.find("id=").unwrap();
        let missing = source.find("missing").unwrap();
        let DocumentDiagnosticReportResult::Report(DocumentDiagnosticReport::Full(full)) =
            result.report()
        else {
            panic!("complete raw report");
        };
        assert_eq!(
            serde_json::to_value(&full.full_document_diagnostic_report.items).unwrap(),
            serde_json::json!([
                {"range":range(&source,id,2),"severity":1,"code":2322,"source":"ts","message":"Type 'number' is not assignable to type 'string'.","relatedInformation":related},
                {"range":range(&source,missing,7),"severity":1,"code":2339,"source":"ts","message":cstr!("Property 'missing' does not exist on type '{missing_type}'.").as_str()}
            ])
        );
        assert_eq!(
            result.authored_spans(),
            [
                Ok(Span::new(id as u32, id as u32 + 2)),
                Ok(Span::new(missing as u32, missing as u32 + 7))
            ]
        );
        assert_eq!(
            original
                .observation()
                .admitted()
                .unwrap()
                .program()
                .body
                .as_ptr(),
            body
        );
        assert_eq!(original.observation().comments().len(), 1);
        assert_eq!(std::fs::read(&path).unwrap(), source.as_bytes());
        assert!(matches!(
            block_on(bridge.check_original_program(original.file(), &path)),
            Err(OriginalProgramError::Projection(
                ProjectionError::UnsupportedProfile
            ))
        ));
        block_on(bridge.shutdown()).unwrap();
    }
}

#[path = "original_jsx_check/identity.rs"]
mod identity;
#[cfg(unix)]
#[path = "original_jsx_check/stale.rs"]
mod stale;
