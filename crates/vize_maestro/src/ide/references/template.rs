#[cfg(test)]
mod tests {
    use tower_lsp::lsp_types::Url;

    use crate::ide::{IdeContext, ReferencesService};
    use crate::server::ServerState;

    #[test]
    fn inline_template_references_include_the_tag_column_and_utf16_prefix() {
        let source =
            "<script setup>\nconst shared = 1\n</script>\n<template>💥 {{ shared }}</template>\n";
        let uri = Url::parse("file:///Inline.vue").unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, source);
        let ctx = IdeContext::new(&state, &uri, source.find("shared =").unwrap()).unwrap();

        let references = ReferencesService::references(&ctx, true).unwrap();
        assert_eq!(
            references.len(),
            2,
            "duplicate or missing hits: {references:#?}"
        );
        for location in references {
            let start = crate::ide::position_to_offset(
                source,
                location.range.start.line,
                location.range.start.character,
            )
            .unwrap();
            let end = crate::ide::position_to_offset(
                source,
                location.range.end.line,
                location.range.end.character,
            )
            .unwrap();
            assert_eq!(&source[start..end], "shared");
        }
    }
}
