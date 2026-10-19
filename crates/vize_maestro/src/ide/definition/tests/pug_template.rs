use std::fs;

use tower_lsp::lsp_types::Url;

use super::super::DefinitionService;
use super::scalar_location;
use crate::ide::{HoverService, IdeContext};
use crate::server::ServerState;

#[test]
fn pug_directive_values_resolve_like_html_attributes() {
    for (cursor, binding) in [
        ("inc\"", "inc() {}"),
        ("count\")", "count = 1"),
        ("ok\"", "ok = true"),
        ("count }}", "count = 1"),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Counter.vue");
        let source = "<script setup lang=\"ts\">\nconst count = 1\nconst ok = true\nfunction inc() {}\n</script>\n<template lang=\"pug\">\ndiv(v-if=\"ok\")\n  button(@click=\"inc\") add\n  p(:title=\"count\") {{ count }}\n</template>\n";
        fs::write(&path, source).unwrap();
        let uri = Url::from_file_path(&path).unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, source);

        let ctx = IdeContext::new(&state, &uri, source.rfind(cursor).unwrap() + 1).unwrap();
        let location = scalar_location(DefinitionService::definition(&ctx).unwrap());
        let (line, character) =
            crate::ide::offset_to_position(source, source.find(binding).unwrap());
        assert_eq!(
            (location.range.start.line, location.range.start.character),
            (line, character),
            "{cursor}"
        );
        assert!(HoverService::hover(&ctx).is_some(), "{cursor}");
    }
}
