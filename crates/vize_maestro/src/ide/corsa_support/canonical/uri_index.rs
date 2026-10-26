//! Native locations name their canonical document by URI. Each URI form is
//! resolved once, so mapping every location of a project-wide answer stays
//! linear in locations plus documents instead of their product (#10622).

use std::path::PathBuf;

use vize_l0::{FxHashMap, String};

use super::{
    CanonicalDependencyDocument, CanonicalMaterializedSource, CanonicalVirtualDocument,
    location_matches_uri_cached,
};
use crate::server::ServerState;

pub(crate) enum IndexedDocument<'a> {
    Primary,
    Dependency(&'a CanonicalDependencyDocument),
    Materialized(&'a CanonicalMaterializedSource),
}

pub(crate) struct CanonicalUriIndex<'a> {
    state: &'a ServerState,
    document: &'a CanonicalVirtualDocument,
    exact: FxHashMap<&'a str, usize>,
    paths: FxHashMap<Option<String>, usize>,
    files: FxHashMap<PathBuf, usize>,
}

impl<'a> CanonicalUriIndex<'a> {
    pub(crate) fn new(state: &'a ServerState, document: &'a CanonicalVirtualDocument) -> Self {
        let mut index = Self {
            state,
            document,
            exact: FxHashMap::default(),
            paths: FxHashMap::default(),
            files: FxHashMap::default(),
        };
        let uris = std::iter::once(document.request_uri.as_str())
            .chain(document.dependencies.iter().map(|d| d.request_uri.as_str()))
            .chain(
                document
                    .materialized_sources
                    .iter()
                    .map(|s| s.request_uri.as_str()),
            );
        for (position, uri) in uris.enumerate() {
            index.exact.entry(uri).or_insert(position);
            index
                .paths
                .entry(super::super::virtual_document_path(uri))
                .or_insert(position);
            if let Some(file) = file_path(uri) {
                index
                    .files
                    .entry(state.canonical_editor_path(&file))
                    .or_insert(position);
            }
        }
        index
    }

    pub(crate) fn document(&self) -> &'a CanonicalVirtualDocument {
        self.document
    }

    /// The first document, in primary, dependency, materialized order, that
    /// `location_matches_uri_cached` accepts for `uri`.
    pub(crate) fn get(&self, uri: &str) -> Option<IndexedDocument<'a>> {
        let exact = self.exact.get(uri).copied();
        let path = self
            .paths
            .get(&super::super::virtual_document_path(uri))
            .copied();
        let file = file_path(uri)
            .and_then(|file| self.files.get(&self.state.canonical_editor_path(&file)))
            .copied();
        let position = [exact, path, file].into_iter().flatten().min()?;
        let dependencies = self.document.dependencies.len();
        match position {
            0 => Some(IndexedDocument::Primary),
            position if position <= dependencies => self
                .document
                .dependencies
                .get(position - 1)
                .map(IndexedDocument::Dependency),
            position => self
                .document
                .materialized_sources
                .get(position - 1 - dependencies)
                .map(IndexedDocument::Materialized),
        }
    }
}

/// The same first match by a linear scan, for a single location.
pub(super) fn matching_document<'a>(
    state: &ServerState,
    document: &'a CanonicalVirtualDocument,
    uri: &str,
) -> Option<IndexedDocument<'a>> {
    if location_matches_uri_cached(state, uri, &document.request_uri) {
        return Some(IndexedDocument::Primary);
    }
    if let Some(dependency) = document
        .dependencies
        .iter()
        .find(|dependency| location_matches_uri_cached(state, uri, &dependency.request_uri))
    {
        return Some(IndexedDocument::Dependency(dependency));
    }
    document
        .materialized_sources
        .iter()
        .find(|source| location_matches_uri_cached(state, uri, &source.request_uri))
        .map(IndexedDocument::Materialized)
}

fn file_path(uri: &str) -> Option<PathBuf> {
    tower_lsp::lsp_types::Url::parse(uri)
        .ok()?
        .to_file_path()
        .ok()
}

