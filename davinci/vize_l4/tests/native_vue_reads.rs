//! Original SFC custody and sealed setup reads reach complete DOM modules.

#[path = "native_vue_reads/maps.rs"]
mod maps;
#[path = "native_vue_reads/refusals.rs"]
mod refusals;

use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::container::vue::DescriptorOptions;
use vize_l1_to_l2::native_file::lower_sfc_native;
use vize_l2::file::vue::VueExposure;
use vize_l3::decision::dom::vue::build_vue_render_decisions;
use vize_l4::{
    module::assemble_template,
    runtime::{Runtime, vocabulary},
    targets::dom::{DomErrorKind, emit_vue},
    write::{NoLinks, Recorded},
};

fn fixtures() -> Result<Vec<serde_json::Value>, serde_json::Error> {
    [
        include_str!("fixtures/vue-setup-let/let-interpolation.json"),
        include_str!("fixtures/vue-setup-let/let-var-prop-binary.json"),
        include_str!("fixtures/vue-setup-let/let-var-class-style.json"),
        include_str!("fixtures/vue-setup-let/let-var-class-style-text.json"),
        include_str!("fixtures/vue-setup-let/unicode-let-var.json"),
        include_str!("fixtures/vue-setup-let/commented-let-prop-text.json"),
    ]
    .into_iter()
    .map(serde_json::from_str)
    .collect()
}

fn options() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: vize_l1::SurfaceParseOptions::default(),
    }
}

#[test]
fn original_sfc_leaf_reads_match_complete_modules_and_keep_authentic_owners() {
    let mut captured = Vec::new();
    let mut complete = 0;
    let mut refused = 0;
    for fixture in fixtures().unwrap() {
        let arena = Allocator::default();
        let source = fixture["source"].as_str().unwrap();
        let observed = lower_sfc_native(&arena, source, options());
        assert!(observed.issues().is_empty(), "{:?}", observed.issues());
        let native = observed.admitted().unwrap();
        assert!(core::ptr::eq(native.observation(), &observed));
        let file = native.file().file();
        assert!(core::ptr::eq(file.artifact().source(), source));
        assert!(file.is_complete());
        let descriptor = observed.descriptor().admitted().unwrap();
        let setup = descriptor.setup().unwrap();
        let script = observed.scripts().first().unwrap();
        let program = script.syntax().unwrap().admitted_program().unwrap();
        let exposure = VueExposure::checked(file, setup, program).unwrap();
        let analysis = build_vue_render_decisions(&exposure).unwrap();
        assert!(core::ptr::eq(analysis.file(), file));
        assert!(core::ptr::eq(analysis.artifact(), file.artifact()));
        assert!(analysis.dom().unwrap().unsupported().is_empty());
        let template = observed.template().unwrap();
        assert!(template.produced().unwrap().is_supported());
        assert!(template.holes().is_empty());
        assert!(template.diagnostics().is_empty());
        for embed in template.embeds() {
            let resolution = analysis
                .expression(embed.node.unwrap())
                .unwrap()
                .resolution();
            let table = resolution.table().unwrap();
            assert!(core::ptr::eq(resolution.file(), file));
            assert_eq!(resolution.scope(), Some(exposure.scope()));
            assert!(core::ptr::eq(
                table.expression().ast,
                embed.syntax.expression().unwrap()
            ));
        }
        let recorded = emit_vue::<Recorded>(&analysis);
        let plain = emit_vue::<NoLinks>(&analysis);
        if matches!(
            fixture["id"].as_str().unwrap(),
            "let-var-prop-binary" | "unicode-let-var"
        ) {
            let error = recorded.unwrap_err();
            assert_eq!(plain.unwrap_err(), error);
            assert_eq!(error.kind, DomErrorKind::UncertifiedExpressionSpelling);
            let row = analysis.expression(error.node.unwrap()).unwrap();
            assert_eq!(
                error.span,
                row.resolution().table().unwrap().expression().span
            );
            captured.push(serde_json::json!({
                "id": fixture["id"], "source": source, "outcome": "typed_refusal",
                "kind": "UncertifiedExpressionSpelling", "span": [error.span.start, error.span.end],
            }));
            refused += 1;
            continue;
        }
        let recorded = assemble_template(recorded.unwrap(), vocabulary(Runtime::VueDom)).unwrap();
        let plain = assemble_template(plain.unwrap(), vocabulary(Runtime::VueDom)).unwrap();
        assert_eq!(recorded.text.as_str(), fixture["code"].as_str().unwrap());
        assert_eq!(plain.text, recorded.text);
        assert_eq!(plain.helpers, recorded.helpers);
        let document = recorded.into_document();
        maps::check(&analysis, template, &document, source).unwrap();
        let map: serde_json::Value =
            serde_json::from_str(&document.source_map("VueSetupLet.vue", source)).unwrap();
        assert_eq!(map["sourcesContent"], serde_json::json!([source]));
        captured.push(serde_json::json!({
            "id": fixture["id"], "source": source, "outcome": "complete_module",
            "code": document.as_str(), "map": map,
            "nodes": analysis.artifact().node_count(), "expressions": template.embeds().len(),
        }));
        complete += 1;
    }
    assert_eq!((complete, refused), (4, 2));
    if let Ok(path) = std::env::var("VIZE_L4_VUE_READ_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&captured).unwrap()).unwrap();
    }
}
