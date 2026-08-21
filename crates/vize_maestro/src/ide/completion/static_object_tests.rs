//! Exactness regressions for source-local object member completion.

mod escape_tests;

use std::fs;

use tower_lsp::lsp_types::{CompletionResponse, Url};

use super::CompletionService;
use crate::{ide::IdeContext, server::ServerState};

#[test]
fn lists_exact_static_object_keys() {
    let source = r#"<script setup lang="ts">
const probe = {
  pinnacle: 1,
  quaver: 'two',
  tessellate() { return 3 },
  nested: { ignored: true },
  'quoted-key': 4,
  ['computed-static']: 5,
}
const chosen = probe.pinnacle
</script>
"#;
    let (state, uri) = state_with_document("StaticObjectKeys.vue", source);
    let offset = source.find("probe.pinnacle").unwrap() + "probe.".len();
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();
    let expected = [
        "pinnacle",
        "quaver",
        "tessellate",
        "nested",
        "quoted-key",
        "computed-static",
    ];

    assert_eq!(
        completion_labels(CompletionService::complete(&ctx).unwrap()),
        expected.map(str::to_string),
        "only top-level statically named keys belong in the exact local answer",
    );

    #[cfg(feature = "native")]
    assert_eq!(
        completion_labels(CompletionService::complete_static_object_member(&ctx).unwrap()),
        expected.map(str::to_string),
        "the pre-Corsa fast path must preserve the same exact answer",
    );
}

#[test]
fn spread_falls_back_to_type_service() {
    assert_falls_back(
        "StaticObjectSpread.vue",
        r#"<script setup lang="ts">
const inherited = { remote: 1 }
const probe = { ...inherited, local: 2 }
const chosen = probe.local
</script>
"#,
        "probe.local",
    );
}

#[test]
fn nested_receiver_shadowing_falls_back_to_type_service() {
    assert_falls_back(
        "StaticObjectShadow.vue",
        r#"<script setup lang="ts">
const probe = { outer: 1 }
function inspect() {
  const probe = getDynamicProbe()
  return probe.value
}
</script>
"#,
        "probe.value",
    );
}

#[test]
fn mutable_binding_falls_back_to_type_service() {
    assert_falls_back(
        "MutableStaticObject.vue",
        r#"<script setup lang="ts">
let probe = { initial: 1 }
probe = getDynamicProbe()
const chosen = probe.value
</script>
"#,
        "probe.value",
    );
}

#[test]
fn property_added_before_completion_falls_back_to_type_service() {
    assert_falls_back(
        "MutatedStaticObject.vue",
        r#"<script setup lang="ts">
const probe = { initial: 1 }
probe.added = 2
const chosen = probe.initial
</script>
"#,
        "probe.initial",
    );
}

#[test]
fn deleted_property_before_completion_falls_back_to_type_service() {
    assert_falls_back(
        "DeletedStaticObjectProperty.vue",
        r#"<script setup lang="ts">
const probe = { initial: 1, removed: 2 }
delete probe.removed
const chosen = probe.initial
</script>
"#,
        "probe.initial",
    );
}

#[test]
fn updated_nested_property_before_completion_falls_back_to_type_service() {
    assert_falls_back(
        "UpdatedStaticObjectProperty.vue",
        r#"<script setup lang="ts">
const probe = { initial: 1, nested: { count: 2 } }
probe.nested.count++
const chosen = probe.initial
</script>
"#,
        "probe.initial",
    );
}

#[test]
fn receiver_alias_before_completion_falls_back_to_type_service() {
    assert_falls_back(
        "AliasedStaticObject.vue",
        r#"<script setup lang="ts">
const probe = { initial: 1 }
const alias = probe
alias.added = 2
const chosen = probe.initial
</script>
"#,
        "probe.initial",
    );
}

#[test]
fn receiver_call_escape_before_completion_falls_back_to_type_service() {
    assert_falls_back(
        "EscapedStaticObject.vue",
        r#"<script setup lang="ts">
const probe = { initial: 1 }
mutate(probe)
const chosen = probe.initial
</script>
"#,
        "probe.initial",
    );
}