#[cfg(all(test, unix))]
mod tests {
    use tower_lsp::lsp_types::Url;
    use vize_canon::CorsaMaterializedMappingKind;

    use super::{CanonicalUriIndex, IndexedDocument, matching_document};
    use crate::ide::corsa_support::canonical::{
        CanonicalDependencyDocument, CanonicalMaterializedSource, CanonicalVirtualDocument,
    };
    use crate::ide::diagnostics::VirtualTsResult;

    fn result() -> VirtualTsResult {
        VirtualTsResult {
            code: "export {};\n".to_owned(),
            source_mappings: Vec::new(),
            semantic_links: Vec::new(),
            prop_default_key_links: Vec::new(),
            diagnostic_mappings: Vec::new(),
            import_source_map: vize_canon::ImportSourceMap::empty(),
        }
    }

    fn position(document: &CanonicalVirtualDocument, found: Option<IndexedDocument<'_>>) -> i64 {
        match found {
            None => -1,
            Some(IndexedDocument::Primary) => 0,
            Some(IndexedDocument::Dependency(found)) => document
                .dependencies
                .iter()
                .position(|candidate| std::ptr::eq(candidate, found))
                .map_or(-2, |index| index as i64 + 1),
            Some(IndexedDocument::Materialized(found)) => document
                .materialized_sources
                .iter()
                .position(|candidate| std::ptr::eq(candidate, found))
                .map_or(-2, |index| (index + 1 + document.dependencies.len()) as i64),
        }
    }

    #[test]
    fn index_returns_the_first_document_the_linear_scan_accepts() {
        let root = crate::test_support::tempdir().unwrap();
        let real = root.path().join("real");
        std::fs::create_dir_all(&real).unwrap();
        std::fs::write(real.join("A.vue.ts"), "").unwrap();
        std::os::unix::fs::symlink(&real, root.path().join("link")).unwrap();
        let file = |relative: &str| {
            Url::from_file_path(root.path().join(relative))
                .unwrap()
                .to_string()
        };
        let source = Url::parse("file:///workspace/App.vue").unwrap();
        let dependency = |uri: &str| CanonicalDependencyDocument {
            source_uri: source.clone(),
            source: "".into(),
            request_uri: uri.into(),
            virtual_result: result(),
        };
        let materialized = |uri: &str| CanonicalMaterializedSource {
            source_uri: source.clone(),
            source: "".into(),
            request_uri: uri.into(),
            virtual_result: result(),
            mapping_kind: CorsaMaterializedMappingKind::Generated,
        };
        // Duplicate identities in later candidates must never win.
        let document = CanonicalVirtualDocument {
            source_uri: source.clone(),
            request_uri: file("real/App.vue.ts").into(),
            virtual_result: result(),
            dependencies: vec![
                dependency(&file("link/A.vue.ts")),
                dependency("vize-virtual:///virtual/B.vue.ts"),
                dependency(&file("real/A.vue.ts")),
                dependency("not a uri"),
            ],
            materialized_sources: vec![
                materialized(&file("real/C%20D.ts")),
                materialized("vize-virtual:///virtual/B.vue.ts"),
                materialized("untitled:Scratch.ts"),
            ],
            session_project_roots: Vec::new(),
            source_catalogs: Vec::new(),
        };
        let state = crate::server::ServerState::new();
        let index = CanonicalUriIndex::new(&state, &document);
        for uri in [
            file("real/App.vue.ts"),
            file("link/App.vue.ts"),
            file("real/A.vue.ts"),
            file("link/A.vue.ts"),
            "vize-virtual:///virtual/B.vue.ts".into(),
            "file:///virtual/B.vue.ts".into(),
            file("real/C D.ts"),
            file("real/C%20D.ts"),
            "untitled:Scratch.ts".into(),
            "another bare string".into(),
            "not a uri".into(),
            file("real/Missing.ts"),
        ] {
            assert_eq!(
                position(&document, index.get(&uri)),
                position(&document, matching_document(&state, &document, &uri)),
                "{uri}"
            );
        }
    }
}
