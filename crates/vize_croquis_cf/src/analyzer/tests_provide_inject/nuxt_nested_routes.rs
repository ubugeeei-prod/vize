use super::*;
use crate::{NuxtRenderDirs, PropsValidationIssueKind};
use std::path::PathBuf;
use vize_carton::{CompactString, smallvec};
use vize_croquis::{
    ScopeId,
    analysis::{ComponentUsage, PassedProp, SpreadProp},
};

fn dirs() -> NuxtRenderDirs {
    NuxtRenderDirs {
        src: PathBuf::from("src"),
        pages: PathBuf::from("src/pages"),
        layouts: PathBuf::from("src/layouts"),
        pages_enabled: true,
        static_render_config: false,
    }
}

fn analyzer(options: CrossFileOptions) -> CrossFileAnalyzer {
    let mut analyzer = CrossFileAnalyzer::with_project_root(options, "/repo");
    analyzer.set_nuxt_render_dirs(Some(dirs()));
    analyzer
}

fn with_nuxt_page(script: &str, spread: bool, start: u32) -> vize_croquis::Croquis {
    with_nuxt_page_props(script, &[], spread, start)
}

fn with_nuxt_page_props(
    script: &str,
    props: &[PassedProp],
    spread: bool,
    start: u32,
) -> vize_croquis::Croquis {
    let mut analysis = script_analysis(script, &[]);
    analysis.note_used_component(CompactString::new("NuxtPage"));
    analysis.note_component_usage(ComponentUsage {
        name: CompactString::new("NuxtPage"),
        start,
        end: start + 24,
        props: props.iter().cloned().collect(),
        events: smallvec![],
        slots: smallvec![],
        has_spread_attrs: spread,
        spread_props: if spread {
            smallvec![SpreadProp {
                expression: CompactString::new("{ order }"),
                start: start + 10,
                end: start + 22,
            }]
        } else {
            smallvec![]
        },
        scope_id: ScopeId::ROOT,
        vif_guard: None,
    });
    analysis
}

fn passed_prop(name: &str, value: Option<&str>, is_dynamic: bool) -> PassedProp {
    PassedProp {
        name: CompactString::new(name),
        name_is_dynamic: false,
        value: value.map(CompactString::new),
        start: 8,
        end: 20,
        is_dynamic,
    }
}

fn app_analysis(layout: bool) -> vize_croquis::Croquis {
    let mut analysis = with_nuxt_page("", false, 16);
    if layout {
        analysis.note_used_component(CompactString::new("NuxtLayout"));
    }
    analysis
}

#[test]
fn nested_routes_form_one_nearest_parent_chain_and_forward_spread_props() {
    let mut analyzer = analyzer(
        CrossFileOptions::minimal()
            .with_provide_inject(true)
            .with_props_validation(true),
    );
    let app = analyzer.add_file_with_analysis(
        Path::new("src/app.vue"),
        "<template><NuxtLayout><NuxtPage /></NuxtLayout></template>",
        app_analysis(true),
    );
    let layout = analyzer.add_file_with_analysis(
        Path::new("src/layouts/default.vue"),
        "<template><main><slot /></main></template>",
        script_analysis(
            "import { provide, reactive } from 'vue'; import { NoticeKey } from '../composables/keys'; provide(NoticeKey, reactive({ show(text: string) {} }))",
            &[],
        ),
    );
    let shop = analyzer.add_file_with_analysis(
        Path::new("src/pages/shop.vue"),
        "<template><NuxtPage /></template>",
        with_nuxt_page("", false, 20),
    );
    let orders = analyzer.add_file_with_analysis(
        Path::new("src/pages/shop/orders.vue"),
        "<template><NuxtPage v-bind=\"{ order }\" /></template>",
        with_nuxt_page(
            "import { provide, ref } from 'vue'; import { OrderKey } from '../../composables/keys'; const order = ref('A-1'); provide(OrderKey, order)",
            true,
            24,
        ),
    );
    let detail = analyzer.add_file_with_analysis(
        Path::new("src/pages/shop/orders/detail.vue"),
        "<template><button /></template>",
        script_analysis(
            "import { inject } from 'vue'; import { NoticeKey, OrderKey } from '../../../composables/keys'; defineProps<{ order: string }>(); const notice = inject(NoticeKey)!; const order = inject(OrderKey)!",
            &[],
        ),
    );
    analyzer.rebuild_component_edges();

    let mut edges = analyzer.graph().component_usage().collect::<Vec<_>>();
    edges.sort_unstable();
    let mut expected_edges = vec![
        (app, layout),
        (layout, shop),
        (shop, orders),
        (orders, detail),
    ];
    expected_edges.sort_unstable();
    assert_eq!(edges, expected_edges);

    let result = analyzer.analyze();
    assert!(result.props_validation_issues.is_empty());
    assert!(result.provide_inject_matches.iter().any(|matched| {
        matched.provider == layout && matched.consumer == detail && matched.key == "NoticeKey"
    }));
    assert!(result.provide_inject_matches.iter().any(|matched| {
        matched.provider == orders && matched.consumer == detail && matched.key == "OrderKey"
    }));
    assert!(result.diagnostics.is_empty(), "{:#?}", result.diagnostics);
}

