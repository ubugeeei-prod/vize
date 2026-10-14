use std::fs;

use tower_lsp::lsp_types::Url;

use super::super::DefinitionService;
use super::scalar_location;
use crate::{ide::IdeContext, server::ServerState};

#[test]
fn v_bind_in_every_preprocessor_block_resolves_to_the_script_binding() {
    for (lang, body) in [
        ("scss", ".foo { .bar { color: v-bind(color); } }"),
        ("less", ".foo { .bar { color: v-bind(color); } }"),
        ("sass", ".foo\n  .bar\n    color: v-bind(color)"),
        ("stylus", ".foo\n  .bar\n    color v-bind(color)"),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Styled.vue");
        let source = [
            "<script setup lang=\"ts\">\nconst color = 'red'\n</script>\n<style lang=\"",
            lang,
            "\">\n",
            body,
            "\n</style>\n",
        ]
        .concat();
        fs::write(&path, &source).unwrap();
        let uri = Url::from_file_path(&path).unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), source.clone(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, &source);

        let offset = source.find("v-bind(color").unwrap() + "v-bind(color".len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let location = scalar_location(DefinitionService::definition(&ctx).unwrap());
        let binding = source.find("const color").unwrap() + "const ".len();
        let (line, character) = crate::ide::offset_to_position(&source, binding);
        assert_eq!(
            (
                location.uri,
                location.range.start.line,
                location.range.start.character
            ),
            (uri, line, character),
            "{lang}"
        );
    }
}
