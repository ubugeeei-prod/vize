use std::path::PathBuf;

use oxc_ast::ast::{ImportDeclarationSpecifier, Statement};
use oxc_span::{SourceType, Span};
use serde_json::json;
use vize_carton::String;

use super::{
    CorsaVueVirtualDocumentOptions, build_vue_virtual_project, install_runtime_stubs, write,
};

fn fixture(
    mode: &str,
    extension: &str,
    specifier: &str,
    newline: &str,
    directory: &str,
) -> (
    tempfile::TempDir,
    super::super::vue_document::CorsaVueVirtualProject,
) {
    let root = tempfile::tempdir().unwrap();
    let app = root.path().join("日本語🍣 quoted'\"root");
    let module = if mode == "nodenext" {
        "nodenext"
    } else {
        "preserve"
    };
    write(&app.join("tsconfig.json"), &json!({"compilerOptions":{"strict":true,"module":module,"moduleResolution":mode,"noEmit":true,"paths":{"#lib/*":[format!("./{directory}/*")]}},"include":["src/**/*"]}).to_string());
    if mode == "nodenext" {
        write(&app.join("package.json"), r#"{"type":"module"}"#);
    }
    let implementation = if extension.starts_with("d.") {
        "export declare function greet(name: string): string;"
    } else if extension == "vue" {
        "<script lang=\"ts\">export const greet = (name: string): string => name;</script>"
    } else {
        "export const greet = (name: string): string => name;"
    };
    write(
        &app.join(format!("{directory}/greet/index.{extension}")),
        implementation,
    );
    let source = format!(
        "<script setup lang=\"ts\">{newline}import {{ greet }} from '{specifier}';{newline}const message = greet('vize');{newline}</script>{newline}<template>{{{{ message }}}}</template>"
    );
    let host = app.join("src/App.vue");
    write(&host, &source);
    install_runtime_stubs(&app);
    let project =
        build_vue_virtual_project(&host, &source, CorsaVueVirtualDocumentOptions::default())
            .unwrap();
    if let Some(capture) = std::env::var_os("VIZE_EDITOR_ALIAS_CAPTURE") {
        let ending = if newline == "\r\n" { "crlf" } else { "lf" };
        let form = if specifier.ends_with("index.ts") {
            "explicit"
        } else {
            "alias"
        };
        let folder = if directory == "src/lib" {
            "ordinary"
        } else {
            "quoted"
        };
        let capture =
            PathBuf::from(capture).join(format!("{mode}-{extension}-{form}-{ending}-{folder}"));
        std::fs::create_dir_all(&capture).unwrap();
        std::fs::write(
            capture.join("generated.vue.ts.txt"),
            project.host.code.as_bytes(),
        )
        .unwrap();
        std::fs::write(
            capture.join("pre-rewrite.vue.ts.txt"),
            project.host.pre_rewrite_code.as_bytes(),
        )
        .unwrap();
        std::fs::write(capture.join("authored.vue.txt"), source.as_bytes()).unwrap();
        std::fs::write(capture.join("module.ts.txt"), implementation).unwrap();
        std::fs::copy(
            app.join("tsconfig.json"),
            capture.join("authored-tsconfig.json.txt"),
        )
        .unwrap();
        std::fs::write(
            capture.join("import-map.txt"),
            format!("{:#?}", project.host.import_source_map),
        )
        .unwrap();
        std::fs::write(capture.join("metadata.json"), serde_json::to_vec_pretty(&json!({"mode":mode,"extension":extension,"specifier":specifier,"directory":directory,"projectRoot":project.session_project_root})).unwrap()).unwrap();
        let before = greet_imports(&project.host.pre_rewrite_code);
        let after = greet_imports(&project.host.code);
        let (_, original) = before.first().unwrap();
        let (_, rewritten) = after.first().unwrap();
        std::fs::write(capture.join("actual-geometry.json"), serde_json::to_vec_pretty(&json!({
            "sourceStart":original.start,"sourceEnd":original.end,
            "generatedStart":rewritten.start,"generatedEnd":rewritten.end,
            "mappedFirstContentByte":project.host.import_source_map.get_original_offset(rewritten.start + 1),
            "expectedFirstContentByte":original.start + 1,
            "mappedAfterLiteral":project.host.import_source_map.get_original_offset(rewritten.end),
            "expectedAfterLiteral":original.end,
            "reverseFirstContentByte":project.host.import_source_map.get_virtual_offset(original.start + 1)
        })).unwrap()).unwrap();
    }
    (root, project)
}

fn greet_imports(code: &str) -> Vec<(String, Span)> {
    let allocator = oxc_allocator::Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, code, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    parsed.program.body.iter().filter_map(|statement| {
        let Statement::ImportDeclaration(import) = statement else { return None };
        import.specifiers.iter().flatten().any(|specifier| matches!(specifier,
            ImportDeclarationSpecifier::ImportSpecifier(named) if named.local.name.as_str() == "greet"
        )).then(|| (String::from(import.source.value.as_str()), import.source.span))
    }).collect()
}

#[test]
fn bundler_ts_alias_uses_exact_extensionless_mirror_and_preserves_import_geometry() {
    for newline in ["\n", "\r\n"] {
        let (_root, project) = fixture("bundler", "ts", "#lib/greet", newline, "src/lib");
        let expected = project
            .session_project_root
            .as_ref()
            .unwrap()
            .join("src/lib/greet/index");
        let before = greet_imports(&project.host.pre_rewrite_code);
        let after = greet_imports(&project.host.code);
        assert_eq!(before.len(), 1);
        assert_eq!(after.len(), 1);
        let (authored, original) = before.first().unwrap();
        let (generated, rewritten) = after.first().unwrap();
        assert_eq!(authored.as_str(), "#lib/greet");
        assert_eq!(
            generated.as_str(),
            expected.to_str().unwrap().replace('\\', "/")
        );
        assert_eq!(
            project
                .host
                .import_source_map
                .get_original_offset(rewritten.start + 1),
            original.start + 1
        );
        assert_eq!(
            project
                .host
                .import_source_map
                .get_original_offset(rewritten.end),
            original.end
        );
        assert!(
            project
                .host
                .materialized_sources
                .iter()
                .any(|source| source.source_path.ends_with("src/lib/greet/index.ts"))
        );
    }
}

#[test]
fn excluded_source_kinds_and_authored_spelling_keep_their_existing_mirror_suffixes() {
    for (mode, extension, specifier, suffix) in [
        ("nodenext", "ts", "#lib/greet", "index.ts"),
        ("bundler", "ts", "#lib/greet/index.ts", "index.ts"),
        ("bundler", "mts", "#lib/greet", "index.mts"),
        ("bundler", "cts", "#lib/greet", "index.cts"),
        ("bundler", "d.ts", "#lib/greet", "index"),
        ("bundler", "vue", "#lib/greet", "index.vue.ts"),
    ] {
        let (_root, project) = fixture(mode, extension, specifier, "\n", "src/lib");
        let expected: PathBuf = project
            .session_project_root
            .as_ref()
            .unwrap()
            .join("src/lib/greet")
            .join(suffix);
        let imports = greet_imports(&project.host.code);
        assert_eq!(imports.len(), 1, "{mode}: {specifier}");
        assert_eq!(
            imports.first().unwrap().0.as_str(),
            expected.to_str().unwrap().replace('\\', "/"),
            "{mode}: {specifier}"
        );
    }
}

#[test]
fn quoted_project_relative_directory_keeps_generated_import_parseable() {
    let directory = "src/日本語🍣 quoted'\"directory/lib";
    let (_root, project) = fixture("bundler", "ts", "#lib/greet", "\r\n", directory);
    let expected = project
        .session_project_root
        .as_ref()
        .unwrap()
        .join(directory)
        .join("greet/index");
    let imports = greet_imports(&project.host.code);
    assert_eq!(imports.len(), 1);
    assert_eq!(
        imports.first().unwrap().0.as_str(),
        expected.to_str().unwrap().replace('\\', "/")
    );
}
