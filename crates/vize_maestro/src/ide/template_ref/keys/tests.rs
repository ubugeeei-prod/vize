use tower_lsp::lsp_types::{PrepareRenameResponse, Url, WorkspaceEdit};

use super::key_at_offset;
use crate::ide::{IdeContext, ReferencesService, RenameService};
use crate::server::ServerState;

const SFC: &str = r#"<script setup lang="ts">
import { useTemplateRef as pick } from 'vue'
const dialog = pick<HTMLDialogElement>('box')
const other = pick('other')
</script>

<template>
  <dialog ref="box" />
  <ul><li v-for="item in 3" :key="item"><span ref='box' /></li></ul>
  <input ref="other" title="box" />
</template>
"#;

fn context<'a>(state: &'a ServerState, uri: &'a Url, source: &str, needle: &str) -> IdeContext<'a> {
    let offset = source.find(needle).unwrap() + 1;
    IdeContext::testing(state, uri, offset, source.into())
}

fn spelled(source: &str, edit: &WorkspaceEdit) -> String {
    let mut edits = edit
        .changes
        .as_ref()
        .unwrap()
        .values()
        .next()
        .unwrap()
        .clone();
    edits.sort_by_key(|edit| std::cmp::Reverse(edit.range.start));
    let mut source = source.to_string();
    for edit in edits {
        let start = crate::ide::position_to_offset(
            &source,
            edit.range.start.line,
            edit.range.start.character,
        )
        .unwrap();
        let end =
            crate::ide::position_to_offset(&source, edit.range.end.line, edit.range.end.character)
                .unwrap();
        source.replace_range(start..end, &edit.new_text);
    }
    source
}

#[test]
fn one_rename_rewrites_the_call_and_every_static_ref() {
    let state = ServerState::new();
    let uri = Url::parse("file:///workspace/App.vue").unwrap();
    let expected = SFC
        .replace("('box')", "('panel')")
        .replace("ref=\"box\"", "ref=\"panel\"")
        .replace("ref='box'", "ref='panel'");
    for needle in ["box')", "box\" />", "box' />"] {
        let ctx = context(&state, &uri, SFC, needle);
        let edit = RenameService::rename(&ctx, "panel").unwrap();
        assert_eq!(spelled(SFC, &edit), expected, "from {needle}");
        assert_eq!(
            ReferencesService::references(&ctx, false).unwrap().len(),
            3,
            "from {needle}"
        );
        let Some(PrepareRenameResponse::Range(range)) = RenameService::prepare_rename(&ctx) else {
            panic!("no prepare range from {needle}");
        };
        let start =
            crate::ide::position_to_offset(SFC, range.start.line, range.start.character).unwrap();
        assert_eq!(&SFC[start..start + 3], "box");
    }
    // A key is any attribute-safe string, not only an identifier.
    let ctx = context(&state, &uri, SFC, "other')");
    let edit = RenameService::rename(&ctx, "my-input").unwrap();
    assert_eq!(
        spelled(SFC, &edit),
        SFC.replace("('other')", "('my-input')")
            .replace("ref=\"other\"", "ref=\"my-input\"")
    );
    assert!(RenameService::rename(&ctx, "a b").is_none());
    assert!(RenameService::rename(&ctx, "it's").is_none());
}

#[test]
fn the_binding_and_unrelated_strings_keep_their_own_identity() {
    let state = ServerState::new();
    let uri = Url::parse("file:///workspace/App.vue").unwrap();
    for needle in ["dialog =", "box\" />\n</template>", "item in"] {
        let ctx = context(&state, &uri, SFC, needle);
        assert!(key_at_offset(&ctx).is_none(), "at {needle}");
    }
}

#[test]
fn keys_with_a_spelling_the_rename_cannot_follow_are_refused() {
    let state = ServerState::new();
    let uri = Url::parse("file:///workspace/App.vue").unwrap();
    let script = |body: &str| {
        [
            "<script setup lang=\"ts\">\nimport { useTemplateRef } from 'vue'\n",
            body,
            "\n</script>\n<template><div ref=\"box\" /></template>\n",
        ]
        .concat()
    };
    let refused = [
        script("const el = useTemplateRef('box')\nconst name = 'box'\nuseTemplateRef(name)"),
        script("const el = useTemplateRef('box')\nconst e = useTemplateRef('b\\x6fx')"),
        script("const el = useTemplateRef('box')\nfunction f(this: any) { return this.$refs.box }"),
        script("const el = useTemplateRef('box')").replace(
            "<div ref=\"box\" />",
            "<div ref=\"box\" /><p :ref=\"'box'\" />",
        ),
        // A same-named function from another module is not Vue's.
        script("const el = useTemplateRef('box')").replace("from 'vue'", "from './local'"),
        // A ref nothing passes to `useTemplateRef` stays a plain attribute.
        script("const el = 1"),
    ];
    for source in &refused {
        let ctx = context(&state, &uri, source, "box\" />");
        assert!(key_at_offset(&ctx).is_none(), "{source}");
        assert!(RenameService::rename(&ctx, "panel").is_none(), "{source}");
    }
    // An unimported call is an auto-import and is followed.
    let auto = script("const el = useTemplateRef('box')")
        .replace("import { useTemplateRef } from 'vue'\n", "");
    let ctx = context(&state, &uri, &auto, "box\" />");
    assert_eq!(key_at_offset(&ctx).unwrap().name, "box");
}

#[cfg(feature = "native")]
#[test]
fn native_entry_points_answer_before_the_checker() {
    crate::runtime::block_on(async {
        let state = ServerState::new();
        let uri = Url::parse("file:///workspace/App.vue").unwrap();
        state
            .documents
            .open(uri.clone(), SFC.into(), 1, "vue".into());
        state.update_virtual_docs(&uri, SFC);
        let offset = SFC.find("box\" />").unwrap() + 1;
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        assert_eq!(
            RenameService::rename_with_corsa(&ctx, "panel", None).await,
            RenameService::rename(&ctx, "panel")
        );
        assert_eq!(
            RenameService::prepare_rename_with_corsa(&ctx, None).await,
            RenameService::prepare_rename(&ctx)
        );
        assert_eq!(
            ReferencesService::references_with_corsa(&ctx, true, None)
                .await
                .map(|locations| locations.len()),
            Some(3)
        );
    });
}
