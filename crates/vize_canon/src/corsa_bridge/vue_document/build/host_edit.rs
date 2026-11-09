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

use vize_carton::FxHashMap;
#[cfg(test)]
use vize_carton::String;

use super::QuerySurface;
use crate::batch::virtual_project::VueDocumentVirtualTs;
use crate::corsa_bridge::vue_document::CorsaMaterializedSource;
#[cfg(test)]
use crate::corsa_bridge::vue_document::CorsaNativeDocumentBatch;
use crate::file_uri::path_to_file_uri;

pub(in crate::corsa_bridge) fn retarget_host(
    surface: &mut QuerySurface,
    materialized_path: &Path,
    generated: VueDocumentVirtualTs,
    fresh: Option<CorsaMaterializedSource>,
) {
    let mut replacements = FxHashMap::default();
    let mut payloads = Vec::new();
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
            *current = Arc::new(fresh);
            replacements.insert(uri, current.code.as_str().into());
            payloads.push(Arc::clone(current.code.native_document()));
        }
    }
    host.request_uri = path_to_file_uri(materialized_path);
    host.code = generated.code;
    host.pre_rewrite_code = generated.pre_rewrite_code;
    host.mapping = generated.mapping;
    host.import_source_map = generated.import_source_map;
    host.source_type = generated.source_type;
    host.virtual_suffix = generated.virtual_suffix;
    surface.documents.replace_texts(&replacements, &payloads);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weak_batch_receipts_do_not_copy_a_uniquely_owned_dependency_tail() {
        let dependency: String = "export const child = '雪😀';\r\n".repeat(4096).into();
        let expected = dependency.clone();
        let mut documents = CorsaNativeDocumentBatch::from(vec![
            ("host".into(), "before".into()),
            ("child".into(), dependency),
        ]);
        let original_tail = documents.to_vec()[1].1.len();
        let tail = documents.iter().nth(1).unwrap().1.as_bytes().as_ptr();
        let receipt = documents.payload_owner_for_test();
        documents.replace_texts(
            &FxHashMap::from_iter([("host".into(), "after".into())]),
            &[],
        );
        assert!(receipt.upgrade().is_none());
        assert_eq!(documents.iter().nth(1).unwrap().1.as_bytes().as_ptr(), tail);
        assert_eq!(documents.iter().nth(1).unwrap().1.len(), original_tail);
        assert_eq!(
            documents.to_vec(),
            vec![("host".into(), "after".into()), ("child".into(), expected)]
        );
    }

    #[test]
    fn a_replaced_shared_leaf_payload_is_shared_not_copied() {
        let mut documents = CorsaNativeDocumentBatch::from(vec![("host".into(), "host".into())]);
        documents.push_shared(Arc::new(("leaf".into(), "before".into())));
        documents.push_shared(Arc::new(("other".into(), "other".into())));
        let other = Arc::clone(&documents.shared_additions_for_test()[1]);
        let fresh = Arc::new((String::from("leaf"), String::from("after")));
        documents.replace_texts(
            &FxHashMap::from_iter([("leaf".into(), "after".into())]),
            std::slice::from_ref(&fresh),
        );
        assert!(Arc::ptr_eq(
            &documents.shared_additions_for_test()[0],
            &fresh
        ));
        assert!(Arc::ptr_eq(
            &documents.shared_additions_for_test()[1],
            &other
        ));
        assert_eq!(documents.iter().nth(1).unwrap().1, "after");
    }
}
