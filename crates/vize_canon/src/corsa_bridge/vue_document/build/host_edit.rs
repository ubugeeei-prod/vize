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
    let host = Arc::make_mut(&mut surface.host);
    replacements.insert(host.request_uri.clone(), code.clone());
    replacements.insert(path_to_file_uri(materialized_path), code.clone());
    if let Some(fresh) = fresh {
        let uri = path_to_file_uri(&fresh.materialized_path);
        if let Some(current) = host
            .materialized_sources
            .iter_mut()
            .find(|source| source.materialized_path == fresh.materialized_path)
        {
            *current = fresh;
            replacements.insert(uri, current.code.clone());
        }
    }
    host.request_uri = path_to_file_uri(materialized_path);
    host.code = generated.code;
    host.pre_rewrite_code = generated.pre_rewrite_code;
    host.mapping = generated.mapping;
    host.import_source_map = generated.import_source_map;
    host.source_type = generated.source_type;
    host.virtual_suffix = generated.virtual_suffix;
    replace_document_texts(&mut surface.documents, &replacements);
}

/// Dissociate weak batch receipts without copying a uniquely owned dependency
/// tail. A live strong payload still gets its own complete frozen allocation.
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
    apply(Arc::make_mut(documents));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weak_batch_receipts_do_not_copy_a_uniquely_owned_dependency_tail() {
        let dependency: String = "export const child = '雪😀';\r\n".repeat(4096).into();
        let expected = dependency.clone();
        let mut documents = Arc::new(vec![
            ("host".into(), "before".into()),
            ("child".into(), dependency),
        ]);
        let original_tail = documents[1].1.as_bytes().as_ptr();
        let receipt = Arc::downgrade(&documents);
        replace_document_texts(
            &mut documents,
            &FxHashMap::from_iter([("host".into(), "after".into())]),
        );
        assert!(receipt.upgrade().is_none());
        assert_eq!(documents[1].1.as_bytes().as_ptr(), original_tail);
        assert_eq!(
            documents.as_ref(),
            &vec![("host".into(), "after".into()), ("child".into(), expected)]
        );
    }
}
