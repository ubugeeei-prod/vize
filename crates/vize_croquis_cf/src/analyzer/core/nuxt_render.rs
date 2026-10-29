//! Framework render edges for Nuxt `app.vue`, layouts and pages.

use super::CrossFileAnalyzer;
use crate::NuxtRenderDirs;
use crate::graph::DependencyEdge;
use crate::registry::{FileId, ModuleEntry};
use std::path::Path;
use vize_croquis::croquis::NuxtPageLayout;

#[derive(Clone, Copy)]
struct NuxtRenderEdge {
    parent: FileId,
    child: FileId,
    outlet: Option<FileId>,
}

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
            static_render_config: dirs.static_render_config,
        });
        self.nuxt_page_outlets.clear();
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
        for edge in edges {
            self.graph
                .add_edge(edge.parent, edge.child, DependencyEdge::ComponentUsage);
            if let Some(outlet) = edge.outlet {
                self.nuxt_page_outlets
                    .insert((edge.parent, edge.child), outlet);
            }
        }
    }

    fn page_edges(
        &self,
        app: Option<FileId>,
        has_layout: bool,
        pages: &Path,
        layouts: &Path,
        edges: &mut Vec<NuxtRenderEdge>,
    ) {
        let default_layout = has_layout
            .then(|| self.registry.get_by_path(layouts.join("default.vue")))
            .flatten()
            .map(|entry| entry.id);
        if let (Some(app), Some(layout)) = (app, default_layout) {
            edges.push(NuxtRenderEdge {
                parent: app,
                child: layout,
                outlet: None,
            });
        }
        for page in self.registry.vue_components() {
            if !page.path.starts_with(pages) {
                continue;
            }
            if let Some(parent_page) = self.nearest_parent_page(page, pages) {
                // A nested route is rendered only by its closest route parent's
                // outlet. It is never also a direct child of the layout/app or
                // of every more distant route ancestor.
                if uses(parent_page, ["NuxtPage", "nuxt-page"]) {
                    edges.push(NuxtRenderEdge {
                        parent: parent_page.id,
                        child: page.id,
                        outlet: Some(parent_page.id),
                    });
                }
                continue;
            }
            let layout = match &page.analysis.nuxt_page_layout {
                Some(NuxtPageLayout::Named(name)) if has_layout => {
                    let filename = vize_carton::cstr!("{name}.vue");
                    self.registry
                        .get_by_path(layouts.join(filename.as_str()))
                        .map(|entry| entry.id)
                }
                Some(NuxtPageLayout::Disabled) => None,
                _ => default_layout,
            };
            if let (Some(app), Some(layout)) = (app, layout) {
                // The app remains above whichever layout the page selected.
                // This is distinct from layout -> page ownership below.
                edges.push(NuxtRenderEdge {
                    parent: app,
                    child: layout,
                    outlet: None,
                });
            }
            if let Some(parent) = layout.or(app) {
                edges.push(NuxtRenderEdge {
                    parent,
                    child: page.id,
                    outlet: app,
                });
            }
        }
    }

    fn nearest_parent_page<'a>(
        &'a self,
        page: &ModuleEntry,
        pages: &Path,
    ) -> Option<&'a ModuleEntry> {
        let mut directory = page.path.parent()?;
        while directory != pages {
            if !directory.starts_with(pages) {
                return None;
            }
            let mut filename = directory.file_name()?.to_os_string();
            filename.push(".vue");
            let candidate = directory.with_file_name(filename);
            if let Some(parent) = self.registry.get_by_path(candidate) {
                return Some(parent);
            }
            directory = directory.parent()?;
        }
        None
    }
}