#[test]
fn receiver_nested_container_before_completion_falls_back_to_type_service() {
    assert_falls_back(
        "NestedAliasedStaticObject.vue",
        r#"<script setup lang="ts">
const probe = { initial: 1 }
const holder = { nested: [probe] }
holder.nested[0].added = 2
const chosen = probe.initial
</script>
"#,
        "probe.initial",
    );
}

#[test]
fn receiver_returned_from_closure_before_completion_falls_back_to_type_service() {
    assert_falls_back(
        "ClosureAliasedStaticObject.vue",
        r#"<script setup lang="ts">
const probe = { initial: 1 }
const getProbe = () => probe
const alias = getProbe()
alias.added = 2
const chosen = probe.initial
</script>
"#,
        "probe.initial",
    );
}

#[test]
fn receiver_member_read_before_completion_keeps_static_path() {
    let source = r#"<script setup lang="ts">
const probe = { initial: 1 }
const read = probe.initial
const readLater = () => probe.initial
readLater()
const chosen = probe.initial
</script>
"#;
    let (state, uri) = state_with_document("ReadStaticObject.vue", source);
    let offset = source.rfind("probe.initial").unwrap() + "probe.".len();
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();

    assert_eq!(
        completion_labels(CompletionService::complete(&ctx).unwrap()),
        ["initial"],
        "a member read must not be treated as an identity escape",
    );

    #[cfg(feature = "native")]
    assert_eq!(
        completion_labels(CompletionService::complete_static_object_member(&ctx).unwrap()),
        ["initial"],
    );
}

#[test]
fn later_declaration_is_not_visible_in_the_temporal_dead_zone() {
    assert_falls_back(
        "StaticObjectTemporalDeadZone.vue",
        r#"<script setup lang="ts">
const chosen = probe.value
const probe = { value: 1 }
</script>
"#,
        "probe.value",
    );
}

#[test]
fn dynamic_computed_key_falls_back_to_type_service() {
    assert_falls_back(
        "DynamicComputedObject.vue",
        r#"<script setup lang="ts">
const dynamicKey = getKey()
const probe = { fixed: 1, [dynamicKey]: 2 }
const chosen = probe.fixed
</script>
"#,
        "probe.fixed",
    );
}

#[test]
fn annotated_object_falls_back_when_type_contributes_members() {
    assert_falls_back(
        "AnnotatedStaticObject.vue",
        r#"<script setup lang="ts">
const probe: { fixed: number; optional?: string } = { fixed: 1 }
const chosen = probe.fixed
</script>
"#,
        "probe.fixed",
    );
}

#[test]
fn asserted_object_falls_back_when_type_contributes_members() {
    assert_falls_back(
        "AssertedStaticObject.vue",
        r#"<script setup lang="ts">
const probe = { fixed: 1 } as { fixed: number; asserted: string }
const chosen = probe.fixed
</script>
"#,
        "probe.fixed",
    );
}

#[test]
fn recovered_script_falls_back_to_type_service() {
    assert_falls_back(
        "RecoveredObjectScript.vue",
        r#"<script setup lang="ts">
const probe = { fixed: 1 }
const broken =
const chosen = probe.fixed
</script>
"#,
        "probe.fixed",
    );
}

fn assert_falls_back(name: &str, source: &str, access: &str) {
    let (state, uri) = state_with_document(name, source);
    let offset = source.find(access).unwrap() + "probe.".len();
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();

    assert!(
        CompletionService::complete(&ctx).is_none(),
        "an inexact source-local answer must fall back to the type service",
    );
    #[cfg(feature = "native")]
    assert!(CompletionService::complete_static_object_member(&ctx).is_none());
}

fn completion_labels(response: CompletionResponse) -> Vec<String> {
    match response {
        CompletionResponse::Array(items) => items,
        CompletionResponse::List(list) => list.items,
    }
    .into_iter()
    .map(|item| item.label)
    .collect()
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
