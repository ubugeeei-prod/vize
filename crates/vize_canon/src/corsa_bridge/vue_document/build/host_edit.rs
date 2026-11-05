//! Keep a revision's dependency documents when only the host buffer changed.
//!
//! The dependency walk and the mirror-wide source clone do not depend on the
//! host's new bytes. Replacing those bytes in the existing surface leaves every
//! unchanged document allocation in place.

#![expect(
    clippy::disallowed_types,
    reason = "retain the existing immutable query surface ownership while replacing host bytes"
)]

use std::path::Path;
use std::sync::Arc;

use vize_carton::{FxHashMap, String};

use super::QuerySurface;
use crate::batch::virtual_project::VueDocumentVirtualTs;
use crate::corsa_bridge::vue_document::CorsaMaterializedSource;
use crate::file_uri::path_to_file_uri;

pub(in crate::corsa_bridge) fn retarget_host(
    surface: &mut QuerySurface,
    materialized_path: &Path,
    generated: VueDocumentVirtualTs,
    fresh: Option<CorsaMaterializedSource>,
) {
    let mut replacements = FxHashMap::default();
    let code = generated.code.clone();
    replacements.insert(surface.host.request_uri.clone(), code.clone());
    replacements.insert(path_to_file_uri(materialized_path), code.clone());
    if let Some(fresh) = fresh {
        let uri = path_to_file_uri(&fresh.materialized_path);
        if let Some(current) = surface
            .host
            .materialized_sources
            .iter_mut()
            .find(|source| source.materialized_path == fresh.materialized_path)
        {
            *current = fresh;
            replacements.insert(uri, current.code.clone());
        }
    }
    replace_document_texts(&mut surface.documents, &replacements);
    surface.host.request_uri = path_to_file_uri(materialized_path);
    surface.host.code = generated.code;
    surface.host.pre_rewrite_code = generated.pre_rewrite_code;
    surface.host.mapping = generated.mapping;
    surface.host.import_source_map = generated.import_source_map;
    surface.host.source_type = generated.source_type;
    surface.host.virtual_suffix = generated.virtual_suffix;
}

/// `Arc::get_mut` edits the host entries without cloning the dependency tail.
fn replace_document_texts(
    documents: &mut Arc<Vec<(String, String)>>,
    replacements: &FxHashMap<String, String>,
) {
    let apply = |entries: &mut Vec<(String, String)>| {
        for (uri, text) in entries {
            if let Some(replacement) = replacements.get(uri.as_str()) {
                *text = replacement.clone();
            }
        }
    };
    if let Some(entries) = Arc::get_mut(documents) {
        apply(entries);
        return;
    }
    let mut entries = documents.as_ref().clone();
    apply(&mut entries);
    *documents = Arc::new(entries);
}
