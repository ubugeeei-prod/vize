use super::{CrossFileAnalyzer, CrossFileOptions};
use crate::PropsValidationIssueKind;
use std::path::Path;
use vize_carton::{CompactString, smallvec};
use vize_croquis::{AnalyzerOptions, ScopeId, analysis::ComponentUsage};

#[test]
fn prop_contracts_are_keyed_by_component_identity() {
    let forward = same_basename_required_props(false);
    let reversed = same_basename_required_props(true);

    assert_eq!(
        forward,
        vec![
            ("AdminParent", CompactString::new("adminId")),
            ("ShopParent", CompactString::new("shopId")),
        ]
    );
    assert_eq!(reversed, forward);
}

fn same_basename_required_props(reverse_children: bool) -> Vec<(&'static str, CompactString)> {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_props_validation(true));
    let children = [
        (
            Path::new("admin/Button.vue"),
            "defineProps<{ adminId: string }>()",
        ),
        (
            Path::new("shop/Button.vue"),
            "defineProps<{ shopId: string }>()",
        ),
    ];
    let order = if reverse_children { [1, 0] } else { [0, 1] };
    for index in order {
        analyzer.add_file_with_analysis(children[index].0, "", analyze(children[index].1, None));
    }

    let admin_parent = analyzer.add_file_with_analysis(
        Path::new("AdminParent.vue"),
        "",
        analyze(
            "import AdminButton from './admin/Button.vue'",
            Some("AdminButton"),
        ),
    );
    let shop_parent = analyzer.add_file_with_analysis(
        Path::new("ShopParent.vue"),
        "",
        analyze(
            "import ShopButton from './shop/Button.vue'",
            Some("ShopButton"),
        ),
    );
    analyzer.rebuild_component_edges();

    let result = analyzer.analyze();
    let mut missing = result
        .props_validation_issues
        .iter()
        .filter_map(|issue| match &issue.kind {
            PropsValidationIssueKind::MissingRequiredProp { prop_name } => Some((
                if issue.parent_file == admin_parent {
                    "AdminParent"
                } else if issue.parent_file == shop_parent {
                    "ShopParent"
                } else {
                    panic!("unexpected parent file")
                },
                prop_name.clone(),
            )),
            _ => None,
        })
        .collect::<Vec<_>>();
    missing.sort_unstable();
    missing
}

fn analyze(script: &str, component: Option<&str>) -> vize_croquis::Croquis {
    let mut analyzer = vize_croquis::Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    let mut analysis = analyzer.finish();
    if let Some(component) = component {
        analysis
            .used_components
            .insert(CompactString::new(component));
        analysis.component_usages.push(ComponentUsage {
            name: CompactString::new(component),
            start: 0,
            end: 0,
            props: smallvec![],
            events: smallvec![],
            slots: smallvec![],
            has_spread_attrs: false,
            spread_props: smallvec![],
            scope_id: ScopeId::ROOT,
            vif_guard: None,
        });
    }
    analysis
}
