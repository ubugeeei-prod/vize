use super::{CrossFileAnalyzer, CrossFileOptions};
use crate::{DependencyEdge, FileId, PropsValidationIssueKind};
use std::path::Path;
use vize_carton::{CompactString, smallvec};
use vize_croquis::{AnalyzerOptions, ScopeId, analysis::ComponentUsage};

#[test]
fn prop_contracts_are_keyed_by_component_identity() {
    assert_prop_contracts_follow_import_targets(false);
}

#[test]
fn prop_contracts_follow_identical_local_import_names() {
    assert_prop_contracts_follow_import_targets(true);
}

#[test]
fn rebuilding_edges_replaces_stale_same_name_fallback() {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_props_validation(true));
    let admin_parent = analyzer.add_file_with_analysis(
        Path::new("AdminParent.vue"),
        "",
        analyze("import Button from './admin/Button.vue'", Some("Button")),
    );
    let shop_parent = analyzer.add_file_with_analysis(
        Path::new("ShopParent.vue"),
        "",
        analyze("import Button from './shop/Button.vue'", Some("Button")),
    );
    let shop_child = analyzer.add_file_with_analysis(
        Path::new("shop/Button.vue"),
        "",
        analyze("defineProps<{ shopId: string }>()", None),
    );

    analyzer.rebuild_component_edges();
    assert!(
        analyzer
            .graph()
            .component_usage()
            .any(|edge| edge == (admin_parent, shop_child))
    );

    let admin_child = analyzer.add_file_with_analysis(
        Path::new("admin/Button.vue"),
        "",
        analyze("defineProps<{ adminId: string }>()", None),
    );
    analyzer.rebuild_component_edges();

    let edges = analyzer.graph().component_usage().collect::<Vec<_>>();
    assert_eq!(edges.len(), 2);
    assert!(edges.contains(&(admin_parent, admin_child)));
    assert!(edges.contains(&(shop_parent, shop_child)));
    assert!(
        !analyzer
            .graph()
            .dependents(shop_child)
            .any(|(source, edge)| {
                source == admin_parent && edge == DependencyEdge::ComponentUsage
            })
    );
    assert_eq!(
        missing_props(&mut analyzer, admin_parent, shop_parent),
        expected_missing_props()
    );
}

fn assert_prop_contracts_follow_import_targets(shared_local_name: bool) {
    let forward = same_basename_required_props(false, shared_local_name);
    let reversed = same_basename_required_props(true, shared_local_name);

    assert_eq!(forward, expected_missing_props());
    assert_eq!(reversed, forward);
}

fn same_basename_required_props(
    reverse_children: bool,
    shared_local_name: bool,
) -> Vec<(&'static str, CompactString)> {
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

    let admin_import = if shared_local_name {
        ("import Button from './admin/Button.vue'", "Button")
    } else {
        (
            "import AdminButton from './admin/Button.vue'",
            "AdminButton",
        )
    };
    let shop_import = if shared_local_name {
        ("import Button from './shop/Button.vue'", "Button")
    } else {
        ("import ShopButton from './shop/Button.vue'", "ShopButton")
    };
    let admin_parent = analyzer.add_file_with_analysis(
        Path::new("AdminParent.vue"),
        "",
        analyze(admin_import.0, Some(admin_import.1)),
    );
    let shop_parent = analyzer.add_file_with_analysis(
        Path::new("ShopParent.vue"),
        "",
        analyze(shop_import.0, Some(shop_import.1)),
    );
    analyzer.rebuild_component_edges();

    missing_props(&mut analyzer, admin_parent, shop_parent)
}

fn missing_props(
    analyzer: &mut CrossFileAnalyzer,
    admin_parent: FileId,
    shop_parent: FileId,
) -> Vec<(&'static str, CompactString)> {
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

fn expected_missing_props() -> Vec<(&'static str, CompactString)> {
    vec![
        ("AdminParent", CompactString::new("adminId")),
        ("ShopParent", CompactString::new("shopId")),
    ]
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
