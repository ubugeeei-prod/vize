use super::*;
use crate::NuxtRenderDirs;
use crate::diagnostics::CrossFileDiagnosticKind;
use std::path::PathBuf;

fn custom_dirs() -> NuxtRenderDirs {
    NuxtRenderDirs {
        src: PathBuf::from("src"),
        pages: PathBuf::from("src/nuxt/pages"),
        layouts: PathBuf::from("src/nuxt/layouts"),
        builtin_app: true,
    }
}

fn add_inject_button(analyzer: &mut CrossFileAnalyzer) -> crate::FileId {
    analyzer.add_file_with_analysis(
        Path::new("src/features/NoticeButton.vue"),
        "",
        script_analysis(
            "import { inject } from 'vue'; import { NoticeKey } from './keys'; const notice = inject(NoticeKey)!",
            &[],
        ),
    )
}

fn provide(path: &str, used: &[&str]) -> vize_croquis::Croquis {
    script_analysis(
        &vize_carton::cstr!(
            "import {{ provide }} from 'vue'; import {{ NoticeKey }} from '{path}'; provide(NoticeKey, {{ show: (text: string) => console.warn(text) }})"
        ),
        used,
    )
}

fn analyzer(dirs: Option<NuxtRenderDirs>) -> CrossFileAnalyzer {
    // Registered paths become absolute under a root; relative dirs follow them.
    let mut analyzer = CrossFileAnalyzer::with_project_root(
        CrossFileOptions::minimal().with_provide_inject(true),
        "/repo",
    );
    analyzer.set_nuxt_render_dirs(dirs);
    analyzer
}

fn unmatched(result: &crate::CrossFileResult) -> usize {
    result
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(
                diagnostic.kind,
                CrossFileDiagnosticKind::UnmatchedInject { .. }
            )
        })
        .count()
}

/// #10776: `dir.layouts` / `dir.pages` and Nuxt's built-in app (no `app.vue` in `srcDir`).
fn add_custom_dir_project(analyzer: &mut CrossFileAnalyzer) -> (crate::FileId, crate::FileId) {
    let button = add_inject_button(analyzer);
    analyzer.add_file_with_analysis(
        Path::new("src/nuxt/pages/index.vue"),
        "",
        script_analysis(
            "import NoticeButton from '../../features/NoticeButton.vue'",
            &["NoticeButton"],
        ),
    );
    let layout = analyzer.add_file_with_analysis(
        Path::new("src/nuxt/layouts/default.vue"),
        "<template><main><slot /></main></template>",
        provide("../../features/keys", &[]),
    );
    // Nuxt resolves `app.vue` from `srcDir`, never from `dir.app`.
    analyzer.add_file_with_analysis(
        Path::new("src/nuxt/app/app.vue"),
        "<template><NuxtLayout><NuxtPage /></NuxtLayout></template>",
        script_analysis("", &["NuxtLayout", "NuxtPage"]),
    );
    (layout, button)
}

#[test]
fn configured_layout_and_page_dirs_keep_layout_ancestry() {
    let mut analyzer = analyzer(Some(custom_dirs()));
    let (layout, button) = add_custom_dir_project(&mut analyzer);
    analyzer.rebuild_component_edges();

    let result = analyzer.analyze();
    assert!(
        result
            .provide_inject_matches
            .iter()
            .any(|matched| matched.provider == layout && matched.consumer == button)
    );
    assert!(result.diagnostics.is_empty(), "{:#?}", result.diagnostics);

    let mut inferred = self::analyzer(None);
    add_custom_dir_project(&mut inferred);
    inferred.rebuild_component_edges();
    assert_eq!(unmatched(&inferred.analyze()), 1);
}

#[test]
fn configured_app_vue_in_src_dir_renders_configured_dirs() {
    let mut analyzer = analyzer(Some(NuxtRenderDirs {
        builtin_app: false,
        ..custom_dirs()
    }));
    let button = add_inject_button(&mut analyzer);
    analyzer.add_file_with_analysis(
        Path::new("src/nuxt/pages/index.vue"),
        "",
        script_analysis(
            "import NoticeButton from '../../features/NoticeButton.vue'",
            &["NoticeButton"],
        ),
    );
    let app = analyzer.add_file_with_analysis(
        Path::new("src/app.vue"),
        "<template><NuxtPage /></template>",
        provide("./features/keys", &["NuxtPage"]),
    );
    analyzer.rebuild_component_edges();

    let result = analyzer.analyze();
    assert!(
        result
            .provide_inject_matches
            .iter()
            .any(|matched| matched.provider == app && matched.consumer == button)
    );
    assert_eq!(unmatched(&result), 0);
}

#[test]
fn parent_page_nuxt_page_provides_to_nested_routes_only() {
    let mut analyzer = analyzer(Some(custom_dirs()));
    let parent = analyzer.add_file_with_analysis(
        Path::new("src/nuxt/pages/orders.vue"),
        "<template><NuxtPage /></template>",
        provide("../../features/keys", &["NuxtPage"]),
    );
    let child = analyzer.add_file_with_analysis(
        Path::new("src/nuxt/pages/orders/[id].vue"),
        "",
        script_analysis(
            "import { inject } from 'vue'; import { NoticeKey } from '../../../features/keys'; const notice = inject(NoticeKey)!",
            &[],
        ),
    );
    analyzer.add_file_with_analysis(
        Path::new("src/nuxt/pages/orders-archive.vue"),
        "",
        script_analysis(
            "import { inject } from 'vue'; import { NoticeKey } from '../../features/keys'; const notice = inject(NoticeKey)!",
            &[],
        ),
    );
    analyzer.rebuild_component_edges();

    let result = analyzer.analyze();
    assert!(
        result
            .provide_inject_matches
            .iter()
            .any(|matched| matched.provider == parent && matched.consumer == child)
    );
    // A sibling route with a shared name prefix is not nested.
    assert_eq!(unmatched(&result), 1);
}
