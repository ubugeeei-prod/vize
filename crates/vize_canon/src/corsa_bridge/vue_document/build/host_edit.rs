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

/// Fork only changed document identities. Live batches keep their exact frozen
/// rows while every unchanged dependency allocation stays shared.
fn replace_document_texts(
    documents: &mut Arc<Vec<Arc<(String, String)>>>,
    replacements: &FxHashMap<String, String>,
) {
    let apply = |entries: &mut Vec<Arc<(String, String)>>| {
        for document in entries {
            if let Some(replacement) = replacements.get(document.0.as_str()) {
                *document = Arc::new((document.0.clone(), replacement.clone()));
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
            Arc::new(("host".into(), "before".into())),
            Arc::new(("child".into(), dependency)),
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
            documents
                .iter()
                .map(|document| document.as_ref().clone())
                .collect::<Vec<_>>(),
            vec![("host".into(), "after".into()), ("child".into(), expected)]
        );
    }

    #[test]
    fn live_batch_forks_only_the_replaced_document() {
        let mut documents = Arc::new(vec![
            Arc::new(("host".into(), "before".into())),
            Arc::new((
                "child".into(),
                String::from("export const child = '雪😀';\r\n").repeat(4096),
            )),
        ]);
        let batch = Arc::clone(&documents);
        let host = Arc::clone(&documents[0]);
        let dependency = Arc::clone(&documents[1]);
        replace_document_texts(
            &mut documents,
            &FxHashMap::from_iter([("host".into(), "after".into())]),
        );
        assert_eq!(batch[0].1, "before");
        assert_eq!(documents[0].1, "after");
        assert!(!Arc::ptr_eq(&documents[0], &host));
        assert!(Arc::ptr_eq(&batch[0], &host));
        assert!(Arc::ptr_eq(&documents[1], &dependency));
        assert!(Arc::ptr_eq(&batch[1], &dependency));
    }
}
