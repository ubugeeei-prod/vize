//! Framework render edges for Nuxt `app.vue`, layouts and pages.

use super::CrossFileAnalyzer;
use crate::NuxtRenderDirs;
use crate::graph::DependencyEdge;
use crate::registry::{FileId, ModuleEntry};
use std::path::Path;

fn uses(entry: &ModuleEntry, names: [&str; 2]) -> bool {
    vize_croquis::facts::used_component_name_list(&entry.analysis)
        .iter()
        .any(|name| names.contains(&name.as_str()))
}

fn is_app_vue(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some("app.vue" | "App.vue")
    )
}

impl CrossFileAnalyzer {
    /// Nuxt renders pages through `app.vue`, its default layout and the
    /// `<NuxtPage>` of parent pages. These components have no static import
    /// edge, so the runtime ancestry is recorded after all SFCs are registered.
    pub(super) fn add_nuxt_render_edges(&mut self) {
        let profiler = vize_l0::profiler::global_profiler();
        let _phase = profiler.global_span("croquis_cf.dependencies.nuxt_render");
        profiler.record_counter("croquis_cf.dependencies.nuxt_render.calls", 1);
        // Configured dirs resolve against the project root like registered paths.
        let resolve = |path: &Path| match self.registry.project_root() {
            Some(root) if path.is_relative() => root.join(path),
            _ => path.to_path_buf(),
        };
        let dirs = self.nuxt_dirs.as_ref().map(|dirs| NuxtRenderDirs {
            src: resolve(&dirs.src),
            pages: resolve(&dirs.pages),
            layouts: resolve(&dirs.layouts),
            pages_enabled: dirs.pages_enabled,
        });
        let mut edges = Vec::new();
        let mut configured_app = false;
        for app in self.registry.vue_components() {
            if !is_app_vue(&app.path) {
                continue;
            }
            let Some(root) = app.path.parent() else {
                continue;
            };
            let configured = dirs.as_ref().filter(|dirs| dirs.src == root);
            configured_app |= configured.is_some();
            if !uses(app, ["NuxtPage", "nuxt-page"]) {
                continue;
            }
            let layout = uses(app, ["NuxtLayout", "nuxt-layout"]);
            match configured {
                Some(dirs) if dirs.pages_enabled => {
                    self.page_edges(Some(app.id), layout, &dirs.pages, &dirs.layouts, &mut edges)
                }
                // Without filesystem routing, `<NuxtPage>` renders no `pages` files.
                Some(_) => {}
                None => self.page_edges(
                    Some(app.id),
                    layout,
                    &root.join("pages"),
                    &root.join("layouts"),
                    &mut edges,
                ),
            }
        }
        if let Some(dirs) = &dirs
            && dirs.pages_enabled
            && !configured_app
        {
            // Without `app.vue`, Nuxt's own app renders `<NuxtLayout><NuxtPage /></NuxtLayout>`.
            self.page_edges(None, true, &dirs.pages, &dirs.layouts, &mut edges);
        }
        profiler.record_counter(
            "croquis_cf.dependencies.nuxt_render.edge_attempts",
            edges.len() as u64,
        );
        for (parent, child) in edges {
            self.graph
                .add_edge(parent, child, DependencyEdge::ComponentUsage);
        }
    }

    fn page_edges(
        &self,
        app: Option<FileId>,
        has_layout: bool,
        pages: &Path,
        layouts: &Path,
        edges: &mut Vec<(FileId, FileId)>,
    ) {
        let layout = has_layout
            .then(|| self.registry.get_by_path(layouts.join("default.vue")))
            .flatten()
            .map(|entry| entry.id);
        if let (Some(app), Some(layout)) = (app, layout) {
            edges.push((app, layout));
        }
        for page in self.registry.vue_components() {
            if !page.path.starts_with(pages) {
                continue;
            }
            if let Some(parent) = layout.or(app) {
                edges.push((parent, page.id));
            }
            if !uses(page, ["NuxtPage", "nuxt-page"]) {
                continue;
            }
            // A parent page's `<NuxtPage>` renders the routes in its same-named directory.
            let children = page.path.with_extension("");
            for child in self.registry.vue_components() {
                if child.path.starts_with(&children) {
                    edges.push((page.id, child.id));
                }
            }
        }
    }
}
