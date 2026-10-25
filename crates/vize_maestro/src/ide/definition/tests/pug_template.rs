use std::fs;

#[cfg(feature = "native")]
use tower_lsp::lsp_types::HoverContents;
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

#[cfg(feature = "native")]
#[test]
fn pug_attribute_names_resolve_component_and_native_contracts() {
    for language in ["pug", "jade"] {
        let dir = tempfile::tempdir().unwrap();
        let child_path = dir.path().join("Child.vue");
        let parent_path = dir.path().join("Parent.vue");
        let child = r#"<script setup lang="ts">
defineProps<{ message: string }>()
defineEmits<{ 'save-item': [id: string] }>()
</script>
<template><p>{{ message }}</p></template>
"#;
        fs::write(&child_path, child).unwrap();
        let language_attr = ["lang=\"", language, "\""].concat();
        let source = r#"<script setup lang="ts">
import Child from './Child.vue'
const label = 'hello'
function save() {}
</script>
<template lang="pug">
Child(
  :message="label"
  @save-item.once="save"
)
input(
  autocomplete="email"
)
</template>
"#
        .replace("lang=\"pug\"", &language_attr);
        fs::write(&parent_path, &source).unwrap();
        let uri = Url::from_file_path(&parent_path).unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), source.clone(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, &source);

        let prop_offset = source.find(":message").unwrap() + 2;
        let prop_ctx = IdeContext::new(&state, &uri, prop_offset).unwrap();
        let prop_definition =
            scalar_location(DefinitionService::definition(&prop_ctx).expect("prop definition"));
        assert_eq!(
            prop_definition
                .uri
                .to_file_path()
                .unwrap()
                .canonicalize()
                .unwrap(),
            child_path.canonicalize().unwrap(),
            "{language}"
        );
        let expected_prop = crate::ide::offset_to_position(child, child.find("message:").unwrap());
        assert_eq!(
            (
                prop_definition.range.start.line,
                prop_definition.range.start.character,
            ),
            expected_prop,
            "{language}"
        );
        assert_eq!(
            hover_markdown(HoverService::hover(&prop_ctx).expect("prop hover")),
            "**message**\n\n_Component prop_\n\n```typescript\nmessage: string\n```\n\n**Requirement**\n\nRequired\n\n**Example**\n\n```vue\n<Child message=\"...\" />\n<Child :message=\"value\" />\n```\n\n**Docs**\n\n[Vue Component Props](https://vuejs.org/guide/components/props.html)",
            "{language}",
        );

        let event_offset = source.find("save-item").unwrap() + 2;
        let event_ctx = IdeContext::new(&state, &uri, event_offset).unwrap();
        let event_definition = scalar_location(
            DefinitionService::definition(&event_ctx).expect("component event definition"),
        );
        let expected_event =
            crate::ide::offset_to_position(child, child.find("save-item").unwrap());
        assert_eq!(
            (
                event_definition.range.start.line,
                event_definition.range.start.character,
            ),
            expected_event,
            "{language}"
        );
        let event_hover = HoverService::hover(&event_ctx).expect("component event hover");
        assert_eq!(
            hover_markdown(event_hover.clone()),
            "**@save-item**\n\n_Component event_\n\n```typescript\nsave-item: [id: string]\n```\n\n**Payload**\n\n`[id: string]`\n\n**Example**\n\n```vue\n@save-item=\"handler\"\n```\n\n**Docs**\n\n[Vue Component Events](https://vuejs.org/guide/components/events.html)",
            "{language}",
        );
        let range = event_hover.range.expect("authored event range");
        let start =
            crate::ide::position_to_offset(&source, range.start.line, range.start.character)
                .unwrap();
        let end =
            crate::ide::position_to_offset(&source, range.end.line, range.end.character).unwrap();
        assert_eq!(source.get(start..end), Some("save-item"), "{language}");

        let native_offset = source.find("autocomplete").unwrap() + 2;
        let native_ctx = IdeContext::new(&state, &uri, native_offset).unwrap();
        let native_definition = scalar_location(
            DefinitionService::definition(&native_ctx).expect("native attribute definition"),
        );
        assert_eq!(native_definition.uri.scheme(), "https", "{language}");
        assert_eq!(
            hover_markdown(HoverService::hover(&native_ctx).expect("native attribute hover")),
            "**autocomplete on <input>**\n\n_HTML attribute_\n\n```typescript\nautocomplete: HTMLElementTagNameMap[\"input\"][\"autocomplete\"]\n```\n\nNative DOM attribute recognized by the Vue template compiler. Vue patches it on the platform element rather than resolving it as a component prop.\n\n**Runtime surface**\n\nDOM reflected attribute\n\n**Editor behavior**\n- Hover and go-to-definition use the DOM library type for the reflected property when the type service is available.\n- Component prop lookup is skipped because the enclosing tag is a native element.\n\n**Example**\n\n```vue\n<template>\n  <input autocomplete=\"...\">\n</template>\n```\n\n**Docs**\n\n[MDN reference](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Attributes/autocomplete)",
            "{language}",
        );
    }
}

#[cfg(feature = "native")]
fn hover_markdown(hover: tower_lsp::lsp_types::Hover) -> String {
    match hover.contents {
        HoverContents::Markup(content) => content.value,
        HoverContents::Scalar(value) => match value {
            tower_lsp::lsp_types::MarkedString::String(value) => value,
            tower_lsp::lsp_types::MarkedString::LanguageString(value) => value.value,
        },
        HoverContents::Array(parts) => parts
            .into_iter()
            .map(|part| match part {
                tower_lsp::lsp_types::MarkedString::String(value) => value,
                tower_lsp::lsp_types::MarkedString::LanguageString(value) => value.value,
            })
            .collect::<Vec<_>>()
            .join("\n"),
    }
}
