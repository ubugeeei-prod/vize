use super::{CrossFileAnalyzer, CrossFileOptions};
use std::path::Path;
use vize_croquis::AnalyzerOptions;

fn script_analysis(script: &str, used_components: &[&str]) -> vize_croquis::Croquis {
    let mut analyzer = vize_croquis::Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    for component in used_components {
        analyzer
            .croquis_mut()
            .note_used_component(vize_carton::CompactString::new(*component));
    }
    analyzer.finish()
}

#[test]
fn an_aliased_import_pairs_through_the_imported_file() {
    use vize_carton::{CompactString, smallvec};
    use vize_croquis::ScopeId;
    use vize_croquis::analysis::ComponentUsage;

    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_provide_inject(true));
    analyzer.add_file_with_analysis(
        Path::new("Provider.vue"),
        "",
        script_analysis(
            "import { provide } from 'vue'; provide('theme', 'dark')",
            &["Parent"],
        ),
    );
    let mut parent = script_analysis(
        "import { Button as ShopButton } from './shop/Button.vue'",
        &[],
    );
    parent.note_component_usage(ComponentUsage {
        name: CompactString::new("ShopButton"),
        start: 1,
        end: 11,
        props: smallvec![],
        events: smallvec![],
        slots: smallvec![],
        has_spread_attrs: false,
        spread_props: smallvec![],
        scope_id: ScopeId::ROOT,
        vif_guard: None,
    });
    analyzer.add_file_with_analysis(Path::new("Parent.vue"), "", parent);
    analyzer.add_file_with_analysis(
        Path::new("shop/Button.vue"),
        "",
        script_analysis(
            "import { inject } from 'vue'; const theme = inject('theme')",
            &[],
        ),
    );
    analyzer.add_file_with_analysis(
        Path::new("admin/Button.vue"),
        "",
        script_analysis(
            "import { provide } from 'vue'; provide('theme', 'light')",
            &[],
        ),
    );
    analyzer.rebuild_component_edges();
    let result = analyzer.analyze();
    let shop = analyzer
        .registry()
        .get_id(Path::new("shop/Button.vue"))
        .expect("shop button");
    let provider = analyzer
        .registry()
        .get_id(Path::new("Provider.vue"))
        .expect("provider");
    let matched = result
        .provide_inject_matches
        .iter()
        .any(|row| row.consumer == shop && row.provider == provider && row.key.as_str() == "theme");
    assert!(
        matched,
        "alias ShopButton must pair with Provider through shop/Button.vue, not admin/Button.vue: {:?}",
        result
            .provide_inject_matches
            .iter()
            .map(|row| (
                row.provider.as_u32(),
                row.consumer.as_u32(),
                row.key.as_str()
            ))
            .collect::<Vec<_>>()
    );
}

mod basic;
mod patterns;
mod playground;
mod provider_context;
mod provider_reactivity;
mod tree;
mod tree_cycle;
mod tree_diamond;
mod tree_partial;
mod tree_shared;
mod tree_typed;
