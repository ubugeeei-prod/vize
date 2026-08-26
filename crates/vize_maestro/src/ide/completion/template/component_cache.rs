//! Version-aware metadata cache for imported components.

use std::{path::Path, sync::Arc};

use crate::ide::IdeContext;

use super::component_meta::{ComponentMetadata, extract_component_metadata};

#[derive(Clone)]
pub(crate) struct CachedComponentMetadata {
    len: u64,
    modified: Option<std::time::SystemTime>,
    version: Option<i32>,
    hash: Option<u64>,
    metadata: Arc<ComponentMetadata>,
}

pub(super) fn cached_component_metadata(
    ctx: &IdeContext,
    resolved: &Path,
) -> Option<Arc<ComponentMetadata>> {
    let cache = ctx.state.component_metadata_cache();
    let open = open_component(ctx, resolved);
    let (content, len, modified, version, hash) = if let Some((content, len, version, hash)) = open
    {
        if let Some(entry) = cache.get(resolved)
            && entry.len == len
            && entry.version == Some(version)
            && entry.hash == Some(hash)
        {
            return Some(entry.metadata.clone());
        }
        (content, len, None, Some(version), Some(hash))
    } else {
        let metadata = std::fs::metadata(resolved).ok()?;
        let len = metadata.len();
        let modified = metadata.modified().ok();
        if let Some(entry) = cache.get(resolved)
            && modified.is_some()
            && entry.len == len
            && entry.modified == modified
            && entry.version.is_none()
        {
            return Some(entry.metadata.clone());
        }
        (
            std::fs::read_to_string(resolved).ok()?,
            len,
            modified,
            None,
            None,
        )
    };

    let metadata = Arc::new(extract_component_metadata(
        &content,
        &resolved.to_string_lossy(),
        ctx.state.options_api_enabled(),
        ctx.state.legacy_vue2_enabled(),
    ));
    cache.insert(
        resolved.to_path_buf(),
        CachedComponentMetadata {
            len,
            modified,
            version,
            hash,
            metadata: metadata.clone(),
        },
    );
    Some(metadata)
}

fn open_component(ctx: &IdeContext<'_>, resolved: &Path) -> Option<(String, u64, i32, u64)> {
    if let Ok(uri) = tower_lsp::lsp_types::Url::from_file_path(resolved)
        && let Some(document) = ctx.state.documents.get(&uri)
    {
        return Some(open_stamp(&document));
    }

    ctx.state.documents.iter().find_map(|document| {
        let path = document.key().to_file_path().ok()?;
        (std::fs::canonicalize(path).ok().as_deref() == Some(resolved))
            .then(|| open_stamp(document.value()))
    })
}

fn open_stamp(document: &crate::document::Document) -> (String, u64, i32, u64) {
    let content = document.text();
    let hash = vize_s0::hash::hash_str(&content);
    (
        content,
        document.content.len_bytes() as u64,
        document.version,
        hash,
    )
}

#[cfg(test)]
mod tests {
    #![allow(clippy::disallowed_methods)]

    use super::cached_component_metadata;
    use crate::{ide::IdeContext, server::ServerState};
    use std::sync::Arc;
    use tower_lsp::lsp_types::Url;

    #[test]
    fn disk_metadata_cache_hits_then_invalidates_on_change() {
        let dir = tempfile::tempdir().unwrap();
        let component = dir.path().join("Widget.vue");
        std::fs::write(
            &component,
            "<script setup lang=\"ts\">defineProps<{ a: string }>()</script>",
        )
        .unwrap();
        let state = ServerState::new();
        let uri = Url::parse("file:///host.vue").unwrap();
        state.documents.open(
            uri.clone(),
            "<template />".to_string(),
            1,
            "vue".to_string(),
        );
        let ctx = IdeContext::new(&state, &uri, 0).unwrap();

        let first = cached_component_metadata(&ctx, &component).unwrap();
        let second = cached_component_metadata(&ctx, &component).unwrap();
        assert_eq!(first.props[0].type_detail.as_deref(), Some("string"));
        assert!(first.props[0].required);
        assert!(Arc::ptr_eq(&first, &second));

        std::fs::write(
            &component,
            "<script setup lang=\"ts\">defineProps<{ a: string; bb: number }>()</script>",
        )
        .unwrap();
        let third = cached_component_metadata(&ctx, &component).unwrap();
        assert!(!Arc::ptr_eq(&first, &third));
        assert!(third.props.len() > first.props.len());
    }
}
