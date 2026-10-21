use super::*;
use crate::diagnostics::CrossFileDiagnosticKind;
use vize_carton::{CompactString, smallvec};
use vize_croquis::{ScopeId, analysis::ComponentUsage};

fn note_component_usage(analysis: &mut vize_croquis::Croquis, name: &str, start: u32) {
    analysis.note_component_usage(ComponentUsage {
        name: CompactString::new(name),
        start,
        end: start + u32::try_from(name.len()).unwrap(),
        props: smallvec![],
        events: smallvec![],
        slots: smallvec![],
        has_spread_attrs: false,
        spread_props: smallvec![],
        scope_id: ScopeId::ROOT,
        vif_guard: None,
    });
}

#[test]
fn nuxt_default_layout_is_an_ancestor_of_pages_and_their_children() {
    let mut analyzer = CrossFileAnalyzer::new(
        CrossFileOptions::minimal()
            .with_provide_inject(true)
            .with_reactivity_tracking(true),
    );
    let button = analyzer.add_file_with_analysis(
        Path::new("packages/web/app/components/SaveButton.vue"),
        "",
        script_analysis(
            "import { inject } from 'vue'; import { ToastKey } from '../composables/toast'; const { show } = inject(ToastKey)!; show('saved')",
            &[],
        ),
    );
    analyzer.add_file_with_analysis(
        Path::new("packages/web/app/pages/index.vue"),
        "",
        script_analysis(
            "import SaveButton from '../components/SaveButton.vue'",
            &["SaveButton"],
        ),
    );
    let layout = analyzer.add_file_with_analysis(
        Path::new("packages/web/app/layouts/default.vue"),
        "<template><main><slot /></main></template>",
        script_analysis(
            "import { provide } from 'vue'; import { ToastKey } from '../composables/toast'; provide(ToastKey, { show: (message: string) => console.log(message) })",
            &[],
        ),
    );
    analyzer.add_file_with_analysis(
        Path::new("packages/web/app/app.vue"),
        "<template><NuxtLayout><NuxtPage /></NuxtLayout></template>",
        script_analysis("", &["NuxtLayout", "NuxtPage"]),
    );
    analyzer.rebuild_component_edges();

    let result = analyzer.analyze();
    assert!(result.provide_inject_matches.iter().any(|matched| {
        matched.provider == layout && matched.consumer == button && matched.path.len() == 3
    }));
    assert!(result.diagnostics.is_empty(), "{:#?}", result.diagnostics);
}

#[test]
fn nuxt_layout_keeps_page_ancestry_when_it_renders_its_own_component() {
    let mut analyzer = CrossFileAnalyzer::new(
        CrossFileOptions::minimal()
            .with_provide_inject(true)
            .with_reactivity_tracking(true),
    );
    let header = analyzer.add_file_with_analysis(
        Path::new("app/components/AppHeader.vue"),
        "",
        script_analysis(
            "import { inject } from 'vue'; import { ToastKey } from '../keys'; const toast = inject(ToastKey)",
            &[],
        ),
    );
    let button = analyzer.add_file_with_analysis(
        Path::new("app/components/SaveButton.vue"),
        "",
        script_analysis(
            "import { inject } from 'vue'; import { ToastKey } from '../keys'; const toast = inject(ToastKey)",
            &[],
        ),
    );
    analyzer.add_file_with_analysis(
        Path::new("app/pages/index.vue"),
        "",
        script_analysis(
            "import SaveButton from '../components/SaveButton.vue'",
            &["SaveButton"],
        ),
    );
    let mut layout_analysis = script_analysis(
        "import { provide } from 'vue'; import { ToastKey } from '../keys'; import AppHeader from '../components/AppHeader.vue'; provide(ToastKey, { show: (message: string) => console.log(message) })",
        &["AppHeader"],
    );
    note_component_usage(&mut layout_analysis, "AppHeader", 10);
    let layout = analyzer.add_file_with_analysis(
        Path::new("app/layouts/default.vue"),
        "<template><AppHeader /><main><slot /></main></template>",
        layout_analysis,
    );
    analyzer.add_file_with_analysis(
        Path::new("app/app.vue"),
        "<template><NuxtLayout><NuxtPage /></NuxtLayout></template>",
        script_analysis("", &["NuxtLayout", "NuxtPage"]),
    );
    analyzer.rebuild_component_edges();

    let result = analyzer.analyze();
    for consumer in [header, button] {
        assert!(
            result
                .provide_inject_matches
                .iter()
                .any(|matched| { matched.provider == layout && matched.consumer == consumer })
        );
    }
    assert!(result.diagnostics.is_empty(), "{:#?}", result.diagnostics);
}

#[test]
fn nuxt_render_edges_do_not_cross_package_roots() {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::minimal().with_provide_inject(true));
    analyzer.add_file_with_analysis(
        Path::new("packages/web/app/layouts/default.vue"),
        "<template><slot /></template>",
        script_analysis(
            "import { provide } from 'vue'; provide('theme', 'dark')",
            &[],
        ),
    );
    for (app, page) in [
        (
            "packages/web/app/app.vue",
            "packages/web/app/pages/index.vue",
        ),
        (
            "packages/admin/app/app.vue",
            "packages/admin/app/pages/index.vue",
        ),
    ] {
        analyzer.add_file_with_analysis(
            Path::new(app),
            "<template><NuxtLayout><NuxtPage /></NuxtLayout></template>",
            script_analysis("", &["NuxtLayout", "NuxtPage"]),
        );
        analyzer.add_file_with_analysis(
            Path::new(page),
            "",
            script_analysis(
                "import { inject } from 'vue'; const theme = inject('theme')",
                &[],
            ),
        );
    }
    analyzer.rebuild_component_edges();

    let result = analyzer.analyze();
    assert_eq!(result.provide_inject_matches.len(), 1);
    let admin_page = analyzer
        .registry()
        .get_id(Path::new("packages/admin/app/pages/index.vue"))
        .expect("admin page");
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.primary_file == admin_page
            && matches!(
                diagnostic.kind,
                CrossFileDiagnosticKind::UnmatchedInject { .. }
            )
    }));
}

#[test]
fn symbol_injection_diagnostic_uses_identifier_syntax() {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::minimal().with_provide_inject(true));
    analyzer.add_file_with_analysis(
        Path::new("app/components/Child.vue"),
        "",
        script_analysis(
            "import { inject } from 'vue'; import { ToastKey } from '../composables/toast'; const toast = inject(ToastKey)",
            &[],
        ),
    );
    let result = analyzer.analyze();
    assert!(result.diagnostics.iter().any(|diagnostic| {
        matches!(
            diagnostic.kind,
            CrossFileDiagnosticKind::UnmatchedInject { .. }
        ) && diagnostic.message.contains("inject(ToastKey)")
    }));
}
