use std::path::Path;

use vize_carton::{CompactString, smallvec};
use vize_croquis::facts::spec::agreement::Agreement;
use vize_croquis::{AnalyzerOptions, ScopeId, analysis::ComponentUsage};
use vize_davinci::fact::FactGroup;

use super::{ProjectFacts, RenderTree, RenderTreeReader, render_tree::evaluate};
use crate::{CrossFileAnalyzer, CrossFileOptions, PropsValidationIssueKind};

fn script(source: &str, component: Option<&str>) -> vize_croquis::Croquis {
    let mut analyzer = vize_croquis::Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(source);
    let mut analysis = analyzer.finish();
    if let Some(component) = component {
        analysis
            .used_components
            .insert(CompactString::new(component));
        analysis.component_usages.push(ComponentUsage {
            name: CompactString::new(component),
            start: 4,
            end: 4 + component.len() as u32,
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

fn missing_props(analyzer: &mut CrossFileAnalyzer) -> Vec<(String, String)> {
    analyzer.rebuild_component_edges();
    let result = analyzer.analyze();
    let mut missing = result
        .props_validation_issues
        .iter()
        .filter_map(|issue| match &issue.kind {
            PropsValidationIssueKind::MissingRequiredProp { prop_name } => {
                let parent = analyzer
                    .registry()
                    .get(issue.parent_file)
                    .map(|entry| entry.path.display().to_string())
                    .unwrap_or_default();
                Some((parent, prop_name.to_string()))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    missing.sort();
    missing
}

fn edges(analyzer: &CrossFileAnalyzer) -> Vec<(u32, u32, String)> {
    let mut facts = ProjectFacts::new(analyzer.registry());
    let table = facts
        .prepare::<RenderTreeReader>()
        .get::<RenderTree>()
        .expect("demand");
    let mut rows: Vec<_> = table
        .iter()
        .map(|(key, sites)| {
            (
                key.caller.as_u32(),
                key.target.as_u32(),
                key.export_name.to_string(),
                sites.len(),
            )
        })
        .collect();
    rows.sort();
    rows.into_iter()
        .map(|(c, t, name, _)| (c, t, name))
        .collect()
}

#[test]
fn aliased_import_keeps_the_exported_name() {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_props_validation(true));
    analyzer.add_file_with_analysis(
        Path::new("admin/Foo.vue"),
        "",
        script("defineProps<{ title: string }>()", None),
    );
    let parent = analyzer.add_file_with_analysis(
        Path::new("Parent.vue"),
        "",
        script("import { Foo as Bar } from './admin/Foo.vue'", Some("Bar")),
    );
    let foo = analyzer
        .registry()
        .get_id(Path::new("admin/Foo.vue"))
        .unwrap();
    analyzer.rebuild_component_edges();

    let row = edges(&analyzer)
        .into_iter()
        .find(|edge| edge.0 == parent.as_u32())
        .expect("parent edge");
    assert_eq!(row.1, foo.as_u32());
    assert_eq!(row.2, "Foo");
    assert_eq!(
        missing_props(&mut analyzer),
        vec![("Parent.vue".to_string(), "title".to_string())]
    );
}

#[test]
fn reexport_resolves_to_the_original_file() {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_props_validation(true));
    analyzer.add_file(
        Path::new("barrel.ts"),
        "export { default as Button } from './shop/Button.vue'\n",
    );
    analyzer.add_file_with_analysis(
        Path::new("shop/Button.vue"),
        "",
        script("defineProps<{ label: string }>()", None),
    );
    let parent = analyzer.add_file_with_analysis(
        Path::new("Parent.vue"),
        "",
        script("import { Button } from './barrel'", Some("Button")),
    );
    let button = analyzer
        .registry()
        .get_id(Path::new("shop/Button.vue"))
        .unwrap();
    analyzer.rebuild_component_edges();

    let row = edges(&analyzer)
        .into_iter()
        .find(|edge| edge.0 == parent.as_u32())
        .expect("parent edge");
    assert_eq!(row.1, button.as_u32());
    assert_eq!(row.2, "default");
    assert_eq!(
        missing_props(&mut analyzer),
        vec![("Parent.vue".to_string(), "label".to_string())]
    );
}

#[test]
fn same_basename_edges_ignore_registration_order() {
    let forward = basename_props(false);
    let reversed = basename_props(true);
    assert_eq!(forward, reversed);
    assert_eq!(
        forward,
        vec![
            ("AdminParent.vue".to_string(), "adminId".to_string()),
            ("ShopParent.vue".to_string(), "shopId".to_string()),
        ]
    );
}

fn basename_props(reverse_children: bool) -> Vec<(String, String)> {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_props_validation(true));
    let children = [
        ("admin/Button.vue", "defineProps<{ adminId: string }>()"),
        ("shop/Button.vue", "defineProps<{ shopId: string }>()"),
    ];
    let order = if reverse_children { [1, 0] } else { [0, 1] };
    for index in order {
        analyzer.add_file_with_analysis(
            Path::new(children[index].0),
            "",
            script(children[index].1, None),
        );
    }
    analyzer.add_file_with_analysis(
        Path::new("AdminParent.vue"),
        "",
        script("import Button from './admin/Button.vue'", Some("Button")),
    );
    analyzer.add_file_with_analysis(
        Path::new("ShopParent.vue"),
        "",
        script("import Button from './shop/Button.vue'", Some("Button")),
    );
    missing_props(&mut analyzer)
}

#[test]
fn rebuilding_drops_the_stale_name_fallback() {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_props_validation(true));
    let admin_parent = analyzer.add_file_with_analysis(
        Path::new("AdminParent.vue"),
        "",
        script("import Button from './admin/Button.vue'", Some("Button")),
    );
    let shop_parent = analyzer.add_file_with_analysis(
        Path::new("ShopParent.vue"),
        "",
        script("import Button from './shop/Button.vue'", Some("Button")),
    );
    let shop_child = analyzer.add_file_with_analysis(
        Path::new("shop/Button.vue"),
        "",
        script("defineProps<{ shopId: string }>()", None),
    );
    analyzer.rebuild_component_edges();
    assert!(
        edges(&analyzer)
            .iter()
            .any(|edge| edge.0 == admin_parent.as_u32() && edge.1 == shop_child.as_u32())
    );

    let admin_child = analyzer.add_file_with_analysis(
        Path::new("admin/Button.vue"),
        "",
        script("defineProps<{ adminId: string }>()", None),
    );
    analyzer.rebuild_component_edges();
    let rows = edges(&analyzer);
    assert!(
        rows.iter()
            .any(|edge| { edge.0 == admin_parent.as_u32() && edge.1 == admin_child.as_u32() })
    );
    assert!(
        rows.iter()
            .any(|edge| { edge.0 == shop_parent.as_u32() && edge.1 == shop_child.as_u32() })
    );
    assert!(
        !rows
            .iter()
            .any(|edge| edge.0 == admin_parent.as_u32() && edge.1 == shop_child.as_u32())
    );
}

#[test]
fn the_naive_evaluator_matches_the_fact_table() {
    let mut analyzer = CrossFileAnalyzer::new(CrossFileOptions::default());
    analyzer.add_file_with_analysis(
        Path::new("admin/Foo.vue"),
        "",
        script("defineProps<{ title: string }>()", None),
    );
    analyzer.add_file_with_analysis(
        Path::new("Parent.vue"),
        "",
        script("import { Foo as Bar } from './admin/Foo.vue'", Some("Bar")),
    );
    let registry = analyzer.registry();
    let spec_table = evaluate(registry);
    let mut facts = ProjectFacts::new(registry);
    let production = facts
        .prepare::<RenderTreeReader>()
        .get::<RenderTree>()
        .expect("demand");
    let spec_rows: Vec<_> = spec_table.iter().collect();
    let production_rows: Vec<_> = production.iter().collect();
    let mut run = Agreement::default();
    let divergence = (spec_rows != production_rows).then(|| CompactString::new("render-tree"));
    run.compare(production.len(), divergence);
    run.verdict("render-tree", "aliased import")
        .unwrap_or_else(|message| panic!("{message}"));
    assert_eq!(RenderTree::NAME, "render-tree");
}
