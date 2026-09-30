use super::*;
use crate::diagnostics::CrossFileDiagnosticKind;

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
    for package in ["web", "admin"] {
        analyzer.add_file_with_analysis(
            Path::new(&format!("packages/{package}/app/app.vue")),
            "",
            script_analysis("", &["NuxtLayout", "NuxtPage"]),
        );
        analyzer.add_file_with_analysis(
            Path::new(&format!("packages/{package}/app/pages/index.vue")),
            "",
            script_analysis("import { inject } from 'vue'; inject('theme')", &[]),
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
