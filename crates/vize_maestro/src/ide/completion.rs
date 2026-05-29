//! Completion provider for Vue SFC files.
//!
//! Provides context-aware completions for:
//! - Template expressions and directives
//! - Script bindings and imports
//! - CSS properties and Vue-specific selectors
//! - Real completions from Corsa (when available)
//!
//! Uses vize_croquis for accurate scope analysis and type information.
#![allow(clippy::disallowed_methods)]

mod items;
mod script;
mod service;
mod style;
mod template;

// Cross-module reuse: inlay-hint code resolves reactive binding types with
// the same heuristic that script completion uses.
pub(crate) use script::infer_reactive_value_type;

/// Completion service for providing context-aware completions.
pub struct CompletionService;

/// Completion trigger characters for Vue SFC.
pub const TRIGGER_CHARACTERS: &[char] = &[
    '<',  // HTML tags
    '.',  // Object property access
    ':',  // v-bind shorthand
    '@',  // v-on shorthand
    '#',  // v-slot shorthand
    '"',  // Attribute values
    '\'', // Attribute values
    '/',  // Closing tags
    ' ',  // Space for attribute completion
];

/// Get trigger characters as strings.
pub fn trigger_characters() -> Vec<String> {
    TRIGGER_CHARACTERS.iter().map(|c| c.to_string()).collect()
}

// =============================================================================
// Context detection helpers
// =============================================================================

/// Check if cursor offset is inside an HTML comment (`<!-- ... -->`).
fn is_inside_html_comment(content: &str, offset: usize) -> bool {
    let before = &content[..offset.min(content.len())];
    if let Some(comment_start) = before.rfind("<!--") {
        let after_start = &before[comment_start + 4..];
        !after_start.contains("-->")
    } else {
        false
    }
}

/// Check if cursor is inside <art ...> opening tag.
fn is_inside_art_tag(before: &str) -> bool {
    if let Some(art_start) = before.rfind("<art") {
        let after_art = &before[art_start..];
        !after_art.contains('>')
    } else {
        false
    }
}

/// Check if cursor is inside <variant ...> opening tag.
fn is_inside_variant_tag(before: &str) -> bool {
    if let Some(variant_start) = before.rfind("<variant") {
        let after_variant = &before[variant_start..];
        !after_variant.contains('>')
    } else {
        false
    }
}

/// Check if we should suggest <art> block at root level.
fn should_suggest_art_block(before: &str) -> bool {
    !before.contains("<art")
        && (before.trim().is_empty() || before.ends_with('\n') || before.ends_with('<'))
}

