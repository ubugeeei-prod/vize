//! Totality and direct-analyzer parity for the cross-file Atlas product.

use vize::artifact_graph::{VizeGraphConfig, create_compilation};
use vize::croquis_cf::{
    CrossFileAnalysisInput, CrossFileAnalysisProduct, CrossFileAnalysisRequest, CrossFileAnalyzer,
    CrossFileOffsetRegion, CrossFileOptions,
};

#[test]
fn empty_malformed_and_frontend_neutral_inputs_are_total() {
    let mut empty = create_compilation(VizeGraphConfig::default()).unwrap();
    let anchor = empty.add_source("<cross-file-anchor>", "").unwrap();
    empty
        .set_input::<CrossFileAnalysisInput>(CrossFileAnalysisRequest::new(
            CrossFileOptions::minimal(),
        ))
        .unwrap();
    let result = empty.query::<CrossFileAnalysisProduct>(anchor).unwrap();
    assert_eq!(result.value().result().stats.files_analyzed, 0);
    assert!(result.value().layouts().is_empty());

    let mut compilation = create_compilation(VizeGraphConfig::default()).unwrap();
    let malformed = compilation
        .add_source(
            "Malformed.vue",
            "<template><div /></template><template><span /></template>",
        )
        .unwrap();
    let jsx = compilation
        .add_source("View.jsx", "export const View = () => <main />")
        .unwrap();
    let tsx = compilation
        .add_source(
            "Typed.tsx",
            "export const Typed = (p: { n: number }) => <b>{p.n}</b>",
        )
        .unwrap();
    compilation
        .set_input::<CrossFileAnalysisInput>(CrossFileAnalysisRequest::new(
            CrossFileOptions::minimal(),
        ))
        .unwrap();
    let artifact = compilation
        .query::<CrossFileAnalysisProduct>(malformed)
        .unwrap();
    assert_eq!(artifact.value().result().stats.files_analyzed, 3);
    for source in [malformed, jsx, tsx] {
        let layout = artifact.value().layout_for_source(source).unwrap();
        assert_eq!(layout.source(), source);
    }
    assert_eq!(
        artifact
            .value()
            .layout_for_source(jsx)
            .unwrap()
            .map_offset(CrossFileOffsetRegion::Script, 12),
        12
    );
}

#[test]
fn atlas_raw_module_result_matches_the_direct_analyzer() {
    let options = CrossFileOptions::minimal()
        .with_provide_inject(true)
        .with_circular_dependencies(true);
    let files = [
        (
            "src/parent.ts",
            "import { provide } from 'vue'; provide('theme', 'dark')",
        ),
        (
            "src/child.ts",
            "import { inject } from 'vue'; inject('missing')",
        ),
    ];
    let mut compilation = create_compilation(VizeGraphConfig::default()).unwrap();
    let mut anchor = None;
    for (path, source) in files {
        anchor.get_or_insert(compilation.add_source(path, source).unwrap());
    }
    compilation
        .set_input::<CrossFileAnalysisInput>(CrossFileAnalysisRequest::new(options.clone()))
        .unwrap();
    let atlas = compilation
        .query::<CrossFileAnalysisProduct>(anchor.unwrap())
        .unwrap();

    let mut direct = CrossFileAnalyzer::new(options);
    for (path, source) in files {
        direct.add_file(path, source);
    }
    direct.rebuild_import_edges();
    direct.rebuild_component_edges();
    let direct_result = direct.analyze();
    let direct_tree = direct_result
        .provide_inject_tree
        .as_ref()
        .map(|tree| tree.to_markdown(direct.registry()));
    let atlas_codes: Vec<_> = atlas
        .value()
        .result()
        .diagnostics
        .iter()
        .map(|diagnostic| (diagnostic.code(), diagnostic.message.as_str()))
        .collect();
    let direct_codes: Vec<_> = direct_result
        .diagnostics
        .iter()
        .map(|diagnostic| (diagnostic.code(), diagnostic.message.as_str()))
        .collect();

    assert_eq!(atlas_codes, direct_codes);
    assert_eq!(
        atlas.value().result().stats.files_analyzed,
        direct_result.stats.files_analyzed
    );
    assert_eq!(
        atlas.value().result().stats.dependency_edges,
        direct_result.stats.dependency_edges
    );
    assert_eq!(
        atlas.value().result().complexity_report,
        direct_result.complexity_report
    );
    assert_eq!(atlas.value().provide_inject_tree(), direct_tree.as_deref());
}
