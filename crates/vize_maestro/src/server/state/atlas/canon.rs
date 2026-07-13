//! Persistent Canon virtual-document queries for editor and dependency revisions.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};

use tower_lsp::lsp_types::Url;
use vize_atlas::Shared;
use vize_carton::{FxHashMap, FxHashSet};

use super::ServerState;

pub(crate) struct CanonVueOverlays {
    pub(crate) sources: Vec<(PathBuf, vize_carton::String)>,
    pub(crate) generated: Vec<(PathBuf, Shared<vize_canon::batch::VueDocumentVirtualTs>)>,
}

impl ServerState {
    pub(crate) fn canon_vue_document_for(
        &self,
        uri: &Url,
        content: &str,
        options: vize_canon::CorsaVueVirtualDocumentOptions,
    ) -> Option<Shared<vize_canon::batch::VueDocumentVirtualTs>> {
        let source = self.ensure_artifact_source(uri, content)?;
        let options = vize_canon::batch::VueDocumentVirtualTsOptions {
            options_api: options.options_api,
            legacy_vue2: options.legacy_vue2,
        };
        let mut compilation = self.artifact_compilation.write();
        if compilation.source_input::<vize_canon::batch::CanonVueDocumentSettingsInput>(source)
            != Some(&options)
            && let Err(error) = vize_canon::batch::install_canon_vue_document_options(
                &mut compilation,
                source,
                options,
            )
        {
            tracing::warn!(%uri, %error, "failed to configure Atlas Canon document request");
            return None;
        }
        match compilation.query::<vize_canon::batch::CanonVueDocumentProduct>(source) {
            Ok(outcome) => Some(outcome.shared()),
            Err(error) => {
                tracing::warn!(%uri, %error, "Atlas Canon document query failed");
                None
            }
        }
    }

    pub(crate) fn canon_vue_overlays(
        &self,
        host: &Url,
        host_generated: &Shared<vize_canon::batch::VueDocumentVirtualTs>,
        options: vize_canon::CorsaVueVirtualDocumentOptions,
    ) -> CanonVueOverlays {
        let documents = self
            .documents
            .iter()
            .map(|document| (document.key().clone(), document.value().text()))
            .collect::<Vec<_>>();
        let mut sources = Vec::with_capacity(documents.len());
        let mut open = FxHashMap::default();
        for (uri, content) in documents {
            if &uri == host {
                continue;
            }
            let Ok(path) = uri.to_file_path() else {
                continue;
            };
            sources.push((path.clone(), content.clone().into()));
            open.insert(normalized(&path), (uri, content));
        }
        let mut generated = Vec::new();
        let mut visited_vue = FxHashSet::default();
        let mut visited_scripts = FxHashSet::default();
        let host_path = host.to_file_path().unwrap_or_else(|_| host.path().into());
        visited_vue.insert(normalized(&host_path));
        let mut queue = VecDeque::from([CanonDependencyScan::Vue(
            host_path,
            Shared::clone(host_generated),
        )]);
        while let Some(scan) = queue.pop_front() {
            let dependencies = scan.dependencies();
            for dependency in dependencies.vue {
                let dependency = normalized(&dependency);
                if !visited_vue.insert(dependency.clone()) || is_art(&dependency) {
                    continue;
                }
                let Some((uri, content)) = dependency_source(&open, &dependency) else {
                    continue;
                };
                let Some(dependency_document) =
                    self.canon_vue_document_for(&uri, &content, options)
                else {
                    continue;
                };
                generated.push((dependency.clone(), Shared::clone(&dependency_document)));
                queue.push_back(CanonDependencyScan::Vue(dependency, dependency_document));
            }
            for dependency in dependencies.scripts {
                let path = normalized(&dependency.path);
                if !visited_scripts.insert(path.clone()) {
                    continue;
                }
                let Some(content) = dependency_content(&open, &path) else {
                    continue;
                };
                queue.push_back(CanonDependencyScan::Script(
                    path,
                    content,
                    dependency.source_type,
                ));
            }
        }
        CanonVueOverlays { sources, generated }
    }
}

fn normalized(path: &Path) -> PathBuf {
    vize_carton::path::canonicalize_non_verbatim(path)
}

fn is_art(path: &Path) -> bool {
    path.to_string_lossy().ends_with(".art.vue")
}

enum CanonDependencyScan {
    Vue(PathBuf, Shared<vize_canon::batch::VueDocumentVirtualTs>),
    Script(PathBuf, String, oxc_span::SourceType),
}

impl CanonDependencyScan {
    fn dependencies(&self) -> vize_canon::CorsaRelativeDependencies {
        match self {
            Self::Vue(path, document) => vize_canon::collect_corsa_relative_dependencies(
                path,
                &document.pre_rewrite_code,
                document.source_type,
            ),
            Self::Script(path, content, source_type) => {
                vize_canon::collect_corsa_relative_dependencies(path, content, *source_type)
            }
        }
    }
}

fn dependency_source(
    open: &FxHashMap<PathBuf, (Url, String)>,
    path: &Path,
) -> Option<(Url, String)> {
    if let Some(source) = open.get(path) {
        return Some(source.clone());
    }
    Some((
        Url::from_file_path(path).ok()?,
        std::fs::read_to_string(path).ok()?,
    ))
}

fn dependency_content(open: &FxHashMap<PathBuf, (Url, String)>, path: &Path) -> Option<String> {
    open.get(path)
        .map(|(_, content)| content.clone())
        .or_else(|| std::fs::read_to_string(path).ok())
}