/// Check if we should suggest <variant> block inside <art>.
fn should_suggest_variant_block(before: &str) -> bool {
    if let Some(art_start) = before.rfind("<art") {
        let after_art = &before[art_start..];
        after_art.contains('>') && !after_art.contains("</art>")
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{
        CompletionService, is_inside_html_comment, items, script, style, template,
        trigger_characters,
    };
    use crate::{ide::IdeContext, server::ServerState};
    use tower_lsp::lsp_types::{CompletionItemKind, CompletionResponse, InsertTextFormat, Url};
    use vize_relief::BindingType;

    #[test]
    fn test_directive_completions() {
        let items = template::directive_completions();
        assert!(!items.is_empty());

        let v_if = items.iter().find(|i| i.label == "v-if");
        assert!(v_if.is_some());
        assert_eq!(v_if.unwrap().kind, Some(CompletionItemKind::KEYWORD));
    }

    #[test]
    fn test_petite_vue_directive_completions() {
        let items = template::petite_vue_directive_completions();
        let labels: Vec<&str> = items.iter().map(|item| item.label.as_str()).collect();

        assert!(labels.contains(&"v-scope"));
        assert!(labels.contains(&"v-effect"));
        assert!(labels.contains(&"@vue:mounted"));
        assert!(labels.contains(&"@vue:unmounted"));
    }

    #[test]
    fn test_composition_api_completions() {
        let items = script::composition_api_completions();
        assert!(!items.is_empty());

        let ref_item = items.iter().find(|i| i.label == "ref");
        assert!(ref_item.is_some());
        assert_eq!(ref_item.unwrap().kind, Some(CompletionItemKind::FUNCTION));
    }

    #[test]
    fn test_macro_completions() {
        let items = script::macro_completions();
        assert!(!items.is_empty());

        let define_props = items.iter().find(|i| i.label == "defineProps");
        assert!(define_props.is_some());
    }

    #[test]
    fn test_vue_css_completions() {
        let items = style::vue_css_completions();
        assert_eq!(items.len(), 4);

        let deep = items.iter().find(|i| i.label == ":deep");
        assert!(deep.is_some());
    }

    #[test]
    fn test_trigger_characters() {
        let chars = trigger_characters();
        assert!(chars.contains(&"<".to_string()));
        assert!(chars.contains(&":".to_string()));
        assert!(chars.contains(&"@".to_string()));
    }

    #[test]
    fn test_binding_type_to_completion_info() {
        let (kind, detail, _) = items::binding_type_to_completion_info(BindingType::SetupRef);
        assert_eq!(kind, CompletionItemKind::VARIABLE);
        insta::assert_snapshot!(detail);

        let (kind, detail, _) = items::binding_type_to_completion_info(BindingType::SetupConst);
        assert_eq!(kind, CompletionItemKind::CONSTANT);
        insta::assert_snapshot!(detail);

        let (kind, detail, _) = items::binding_type_to_completion_info(BindingType::Props);
        assert_eq!(kind, CompletionItemKind::PROPERTY);
        insta::assert_snapshot!(detail);
    }

    #[test]
    fn test_vize_directive_completions() {
        let items = template::vize_directive_completions();
        assert_eq!(items.len(), 9);

        for item in &items {
            assert_eq!(item.kind, Some(CompletionItemKind::KEYWORD));
        }

        let todo = items.iter().find(|i| i.label == "@vize:todo");
        assert!(todo.is_some());
        let todo = todo.unwrap();
        assert_eq!(todo.insert_text_format, Some(InsertTextFormat::SNIPPET));
        assert_eq!(todo.insert_text, Some("@vize:todo $1 ".to_string()));

        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert!(labels.contains(&"@vize:todo"));
        assert!(labels.contains(&"@vize:fixme"));
        assert!(labels.contains(&"@vize:expected"));
        assert!(labels.contains(&"@vize:docs"));
        assert!(labels.contains(&"@vize:ignore-start"));
        assert!(labels.contains(&"@vize:ignore-end"));
        assert!(labels.contains(&"@vize:level(warn)"));
        assert!(labels.contains(&"@vize:deprecated"));
        assert!(labels.contains(&"@vize:dev-only"));
    }

    #[test]
    fn test_is_inside_html_comment() {
        assert!(is_inside_html_comment("<!-- @vize:", 11));
        assert!(is_inside_html_comment("<!-- ", 5));
        assert!(is_inside_html_comment("<div><!-- hello", 15));

        assert!(!is_inside_html_comment("<div>", 5));
        assert!(!is_inside_html_comment("", 0));

        assert!(!is_inside_html_comment("<!-- done -->", 13));
        assert!(!is_inside_html_comment("<!-- done --> text", 18));

        assert!(is_inside_html_comment("<!-- a --> <!-- b", 17));
        assert!(!is_inside_html_comment("<!-- a --> <!-- b --> after", 26));
    }

    #[test]
    fn test_template_completion_skips_bindings_in_plain_text_node() {
        let source = r#"<script setup lang="ts">
const message = ref('hello')
</script>
<template>
  <div>Hello </div>
</template>
"#;
        let (state, uri) = state_with_document("PlainTextCompletion.vue", source);
        let offset = source.find("Hello ").unwrap() + "Hello ".len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let labels = completion_labels(CompletionService::complete(&ctx).unwrap());

        assert!(!has_label(&labels, "message"));
        assert!(has_label(&labels, "v-if"));
    }

    #[test]
    fn test_standalone_html_completion_includes_petite_vue_directives() {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("index.html");
        let source = r#"<script src="https://unpkg.com/petite-vue" defer init></script>
<div v-scope="{ count: 0 }" >{{ count }}</div>
"#;
        fs::write(&source_path, source).unwrap();

        let uri = Url::from_file_path(&source_path).unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "html".to_string());
        state.update_virtual_docs(&uri, source);

        let offset = source.find("<div ").unwrap() + "<div ".len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let labels = completion_labels(CompletionService::complete(&ctx).unwrap());

        assert!(has_label(&labels, "v-scope"));
        assert!(has_label(&labels, "v-effect"));
        assert!(has_label(&labels, "@vue:mounted"));
        assert!(has_label(&labels, "v-if"));
    }

    #[test]
    fn test_script_ref_member_completion_includes_value() {
        let source = r#"<script setup lang="ts">
import { ref, computed } from 'vue'

const count = ref(0)
const double = computed(() => count.value * 2)

count.
</script>
"#;
        let (state, uri) = state_with_document("ScriptRefMemberCompletion.vue", source);
        let offset = source.find("count.").unwrap() + "count.".len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let labels = completion_labels(CompletionService::complete(&ctx).unwrap());

        assert_eq!(labels, vec!["value"]);
    }

    #[test]
    fn test_template_completion_includes_v_for_binding() {
        // Cursor inside the v-for body should see the iteration variable.
        let source = r#"<script setup lang="ts">
const items = [1, 2, 3]
</script>
<template>
  <div v-for="item in items">{{ it }}</div>
</template>
"#;
        let (state, uri) = state_with_document("VForCompletion.vue", source);
        let offset = source.find("{{ it ").unwrap() + "{{ it".len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let labels = completion_labels(CompletionService::complete(&ctx).unwrap());
        assert!(
            labels.contains(&"item".to_string()),
            "v-for binding must be visible inside v-for body, got {labels:?}",
        );
    }

    #[test]
    fn test_template_completion_excludes_v_for_binding_outside() {
        // The same v-for binding must NOT leak outside the v-for subtree.
        let source = r#"<script setup lang="ts">
const items = [1, 2, 3]
</script>
<template>
  <div v-for="item in items">{{ item }}</div>
  <p>{{ ite }}</p>
</template>
"#;
        let (state, uri) = state_with_document("VForLeak.vue", source);
        let offset = source.find("{{ ite ").unwrap() + "{{ ite".len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let labels = completion_labels(CompletionService::complete(&ctx).unwrap());
        assert!(
            !labels.iter().any(|l| l == "item"),
            "v-for binding must not leak outside its subtree, got {labels:?}",
        );
    }

    #[test]
    fn test_script_completion_includes_closure_local_binding() {
        // Cursor inside a closure body should see the binding declared in
        // that closure as well as setup-scope siblings.
        let source = r#"<script setup lang="ts">
import { ref } from 'vue'

const outer = ref(0)

function increment() {
  const localStep = 1
  loc
}
</script>
"#;
        let (state, uri) = state_with_document("ScopeAwareCompletion.vue", source);
        let offset = source.find("\n  loc\n").unwrap() + "\n  loc".len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let items = completion_items(CompletionService::complete(&ctx).unwrap());
        let labels: Vec<&str> = items.iter().map(|item| item.label.as_str()).collect();

        assert!(
            labels.contains(&"localStep"),
            "closure local should be visible, got {labels:?}",
        );
        assert!(
            labels.contains(&"outer"),
            "setup-scope binding should be visible, got {labels:?}",
        );

        let local_item = items.iter().find(|item| item.label == "localStep").unwrap();
        let outer_item = items.iter().find(|item| item.label == "outer").unwrap();
        assert!(
            local_item.sort_text.as_deref().unwrap_or("")
                < outer_item.sort_text.as_deref().unwrap_or(""),
            "inner scope binding must sort before setup-scope binding: \
             local={:?}, outer={:?}",
            local_item.sort_text,
            outer_item.sort_text,
        );
    }

    #[test]
    fn test_script_completion_excludes_inner_binding_outside_its_scope() {
        // The same binding declared inside a closure must NOT leak into
        // completion at the module level.
        let source = r#"<script setup lang="ts">
import { ref } from 'vue'

function helper() {
  const onlyHere = 1
  void onlyHere
}

const outer = ref(0)
out
</script>
"#;
        let (state, uri) = state_with_document("ScopeAwareLeak.vue", source);
        let offset = source.find("\nout\n").unwrap() + "\nout".len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let labels = completion_labels(CompletionService::complete(&ctx).unwrap());

        assert!(
            !labels.iter().any(|l| l == "onlyHere"),
            "closure-local binding must not leak to setup scope, got {labels:?}",
        );
        assert!(labels.iter().any(|l| l == "outer"));
    }

    #[test]
    fn test_script_member_access_lists_reactive_object_keys() {
        // Follow-up to #678: `const obj = reactive({ a: 1, b: '' })` then
        // `obj.|` should surface `a` and `b` from the initializer even
        // when Corsa isn't available.
        let source = r#"<script setup lang="ts">
import { reactive } from 'vue'
const obj = reactive({ count: 0, label: 'hello' })
obj.
</script>
"#;
        let (state_, uri) = state_with_document("ReactiveKeys.vue", source);
        let offset = source.find("obj.\n").unwrap() + "obj.".len();
        let ctx = IdeContext::new(&state_, &uri, offset).unwrap();
        let labels = completion_labels(CompletionService::complete(&ctx).unwrap());
        assert!(
            labels.iter().any(|l| l == "count"),
            "expected `count` from reactive() keys, got {labels:?}",
        );
        assert!(
            labels.iter().any(|l| l == "label"),
            "expected `label` from reactive() keys, got {labels:?}",
        );
    }

    #[test]
    fn test_script_member_access_on_non_ref_returns_empty_in_sync_fallback() {
        // When Corsa is not available, the synchronous completion path used to
        // fall through to the full Composition-API + setup-bindings list at
        // `.|` sites, which is misleading because none of those names make
        // sense after a dot. The empty response lets the editor either show
        // nothing or pick up Corsa-backed members from `complete_with_corsa`.
        let source = r#"<script setup lang="ts">
const arr = [1, 2, 3]
arr.
</script>
"#;
        let (state, uri) = state_with_document("MemberAccessNonRef.vue", source);
        let offset = source.find("arr.").unwrap() + "arr.".len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let response = CompletionService::complete(&ctx);
        assert!(
            response.is_none(),
            "expected no completion at `arr.|`, got {response:?}",
        );
    }

    #[test]
    fn test_script_member_access_after_decimal_literal_is_not_member_access() {
        // `1.` is a decimal literal in progress, not a member access. The
        // sync fallback should NOT swallow completion here — the user is
        // still typing a number.
        let source = r#"<script setup lang="ts">
const n = 1.
</script>
"#;
        let (state, uri) = state_with_document("DecimalLiteralCompletion.vue", source);
        let offset = source.find("1.").unwrap() + "1.".len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let labels = completion_labels(CompletionService::complete(&ctx).unwrap());
        // The response is not the empty short-circuit — completion proceeds
        // and offers at least the standard Composition-API items.
        assert!(labels.contains(&"ref".to_string()));
    }

    #[test]
    fn test_script_completion_infers_computed_ref_type() {
        let source = r#"<script setup lang="ts">
import { ref, computed } from 'vue'

const count = ref(0)
const double = computed(() => count.value * 2)

doub
</script>
"#;
        let (state, uri) = state_with_document("ScriptComputedCompletion.vue", source);
        let offset = source.find("doub").unwrap() + "doub".len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let items = completion_items(CompletionService::complete(&ctx).unwrap());
        let double = items
            .iter()
            .find(|item| item.label == "double")
            .expect("double completion should be present");

        assert_eq!(double.detail.as_deref(), Some("ComputedRef<number>"));
    }

    #[test]
    fn test_template_completion_skips_bindings_in_static_attribute_value() {
        let source = r#"<script setup lang="ts">
const message = ref('hello')
</script>
<template>
  <div title="message" />
</template>
"#;
        let (state, uri) = state_with_document("StaticAttributeCompletion.vue", source);
        let offset = source.rfind("message\"").unwrap() + "message".len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let labels = completion_labels(CompletionService::complete(&ctx).unwrap());

        assert!(!has_label(&labels, "message"));
        assert!(has_label(&labels, "v-if"));
    }

    #[test]
    fn test_template_completion_keeps_bindings_in_dynamic_attribute_value() {
        let source = r#"<script setup lang="ts">
const message = ref('hello')
</script>
<template>
  <div :title = "message" />
</template>
"#;
        let (state, uri) = state_with_document("DynamicAttributeCompletion.vue", source);
        let offset = source.rfind("message").unwrap() + "message".len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let labels = completion_labels(CompletionService::complete(&ctx).unwrap());

        assert!(has_label(&labels, "message"));
        assert!(has_label(&labels, "v-if"));
    }

    #[test]
    fn test_art_variant_completion_includes_script_bindings_in_non_default_variant() {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("Button.art.vue");
        let source = r#"<script setup lang="ts">
const primaryLabel = ref('primary')
const secondaryLabel = ref('secondary')
</script>

<art title="Button" component="./Button.vue">
  <variant name="Primary" default>
    <Button>{{ primaryLabel }}</Button>
  </variant>
  <variant name="Secondary">
    <Button>{{ secondaryLabel }}</Button>
  </variant>
</art>
"#;
        fs::write(&source_path, source).unwrap();

        let uri = Url::from_file_path(&source_path).unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "art-vue".to_string());
        state.update_virtual_docs(&uri, source);

        let offset = source.rfind("secondaryLabel").unwrap() + "secondaryLabel".len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let response = CompletionService::complete(&ctx).unwrap();
        let items = match response {
            CompletionResponse::Array(items) => items,
            CompletionResponse::List(list) => list.items,
        };

        let labels: Vec<&str> = items.iter().map(|item| item.label.as_str()).collect();
        assert!(labels.contains(&"secondaryLabel"));
        assert!(labels.contains(&"primaryLabel"));
        assert!(labels.contains(&"v-if"));
    }

    #[test]
    fn test_art_variant_completion_infers_imported_component_props() {
        let dir = tempfile::tempdir().unwrap();
        let component_path = dir.path().join("Button.vue");
        fs::write(
            &component_path,
            r#"<script setup lang="ts">
defineProps<{
  label: string
  disabled?: boolean
  modelValue?: string
}>()
</script>
<template><button><slot /></button></template>
"#,
        )
        .unwrap();

        let art_path = dir.path().join("Button.art.vue");
        let source = r#"<script setup lang="ts">
import Button from "./Button.vue"
</script>

<art title="Button" component="./Button.vue">
  <variant name="Default">
    <Button  />
  </variant>
</art>
"#;
        fs::write(&art_path, source).unwrap();

        let uri = Url::from_file_path(&art_path).unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "art-vue".to_string());
        state.update_virtual_docs(&uri, source);

        let offset = source.find("<Button  />").unwrap() + "<Button ".len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let items = completion_items(CompletionService::complete(&ctx).unwrap());
        let labels: Vec<String> = items.iter().map(|item| item.label.clone()).collect();

        assert!(has_label(&labels, "label"));
        assert!(has_label(&labels, "disabled"));
        assert!(has_label(&labels, "model-value"));

        let label = items.iter().find(|item| item.label == "label").unwrap();
        assert!(
            label
                .detail
                .as_deref()
                .unwrap_or_default()
                .contains("string")
        );
    }

    #[test]
    fn test_art_variant_completion_infers_imported_component_slots() {
        let dir = tempfile::tempdir().unwrap();
        let component_path = dir.path().join("Button.vue");
        fs::write(
            &component_path,
            r#"<script setup lang="ts">
defineSlots<{
  default(): any
  icon(props: { size: number }): any
}>()
</script>
<template>
  <button><slot /><slot name="suffix" /></button>
</template>
"#,
        )
        .unwrap();

        let art_path = dir.path().join("Button.art.vue");
        let source = r#"<script setup lang="ts">
import Button from "./Button.vue"
</script>

<art title="Button" component="./Button.vue">
  <variant name="Default">
    <Button>
      <template #></template>
    </Button>
  </variant>
</art>
"#;
        fs::write(&art_path, source).unwrap();

        let uri = Url::from_file_path(&art_path).unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "art-vue".to_string());
        state.update_virtual_docs(&uri, source);

        let offset = source.find("<template #").unwrap() + "<template #".len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let labels = completion_labels(CompletionService::complete(&ctx).unwrap());

        assert!(has_label(&labels, "default"));
        assert!(has_label(&labels, "icon"));
        assert!(has_label(&labels, "suffix"));
    }

    #[test]
    fn test_art_variant_completion_uses_art_component_attribute_for_props() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("Switch.vue"),
            r#"<script setup lang="ts">
