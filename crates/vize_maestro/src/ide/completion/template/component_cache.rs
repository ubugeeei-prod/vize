//! Version-aware metadata cache for imported components.

use std::{path::Path, sync::Arc};

use crate::ide::IdeContext;

use super::component_interface::component_metadata_from_interface;
use super::component_meta::ComponentMetadata;

#[derive(Clone)]
pub(crate) struct CachedComponentMetadata {
    len: u64,
    modified: Option<std::time::SystemTime>,
    version: Option<i32>,
    hash: Option<u64>,
    configuration: u64,
    source_revision: u64,
    metadata: Arc<ComponentMetadata>,
}

pub(super) fn cached_component_metadata(
    ctx: &IdeContext,
    resolved: &Path,
) -> Option<Arc<ComponentMetadata>> {
    let cache = ctx.state.component_metadata_cache();
    let options_api = ctx.state.options_api_enabled();
    let legacy_vue2 = ctx.state.legacy_vue2_enabled();
    let revision = ctx.state.documents.revision();
    let configuration = serde_json::to_string(&(
        ctx.state.get_type_checker_config(),
        options_api,
        legacy_vue2,
    ))
    .ok()?;
    let configuration_hash = vize_l0::hash::hash_str(&configuration);
    let open = open_component(ctx, resolved);
    let (content, len, modified, version, hash) = if let Some((content, len, version, hash)) = open
    {
        if let Some(entry) = cache.get(resolved)
            && entry.configuration == configuration_hash
            && entry.source_revision == revision
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
            && entry.configuration == configuration_hash
            && entry.source_revision == revision
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

    let (source_revision, sources) = ctx.state.component_type_sources()?;
    if source_revision != revision {
        return None;
    }
    let summary =
        ctx.state
            .component_interface(resolved, &content, &configuration, |descriptor| {
                super::component_interface::export_component_interface_with_sources(
                    descriptor,
                    &resolved.to_string_lossy(),
                    options_api,
                    legacy_vue2,
                    &sources,
                )
            })?;
    let projected = component_metadata_from_interface(&summary)?;
    let metadata = cache
        .get(resolved)
        .filter(|entry| *entry.metadata == projected)
        .map_or_else(|| Arc::new(projected), |entry| entry.metadata.clone());
    cache.insert(
        resolved.to_path_buf(),
        CachedComponentMetadata {
            len,
            modified,
            version,
            hash,
            configuration: configuration_hash,
            source_revision: revision,
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
    let hash = vize_l0::hash::hash_str(&content);
    (
        content,
        document.content.len_bytes() as u64,
        document.version,
        hash,
    )
}

#[cfg(test)]
mod overlay_tests;

#[cfg(test)]
mod tests {
    #![expect(
        clippy::disallowed_methods,
        reason = "test fixtures build owned source text with to_string"
    )]

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

    #[test]
    fn body_edits_refresh_production_alpha_and_reuse_the_metadata_snapshot() {
        let dir = tempfile::tempdir().unwrap();
        let component = dir.path().join("Widget.vue");
        let first_source = "<script setup lang='ts'>defineProps<{ title: string }>(); const count = 1</script><template><p>{{ count }}</p><slot name='footer'/></template>";
        std::fs::write(&component, first_source).unwrap();
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
        assert_eq!(state.resident.take_interface_stats().exports, 1);
        let component_uri = Url::from_file_path(&component).unwrap();
        state.documents.open(
            component_uri.clone(),
            first_source.replace("count = 1", "count = 42"),
            2,
            "vue".to_string(),
        );
        let body = cached_component_metadata(&ctx, &component).unwrap();
        assert!(Arc::ptr_eq(&first, &body));
        let body_stats = state.resident.take_interface_stats();
        assert_eq!(
            (
                body_stats.exports,
                body_stats.consumers,
                body_stats.consumer_reuses
            ),
            (1, 0, 1)
        );
        assert!(Arc::ptr_eq(
            &body,
            &cached_component_metadata(&ctx, &component).unwrap()
        ));
        assert_eq!(state.resident.take_interface_stats().exports, 0);
        state.documents.open(
            component_uri,
            first_source.replace("title: string", "title: number"),
            3,
            "vue".to_string(),
        );
        let prop = cached_component_metadata(&ctx, &component).unwrap();
        assert!(!Arc::ptr_eq(&body, &prop));
        assert_eq!(
            prop.props.first().unwrap().type_detail.as_deref(),
            Some("number")
        );
        assert_eq!(state.resident.take_interface_stats().exports, 1);
        state.invalidate_component_interfaces();
        let refreshed = cached_component_metadata(&ctx, &component).unwrap();
        assert_eq!(*refreshed, *prop);
        assert_eq!(state.resident.take_interface_stats().exports, 1);
    }
}