#[test]
fn missing_route_props_point_to_the_parent_nuxt_page_not_the_app() {
    let mut analyzer = analyzer(CrossFileOptions::minimal().with_props_validation(true));
    let app = analyzer.add_file_with_analysis(
        Path::new("src/app.vue"),
        "<template><NuxtPage /></template>",
        app_analysis(false),
    );
    let parent = analyzer.add_file_with_analysis(
        Path::new("src/pages/orders.vue"),
        "<template><NuxtPage /></template>",
        with_nuxt_page("", false, 37),
    );
    let child = analyzer.add_file_with_analysis(
        Path::new("src/pages/orders/[id].vue"),
        "<template><p /></template>",
        script_analysis("defineProps<{ order: string }>()", &[]),
    );
    analyzer.rebuild_component_edges();

    let mut edges = analyzer.graph().component_usage().collect::<Vec<_>>();
    edges.sort_unstable();
    let mut expected_edges = vec![(app, parent), (parent, child)];
    expected_edges.sort_unstable();
    assert_eq!(edges, expected_edges);

    let result = analyzer.analyze();
    assert_eq!(result.props_validation_issues.len(), 1);
    let issue = &result.props_validation_issues[0];
    assert_eq!(issue.parent_file, parent);
    assert_eq!(issue.child_file, child);
    assert_eq!(issue.offset, 37);
    assert!(matches!(
        &issue.kind,
        PropsValidationIssueKind::MissingRequiredProp { prop_name } if prop_name == "order"
    ));
}

#[test]
fn nuxt_page_controls_are_consumed_while_route_attrs_are_forwarded() {
    let mut analyzer = analyzer(CrossFileOptions::minimal().with_props_validation(true));
    analyzer.add_file_with_analysis(
        Path::new("src/app.vue"),
        "<template><NuxtPage /></template>",
        app_analysis(false),
    );
    let parent = analyzer.add_file_with_analysis(
        Path::new("src/pages/orders.vue"),
        "<template><NuxtPage name=\"sidebar\" :transition=\"false\" keepalive :route=\"route\" page-key=\"order\" order=\"A-1\" /></template>",
        with_nuxt_page_props(
            "",
            &[
                passed_prop("name", Some("sidebar"), false),
                passed_prop("transition", Some("false"), true),
                passed_prop("keepalive", None, false),
                passed_prop("route", Some("route"), true),
                passed_prop("pageKey", Some("order"), false),
                passed_prop("page-key", Some("order"), false),
                passed_prop("order", Some("A-1"), false),
            ],
            false,
            42,
        ),
    );
    let child = analyzer.add_file_with_analysis(
        Path::new("src/pages/orders/[id].vue"),
        "<template><p /></template>",
        script_analysis(
            "defineProps<{ name: string; transition?: string; order: string }>()",
            &[],
        ),
    );
    analyzer.rebuild_component_edges();

    let result = analyzer.analyze();
    assert_eq!(result.props_validation_issues.len(), 1);
    let issue = &result.props_validation_issues[0];
    assert_eq!(issue.parent_file, parent);
    assert_eq!(issue.child_file, child);
    assert_eq!(issue.offset, 42);
    assert!(matches!(
        &issue.kind,
        PropsValidationIssueKind::MissingRequiredProp { prop_name } if prop_name == "name"
    ));
    assert_eq!(result.diagnostics.len(), 1, "{:#?}", result.diagnostics);
}