defineProps<{ checked?: boolean }>()
</script>
"#,
        )
        .unwrap();

        let art_path = dir.path().join("Switch.art.vue");
        let source = r#"<art title="Switch" component="./Switch.vue">
  <variant name="Default">
    <Switch  />
  </variant>
</art>
"#;
        fs::write(&art_path, source).unwrap();

        let uri = Url::from_file_path(&art_path).unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "art-vue".to_string());
        state.update_virtual_docs(&uri, source);

        let offset = source.find("<Switch  />").unwrap() + "<Switch ".len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let labels = completion_labels(CompletionService::complete(&ctx).unwrap());

        assert!(has_label(&labels, "checked"));
    }

    #[test]
    fn test_art_variant_completion_uses_define_art_component_for_props() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("Switch.vue"),
            r#"<script setup lang="ts">
defineProps<{ checked?: boolean }>()
</script>
"#,
        )
        .unwrap();

        let art_path = dir.path().join("Switch.art.vue");
        let source = r#"<script setup lang="ts">
defineArt("./Switch.vue", {
  title: "Switch",
})
</script>

<art>
  <variant name="Default">
    <Switch  />
  </variant>
</art>
"#;
        fs::write(&art_path, source).unwrap();

        let uri = Url::from_file_path(&art_path).unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "art-vue".to_string());
        state.update_virtual_docs(&uri, source);

        let offset = source.find("<Switch  />").unwrap() + "<Switch ".len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let labels = completion_labels(CompletionService::complete(&ctx).unwrap());

        assert!(has_label(&labels, "checked"));
    }

    #[test]
    fn test_define_art_source_completion_suggests_vue_files() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("components")).unwrap();
        fs::write(dir.path().join("Button.vue"), "<template />").unwrap();
        fs::write(dir.path().join("components/IconButton.vue"), "<template />").unwrap();

        let art_path = dir.path().join("Button.art.vue");
        let source = r#"<script setup lang="ts">
