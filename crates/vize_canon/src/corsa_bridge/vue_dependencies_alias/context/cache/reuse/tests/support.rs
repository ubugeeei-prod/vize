use std::path::{Path, PathBuf};

use vize_carton::{FxHashMap, String};

use crate::corsa_bridge::EditorMirrorSession;
use crate::corsa_bridge::vue_dependencies_alias::context::cache::fingerprint::{
    ContextFingerprint, SourceGuard,
};
use crate::corsa_bridge::vue_dependencies_alias::{AliasContext, PreparedAliasContext};
use crate::corsa_bridge::vue_document::{CorsaProjectEnvironment, CorsaVueVirtualDocumentOptions};

pub(super) const CHILD: &str = "<!-- 🦀 -->\r\n<script setup lang='ts'>type Props = { label?: string }; withDefaults(defineProps<Props>(), { label: 'before' });</script>\r\n<template>{{ label }}</template>\r\n";
pub(super) const HOST: &str = "<script setup lang='ts'>import Child from './Child.vue'; const count = 1;</script><template><Child :label='String(count)' /></template>";

pub(super) struct Fixture {
    _storage: tempfile::TempDir,
    pub(super) root: PathBuf,
    pub(super) host: PathBuf,
    pub(super) session: EditorMirrorSession,
    pub(super) routes: crate::PackageRouteResolver,
    pub(super) options: CorsaVueVirtualDocumentOptions,
    pub(super) virtual_options: crate::virtual_ts::VirtualTsOptions,
}

impl Fixture {
    pub(super) fn new(host: &str) -> Self {
        let storage = tempfile::tempdir().unwrap();
        let root = storage.path().canonicalize().unwrap();
        let fixture = Self {
            host: root.join("src/Host.vue"),
            root,
            _storage: storage,
            session: EditorMirrorSession::new(),
            routes: Default::default(),
            options: Default::default(),
            virtual_options: Default::default(),
        };
        fixture.write("tsconfig.json", "{\"compilerOptions\":{\"strict\":true,\"baseUrl\":\".\",\"paths\":{\"@/*\":[\"src/*\"]}},\"include\":[\"src/**/*\"]}\n");
        fixture.write("src/Host.vue", host);
        fixture.write("src/Child.vue", CHILD);
        fixture
    }

