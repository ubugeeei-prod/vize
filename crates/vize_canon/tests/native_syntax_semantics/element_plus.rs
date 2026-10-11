//! Whole pinned ElementPlus projection controls.

use super::{native, retain_packet};
use std::{fs, path::Path, process::Command};

#[test]
fn full_pinned_element_plus_projections_parse_through_their_original_import_graph() {
    let Some(_native) = native() else { return };
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let original_root = repo.join("tests/_fixtures/_git/element-plus");
    let pin = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&original_root)
        .output()
        .unwrap();
    assert!(pin.status.success());
    let pin = String::from_utf8(pin.stdout).unwrap();
    assert_eq!(pin, "20298a21ae54a5417955373b80130ae7c548a6bb\n");
    let corpus = repo.join("tests/_fixtures/differential/typechecker/quoted-prop-setup-anchors");
    for name in ["popover", "tooltip"] {
        let original_path =
            original_root.join(format!("packages/components/{name}/src/{name}.vue"));
        let source = fs::read_to_string(&original_path).unwrap();
        assert_eq!(
            source,
            fs::read_to_string(corpus.join(format!("{name}.vue.txt"))).unwrap()
        );
        let generated = vize_canon::batch::generate_vue_document_virtual_ts(
            &original_path,
            &source,
            &vize_canon::virtual_ts::VirtualTsOptions::default(),
            &vize_canon::batch::ImportRewriter::new(),
            false,
        )
        .unwrap();
        let before =
            fs::read_to_string(corpus.join(format!("{name}.original-h.vue.ts.txt"))).unwrap();
        let allocator = oxc_allocator::Allocator::default();
        let before_parse =
            oxc_parser::Parser::new(&allocator, &before, oxc_span::SourceType::ts()).parse();
        let after_parse = oxc_parser::Parser::new(
            &allocator,
            &generated.pre_rewrite_code,
            oxc_span::SourceType::ts(),
        )
        .parse();
        retain_packet(
            &format!("original-element-plus-{name}"),
            &serde_json::json!({
                "upstreamPin":pin, "originalRoot":original_root, "originalPath":original_path,
                "wholeOriginalRootConfig":fs::read_to_string(original_root.join("tsconfig.json")).unwrap(),
                "wholeOriginalPackageManifest":fs::read_to_string(original_root.join("package.json")).unwrap(),
                "wholeAuthoredSource":source, "wholeOriginalProducerProjection":before,
                "wholeFreshProjection":generated.pre_rewrite_code.as_str(),
                "wholeFreshRewrittenProjection":generated.code.as_str(),
                "wholeFreshMapping":format!("{:?}",generated.mapping),
                "wholeFreshImportMap":format!("{:?}",generated.import_source_map),
                "wholeBeforeParserDiagnostics":format!("{:#?}",before_parse.diagnostics),
                "wholeAfterParserDiagnostics":format!("{:#?}",after_parse.diagnostics),
            }),
        );
        assert!(
            !before_parse.diagnostics.is_empty(),
            "whole original producer is the regression control"
        );
        assert_eq!(
            after_parse.diagnostics.len(),
            0,
            "{name}: {:#?}",
            after_parse.diagnostics
        );
    }
}