defineArt("./", {
  title: "Button",
});
</script>

<art>
  <variant name="Default">
    <Button />
  </variant>
</art>
"#;
        fs::write(&art_path, source).unwrap();

        let uri = Url::from_file_path(&art_path).unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "art-vue".to_string());
        state.update_virtual_docs(&uri, source);

        let offset = source.find("\"./").unwrap() + "\"./".len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let labels = completion_labels(CompletionService::complete(&ctx).unwrap());

        assert!(has_label(&labels, "./Button.vue"));
        assert!(has_label(&labels, "./components/"));
    }

    #[test]
    fn test_art_variant_completion_includes_script_setup_state_bindings() {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("Counter.art.vue");
        let source = r#"<script setup lang="ts">
const count = ref(0)
const doubled = computed(() => count.value * 2)
</script>

<art title="Counter" component="./Counter.vue">
  <variant name="Default">
    <Counter :count="" />
  </variant>
</art>
"#;
        fs::write(&source_path, source).unwrap();

        let uri = Url::from_file_path(&source_path).unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "art-vue".to_string());
        state.update_virtual_docs(&uri, source);

        let offset = source.find(":count=\"\"").unwrap() + ":count=\"".len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let labels = completion_labels(CompletionService::complete(&ctx).unwrap());

        assert!(has_label(&labels, "count"));
        assert!(has_label(&labels, "doubled"));
    }

    #[test]
    fn test_i18n_key_completion_from_same_file_json_block() {
        let source = r#"<template>
  <p>{{ $t("auth.") }}</p>
</template>
<i18n lang="json">
{
  "en": {
    "auth": { "login": "Log in" }
  }
}
</i18n>
"#;
        let (state, uri) = state_with_document("I18nCompletion.vue", source);
        state.apply_lsp_initialization_options(Some(&serde_json::json!({ "ecosystem": true })));
        let offset = source.find("auth.").unwrap() + "auth.".len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let labels = completion_labels(CompletionService::complete(&ctx).unwrap());

        assert_eq!(labels, vec!["auth.login"]);
    }

    #[test]
    fn test_route_name_completion_from_same_file_route_sources() {
        let source = r#"<script setup>
definePage({ name: "settings" })
router.push({ name: "" })
</script>
<route lang="json">
{ "name": "home" }
</route>
"#;
        let (state, uri) = state_with_document("RouteCompletion.vue", source);
        state.apply_lsp_initialization_options(Some(&serde_json::json!({ "ecosystem": true })));
        let offset = source.find("name: \"\"").unwrap() + "name: \"".len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let labels = completion_labels(CompletionService::complete(&ctx).unwrap());

        assert_eq!(labels, vec!["home", "settings"]);
    }

    #[test]
    fn test_route_param_completion_from_file_route_name() {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("src/pages/users/[id].vue");
        fs::create_dir_all(source_path.parent().unwrap()).unwrap();
        let source = r#"<script setup lang="ts">
import { useRoute } from "vue-router"
const route = useRoute()
route.params.
</script>
"#;
        fs::write(&source_path, source).unwrap();

        let uri = Url::from_file_path(&source_path).unwrap();
        let state = ServerState::new();
        state.apply_lsp_initialization_options(Some(&serde_json::json!({ "ecosystem": true })));
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, source);

        let offset = source.find("route.params.").unwrap() + "route.params.".len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let labels = completion_labels(CompletionService::complete(&ctx).unwrap());

        assert!(has_label(&labels, "id"));
    }

    #[test]
    fn test_route_param_completion_from_route_path() {
        let source = r#"<script setup lang="ts">
import { useRoute } from "vue-router"
const route = useRoute()
route.params.
</script>
<route lang="json">
{ "name": "article", "path": "/articles/:id(\\d+)/:tab?" }
</route>
"#;
        let (state, uri) = state_with_document("RoutePathCompletion.vue", source);
        state.apply_lsp_initialization_options(Some(&serde_json::json!({ "ecosystem": true })));
        let offset = source.find("route.params.").unwrap() + "route.params.".len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let labels = completion_labels(CompletionService::complete(&ctx).unwrap());

        assert_eq!(labels, vec!["id", "tab"]);
    }

    #[test]
    fn test_i18n_key_completion_from_workspace_json_catalog() {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("src/components/LoginButton.vue");
        let locale_path = dir.path().join("src/locales/en.json");
        fs::create_dir_all(source_path.parent().unwrap()).unwrap();
        fs::create_dir_all(locale_path.parent().unwrap()).unwrap();
        fs::write(&locale_path, r#"{ "auth": { "login": "Log in" } }"#).unwrap();

        let source = r#"<script setup lang="ts">
const title = t("auth.")
</script>
"#;
        fs::write(&source_path, source).unwrap();

        let uri = Url::from_file_path(&source_path).unwrap();
        let state = ServerState::new();
        state.apply_lsp_initialization_options(Some(&serde_json::json!({ "ecosystem": true })));
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, source);

        let offset = source.find("auth.").unwrap() + "auth.".len();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let labels = completion_labels(CompletionService::complete(&ctx).unwrap());

        assert_eq!(labels, vec!["auth.login"]);
    }

    #[test]
    fn test_script_completion_lists_reactive_binding_once() {
        // A ref/computed binding is both a binding and a reactive source.
        // Completion must surface each name once, not twice.
        let source = r#"<script setup lang="ts">
import { ref, computed } from 'vue'
const st = ref(0)
const ts = computed(() => st.value * 2)
st
</script>
"#;
        let (state, uri) = state_with_document("ScriptDedup.vue", source);
        let offset = source.rfind("st\n").unwrap() + 2;
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let labels = completion_labels(CompletionService::complete(&ctx).unwrap());

        assert_eq!(labels.iter().filter(|l| l.as_str() == "st").count(), 1);
        assert_eq!(labels.iter().filter(|l| l.as_str() == "ts").count(), 1);
    }

    #[test]
    fn test_template_completion_lists_reactive_binding_once() {
        let source = r#"<script setup lang="ts">
import { ref, computed } from 'vue'
const st = ref(0)
const ts = computed(() => st.value * 2)
</script>
<template>
  <div>{{ st }}</div>
</template>
"#;
        let (state, uri) = state_with_document("TemplateDedup.vue", source);
        let offset = source.rfind("st }}").unwrap() + 2;
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let labels = completion_labels(CompletionService::complete(&ctx).unwrap());

        assert_eq!(labels.iter().filter(|l| l.as_str() == "st").count(), 1);
        assert_eq!(labels.iter().filter(|l| l.as_str() == "ts").count(), 1);
    }

    fn completion_labels(response: CompletionResponse) -> Vec<String> {
        completion_items(response)
            .into_iter()
            .map(|item| item.label)
            .collect()
    }

    fn completion_items(response: CompletionResponse) -> Vec<tower_lsp::lsp_types::CompletionItem> {
        match response {
            CompletionResponse::Array(items) => items,
            CompletionResponse::List(list) => list.items,
        }
    }

    fn has_label(labels: &[String], expected: &str) -> bool {
        labels.iter().any(|label| label == expected)
    }

    fn state_with_document(name: &str, source: &str) -> (ServerState, Url) {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join(name);
        fs::write(&source_path, source).unwrap();

        let uri = Url::from_file_path(&source_path).unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, source);

        (state, uri)
    }
}