    pub(super) fn write(&self, relative: &str, source: &str) -> PathBuf {
        let path = self.root.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, source).unwrap();
        path
    }

    pub(super) fn environment(&self) -> CorsaProjectEnvironment<'_> {
        CorsaProjectEnvironment {
            virtual_ts_options: &self.virtual_options,
            package_routes: &self.routes,
            project_root: Some(&self.root),
            tsconfig_path: None,
            editor_session: &self.session,
        }
    }

    pub(super) fn open(
        &self,
        source: &str,
        overlays: &FxHashMap<PathBuf, &str>,
    ) -> PreparedAliasContext {
        AliasContext::for_host_cached(
            &self.host,
            source,
            overlays,
            self.options,
            self.environment(),
        )
        .unwrap()
    }

    pub(super) fn project(
        &self,
        source: &str,
        overlays: &FxHashMap<PathBuf, &str>,
    ) -> crate::corsa_bridge::vue_document::CorsaVueVirtualProject {
        let buffers = overlays
            .iter()
            .map(|(path, text)| (path.clone(), *text))
            .collect::<Vec<_>>();
        crate::corsa_bridge::vue_document::build_vue_virtual_project_with_overlays_and_options_and_package_routes(
            &self.host, source, self.options, &buffers, self.environment(),
        ).unwrap()
    }

    pub(super) fn guard(
        &self,
        source: &str,
        overlays: &FxHashMap<PathBuf, &str>,
    ) -> Result<Vec<PathBuf>, SourceGuard> {
        let current = ContextFingerprint::capture(
            &self.host,
            source,
            overlays,
            self.options,
            &self.virtual_options,
            Some(&self.root),
            None,
        );
        let cache = self.session.cache();
        let old = &cache.slots[&self.host];
        old.fingerprint
            .source_changes(&current, &old.context, overlays)
    }

    pub(super) fn assert_cold_equal(
        &self,
        context: &PreparedAliasContext,
        source: &str,
        overlays: &FxHashMap<PathBuf, &str>,
    ) {
        let session = EditorMirrorSession::new();
        let cold = AliasContext::for_host_cached(
            &self.host,
            source,
            overlays,
            self.options,
            CorsaProjectEnvironment {
                editor_session: &session,
                ..self.environment()
            },
        )
        .unwrap();
        let left = context
            .mirror
            .as_ref()
            .unwrap()
            .virtual_root()
            .as_os_str()
            .len();
        let right = cold
            .mirror
            .as_ref()
            .unwrap()
            .virtual_root()
            .as_os_str()
            .len();
        assert_eq!(
            left, right,
            "only the equally sized private namespace is normalized"
        );
        assert_eq!(facts(context, &self.host), facts(&cold, &self.host));
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct DocumentFacts {
    path: std::string::String,
    source_path: PathBuf,
    source: String,
    code: std::string::String,
    mapping: crate::virtual_ts::ProjectionMapping,
    import_map: crate::batch::ImportSourceMap,
    kind: crate::corsa_bridge::CorsaMaterializedMappingKind,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct WholeFacts {
    documents: Vec<DocumentFacts>,
    catalog: Vec<DocumentFacts>,
    files: Vec<(std::string::String, std::string::String)>,
    links: Vec<(
        std::string::String,
        std::string::String,
        std::string::String,
    )>,
    query_paths: Vec<std::string::String>,
    diagnostics: std::string::String,
}

pub(super) fn facts(context: &PreparedAliasContext, host: &Path) -> WholeFacts {
    let mirror = context.mirror.as_ref().unwrap();
    let namespace = mirror.virtual_root().to_str().unwrap();
    let normalize = |value: &str| value.replace(namespace, "$EDITOR_PROJECT");
    let documents = immutable_documents(context);
    let catalog = context
        .materialized_sources()
        .into_iter()
        .map(|document| {
            let leaf = context
                .source_catalog
                .get(&document.materialized_path)
                .unwrap();
            document_fact((*leaf).clone(), namespace)
        })
        .collect();
    let mut files = mirror
        .expected_materialized_files()
        .into_iter()
        .map(|path| {
            let contents = std::fs::read_to_string(&path).unwrap();
            (normalize(path.to_str().unwrap()), normalize(&contents))
        })
        .collect::<Vec<_>>();
    let mut links = mirror
        .desired_package_links()
        .into_iter()
        .map(|(path, target)| {
            let raw = std::fs::read_link(&path).unwrap();
            assert_eq!(path.canonicalize().unwrap(), target.canonicalize().unwrap());
            (
                normalize(path.to_str().unwrap()),
                normalize(raw.to_str().unwrap()),
                normalize(target.to_str().unwrap()),
            )
        })
        .collect::<Vec<_>>();
    files.sort();
    links.sort();
    WholeFacts {
        documents,
        catalog,
        files,
        links,
        query_paths: mirror
            .editor_query_paths(host)
            .iter()
            .map(|path| normalize(path.to_str().unwrap()))
            .collect(),
        diagnostics: format!("{:?}", mirror.diagnostics()),
    }
}

pub(super) fn immutable_documents(context: &AliasContext) -> Vec<DocumentFacts> {
    let mirror = context.mirror.as_ref().unwrap();
    let namespace = mirror.virtual_root().to_str().unwrap();
    context
        .materialized_sources()
        .into_iter()
        .map(|document| document_fact(document, namespace))
        .collect()
}

fn document_fact(
    document: crate::corsa_bridge::CorsaMaterializedSource,
    namespace: &str,
) -> DocumentFacts {
    let normalize = |value: &str| value.replace(namespace, "$EDITOR_PROJECT");
    DocumentFacts {
        path: normalize(document.materialized_path.to_str().unwrap()),
        source_path: document.source_path,
        source: document.source,
        code: normalize(&document.code),
        mapping: document.mapping,
        import_map: document.import_source_map,
        kind: document.mapping_kind,
    }
}

pub(super) fn same_mtime_write(path: &Path, source: &str) {
    let before = std::fs::metadata(path).unwrap();
    assert_eq!(before.len(), source.len() as u64);
    let modified = before.modified().unwrap();
    std::fs::write(path, source).unwrap();
    std::fs::File::options()
        .write(true)
        .open(path)
        .unwrap()
        .set_times(std::fs::FileTimes::new().set_modified(modified))
        .unwrap();
    assert_eq!(
        std::fs::metadata(path).unwrap().modified().unwrap(),
        modified
    );
}
