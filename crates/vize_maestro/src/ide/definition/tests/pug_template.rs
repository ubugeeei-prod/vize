use std::fs;

use tower_lsp::lsp_types::Url;

use super::super::DefinitionService;
use super::scalar_location;
use crate::ide::{HoverService, IdeContext};
use crate::server::ServerState;

#[test]
fn pug_directive_values_resolve_like_html_attributes() {
    for language in ["pug", "jade"] {
        for (cursor, binding) in [
            ("inc\"", "inc() {}"),
            ("count\")", "count = 1"),
            ("ok\"", "ok = true"),
            ("count }}", "count = 1"),
        ] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("Counter.vue");
            let language_attr = ["lang=\"", language, "\""].concat();
            let source = "<script setup lang=\"ts\">\nconst count = 1\nconst ok = true\nfunction inc() {}\n</script>\n<template lang=\"pug\">\ndiv(v-if=\"ok\")\n  button(@click=\"inc\") add\n  p(:title=\"count\") {{ count }}\n</template>\n"
                .replace("lang=\"pug\"", &language_attr);
            fs::write(&path, &source).unwrap();
            let uri = Url::from_file_path(&path).unwrap();
            let state = ServerState::new();
            state
                .documents
                .open(uri.clone(), source.clone(), 1, "vue".to_string());
            state.update_virtual_docs(&uri, &source);

            let ctx = IdeContext::new(&state, &uri, source.rfind(cursor).unwrap() + 1).unwrap();
            let location = scalar_location(DefinitionService::definition(&ctx).unwrap());
            let (line, character) =
                crate::ide::offset_to_position(&source, source.find(binding).unwrap());
            assert_eq!(
                (location.range.start.line, location.range.start.character),
                (line, character),
                "{language}/{cursor}"
            );
            assert!(HoverService::hover(&ctx).is_some(), "{language}/{cursor}");
        }
    }
}

#[test]
fn script_binding_references_reach_pug_attribute_values() {
    for language in ["pug", "jade"] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Counter.vue");
        let language_attr = ["lang=\"", language, "\""].concat();
        let source = "<script setup lang=\"ts\">\nconst count = 1\n</script>\n<template lang=\"pug\">\np(\n  :title=\"count\"\n) {{ count }}\n</template>\n"
            .replace("lang=\"pug\"", &language_attr);
        fs::write(&path, &source).unwrap();
        let uri = Url::from_file_path(&path).unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), source.clone(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, &source);

        let attribute = source.find(":title=\"count\"").unwrap();
        let ctx = IdeContext::new(&state, &uri, attribute + ":title=\"".len() + 1).unwrap();
        let mut starts = crate::ide::ReferencesService::references(&ctx, true)
            .unwrap()
            .into_iter()
            .map(|location| (location.range.start.line, location.range.start.character))
            .collect::<Vec<_>>();
        starts.sort_unstable();
        let expected = source
            .match_indices("count")
            .map(|(offset, _)| crate::ide::offset_to_position(&source, offset))
            .collect::<Vec<_>>();
        assert_eq!(starts, expected, "{language}");

        let rename = crate::ide::RenameService::rename(&ctx, "total").expect("Pug rename");
        let edits = &rename.changes.as_ref().unwrap()[&uri];
        assert_eq!(edits.len(), expected.len(), "{language}");
        assert!(edits.iter().all(|edit| edit.new_text == "total"));
    }
}

#[test]
fn pug_multiline_directive_values_resolve_like_html_attributes() {
    for language in ["pug", "jade"] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Counter.vue");
        let language_attr = ["lang=\"", language, "\""].concat();
        let source = "<script setup lang=\"ts\">\nconst count = 1\nconst ok = true\nfunction inc() {}\n</script>\n<template lang=\"pug\">\nbutton(\n  v-if=\"ok\"\n  :title=\"count\"\n  @click=\"inc\"\n) {{ count }}\n</template>\n"
            .replace("lang=\"pug\"", &language_attr);
        fs::write(&path, &source).unwrap();
        let uri = Url::from_file_path(&path).unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), source.clone(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, &source);

        for (cursor, binding) in [
            ("ok\"", "ok = true"),
            ("count\"", "count = 1"),
            ("inc\"", "inc() {}"),
        ] {
            let ctx = IdeContext::new(&state, &uri, source.find(cursor).unwrap() + 1).unwrap();
            let location = scalar_location(DefinitionService::definition(&ctx).unwrap());
            let (line, character) =
                crate::ide::offset_to_position(&source, source.find(binding).unwrap());
            assert_eq!(
                (location.range.start.line, location.range.start.character),
                (line, character),
                "{language}/{cursor}"
            );
            assert!(HoverService::hover(&ctx).is_some(), "{language}/{cursor}");
        }
    }
}
