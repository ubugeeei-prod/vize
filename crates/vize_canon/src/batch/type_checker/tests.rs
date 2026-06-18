use super::{BatchTypeChecker, DeclarationEmitOptions};
use crate::batch::TypeChecker;
use crate::sfc_typecheck::{SfcTypeCheckOptions, type_check_sfc};
use corsa::{
    api::{ApiMode, ApiSpawnConfig, ProjectSession},
    runtime::block_on,
};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use vize_carton::{String, cstr};
mod emit_object_recursion;
mod generic_component_listener_payload;
mod no_check_props;
mod no_unused;
mod options_api_required_props;
mod scan;

#[test]
fn batch_type_checker_snapshots_vue_diagnostics() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let source = r#"<script setup lang="ts">
const count: number = 'oops'
</script>
"#;
    let virtual_ts = type_check_sfc(
        source,
        &SfcTypeCheckOptions::new("App.vue").with_virtual_ts(),
    )
    .virtual_ts
    .expect("virtual ts should be generated");
    let snapshot = corsa_type_mismatch_snapshot(&virtual_ts, "count: number", "'oops'");

    insta::with_settings!({
        snapshot_path => "../../snapshots"
    }, {
        insta::assert_debug_snapshot!("batch_type_checker_vue_diagnostics", snapshot);
    });
}

#[test]
fn batch_type_checker_snapshots_script_setup_type_error() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let virtual_ts = type_check_sfc(
        r#"<script setup lang="ts">
const count: string = 0;
</script>
"#,
        &SfcTypeCheckOptions::new("App.vue").with_virtual_ts(),
    )
    .virtual_ts
    .expect("virtual ts should be generated");
    let relevant = corsa_type_mismatch_snapshot(&virtual_ts, "count: string", "= 0");

    assert_eq!(
        relevant.len(),
        2,
        "expected declaration and initializer types, got: {relevant:#?}"
    );
    insta::with_settings!({
        snapshot_path => "../../snapshots"
    }, {
        insta::assert_debug_snapshot!("batch_type_checker_script_setup_type_error", relevant);
    });
}

#[test]
fn corsa_bridge_completion_returns_inner_members_for_chained_ref_value() {
    // Guards the wired Corsa completion path (see #751): when the bridge is
    // initialized, `count.value.` must surface `number`'s inner members
    // (`toFixed`, `toString`), proving that completion is not silently
    // collapsing to the heuristic fallback.
    use crate::corsa_bridge::{CorsaBridge, CorsaBridgeConfig};

    let Some(corsa_path) = resolve_test_tsgo_binary() else {
        return;
    };

    let project_root = unique_case_dir("corsa-bridge-completion");
    let _ = std::fs::remove_dir_all(&project_root);
    let src_dir = project_root.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    if link_workspace_node_modules(&project_root).is_err() {
        return;
    }
    write_project_tsconfig(&project_root);

    // Virtual TS shape that the maestro completion path would feed to Corsa.
    let virtual_ts = "import { ref } from 'vue';\nconst count = ref(0);\ncount.value.\n";
    let virtual_path = src_dir.join("App.vue.ts");
    std::fs::write(&virtual_path, virtual_ts).unwrap();

    let bridge = CorsaBridge::with_config(CorsaBridgeConfig {
        corsa_path: Some(corsa_path),
        working_dir: Some(project_root.clone()),
        timeout_ms: 30_000,
        ..Default::default()
    });

    let labels: Option<Vec<std::string::String>> = block_on(async {
        if bridge.spawn().await.is_err() {
            return None;
        }
        let uri = virtual_path.display().to_string();
        if bridge
            .open_or_update_virtual_document(uri.as_str(), virtual_ts)
            .await
            .is_err()
        {
            let _ = bridge.shutdown().await;
            return None;
        }
        // Position of the caret right after the second `.` on line 2 (0-indexed):
        //   line 2: "count.value."
        //                       ^ character 12
        let items = bridge.completion(uri.as_str(), 2, 12).await.ok()?;
        let _ = bridge.shutdown().await;
        Some(items.into_iter().map(|item| item.label).collect())
    });

    let _ = std::fs::remove_dir_all(&project_root);

    let Some(labels) = labels else {
        // Bridge or session failed to start in this environment.
        // The test already exits before this point when the runtime is missing.
        return;
    };

    assert!(
        labels.iter().any(|label| label == "toFixed"),
        "expected `toFixed` in Corsa completion labels for `count.value.`, got: {labels:?}"
    );
    assert!(
        labels.iter().any(|label| label == "toString"),
        "expected `toString` in Corsa completion labels for `count.value.`, got: {labels:?}"
    );
}

#[test]
fn batch_type_checker_accepts_template_ref_unwrap_and_array_access() {
    let project_root = unique_case_dir("template-ref");
    let _ = std::fs::remove_dir_all(&project_root);
    let src_dir = project_root.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    if link_workspace_node_modules(&project_root).is_err() {
        return;
    }
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["App.virtual.ts"]
}"#,
    )
    .unwrap();
    std::fs::write(
        src_dir.join("App.vue"),
        r#"<script setup lang="ts">
import { ref, useTemplateRef } from 'vue'

const users = ref([{ id: 1 }])
const inputRef = useTemplateRef<HTMLInputElement>('input')
</script>

<template>
  <div>{{ users.length }} {{ inputRef && inputRef.focus() }}</div>
</template>
"#,
    )
    .unwrap();

    let mut checker = match BatchTypeChecker::new(&project_root) {
        Ok(checker) => checker,
        Err(_) => return,
    };
    checker.scan_project().unwrap();

    let result = match checker.check_project() {
        Ok(result) => result,
        Err(_) => return,
    };

    let relevant: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|diagnostic| matches!(diagnostic.code, Some(2339) | Some(2349)))
        .map(|diagnostic| {
            (
                relative_path(&project_root, &diagnostic.file),
                diagnostic.code,
                diagnostic.line,
                diagnostic.column,
                diagnostic.message.clone(),
                diagnostic.block_type,
            )
        })
        .collect();

    assert!(
        relevant.is_empty(),
        "unexpected template unwrap diagnostics: {relevant:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn batch_type_checker_accepts_setup_binding_named_like_instance_global() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let project_root = create_project_case_without_node_modules(
        "setup-binding-instance-global",
        &[(
            "src/App.vue",
            r#"<template>
  <div v-if="$q">
    None
  </div>
</template>

<script setup lang="ts">
function functionCall(): any {}

const $q = functionCall()
</script>
"#,
        )],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(
        snapshot
            .iter()
            .all(|(file, code, _)| { file != "src/App.vue" || *code != Some(2300) }),
        "unexpected duplicate identifier diagnostic for setup $ binding: {snapshot:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn batch_type_checker_reports_original_key_for_renamed_props_destructure() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let project_root = create_project_case_without_node_modules(
        "renamed-props-destructure-original-key",
        &[(
            "src/App.vue",
            r#"<script setup lang="ts">
const { foo: bar } = defineProps<{ foo: string }>()
void bar
</script>

<template>{{ foo }} {{ bar }}</template>
"#,
        )],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(
        snapshot.iter().any(|(file, code, message)| {
            file == "src/App.vue" && *code == Some(2304) && message.contains("foo")
        }),
        "expected original prop key to report TS2304, got: {snapshot:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn batch_type_checker_reports_nested_interface_member_as_unknown_template_name() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let project_root = create_project_case_without_node_modules(
        "nested-interface-member-template-name",
        &[(
            "src/App.vue",
            r#"<script setup lang="ts">
interface Props {
  config: {
    inner: string
  }
  name: string
}

defineProps<Props>()
</script>

<template>{{ config.inner }} {{ inner }} {{ name }}</template>
"#,
        )],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(
        snapshot.iter().any(|(file, code, message)| {
            file == "src/App.vue" && *code == Some(2304) && message.contains("inner")
        }),
        "expected nested object member to report TS2304, got: {snapshot:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn batch_type_checker_reports_user_ts6133_with_no_unused_locals() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let project_root = create_project_case_without_node_modules(
        "user-ts6133-no-unused-locals",
        &[(
            "src/App.vue",
            r#"<script setup lang="ts">
const used = 1
const unusedLocal = 2
</script>

<template>{{ used }}</template>
"#,
        )],
    );
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true,
    "noUnusedLocals": true
  },
  "include": ["src/**/*"]
}"#,
    )
    .unwrap();

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    let ts6133: Vec<_> = snapshot
        .iter()
        .filter(|(file, code, _)| file == "src/App.vue" && *code == Some(6133))
        .collect();

    assert_eq!(
        ts6133.len(),
        1,
        "expected one user TS6133, got: {snapshot:#?}"
    );
    assert!(
        ts6133[0].2.contains("unusedLocal"),
        "expected unusedLocal diagnostic, got: {snapshot:#?}"
    );
    assert!(
        snapshot
            .iter()
            .all(|(_, _, message)| !message.contains("defineProps")),
        "generated helper diagnostics should stay suppressed: {snapshot:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn batch_type_checker_accepts_nested_ref_value_component_props() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let project_root = create_project_case(
        "nested-ref-value-props",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts">
defineProps<{
  count: number
}>()
</script>

<template>
  <div>{{ count }}</div>
</template>
"#,
            ),
            (
                "src/App.vue",
                r#"<script setup lang="ts">
import { ref } from 'vue'
import Child from './Child.vue'

const state = ref({
  nested: ref(1),
})
</script>

<template>
  <Child :count="state.nested.value" />
</template>
"#,
            ),
        ],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    let relevant: Vec<_> = snapshot
        .iter()
        .filter(|(file, code, _)| {
            file == "src/App.vue" && matches!(*code, Some(18048) | Some(2322) | Some(2339))
        })
        .cloned()
        .collect();

    assert!(
        relevant.is_empty(),
        "unexpected nested ref prop diagnostics: {relevant:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn batch_type_checker_reports_template_handler_mismatches_without_node_modules() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let project_root = create_project_case_without_node_modules(
        "template-handler-mismatches",
        &[
            (
                "src/InlineHandlerError.vue",
                r#"<script setup lang="ts">
import { ref } from 'vue'

const count = ref(0)

function processString(value: string): void {
  console.log(value)
}
</script>

<template>
  <button @click="processString(count)">Click</button>
</template>
"#,
            ),
            (
                "src/WrongEventHandler.vue",
                r#"<script setup lang="ts">
function handleName(name: string): void {
  console.log(name)
}
</script>

<template>
  <button @click="handleName">Click me</button>
</template>
"#,
            ),
            (
                "src/ComputedMemberHandler.vue",
                r#"<script setup lang="ts">
const handlers = { x: 42 } as const
</script>

<template>
  <button @click="handlers['x']">Click me</button>
</template>
"#,
            ),
            (
                "src/ZeroArgButton.vue",
                r#"<script setup lang="ts">
const emit = defineEmits<{
  click: []
}>()
</script>

<template>
  <button @click="emit('click')">Toggle</button>
</template>
"#,
            ),
            (
                "src/ClickPointerEvent.vue",
                r#"<script setup lang="ts">
const emit = defineEmits<{
  (event: 'click', payload: PointerEvent): void
}>()
</script>

<template>
  <button @click="emit('click', $event)">Click</button>
</template>
"#,
            ),
            (
                "src/InlineArrowHandler.vue",
                r#"<script setup lang="ts">
</script>

<template>
  <button @click="(payload) => payload.preventDefault()">Click</button>
</template>
"#,
            ),
            (
                "src/ComponentZeroArgHandler.vue",
                r#"<script setup lang="ts">
import ZeroArgButton from './ZeroArgButton.vue'

function toggle(open = false): void {
  console.log(open)
}
</script>

<template>
  <ZeroArgButton @click="toggle" />
</template>
"#,
            ),
        ],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(
        snapshot
            .iter()
            .any(|(file, code, _)| { file == "src/InlineHandlerError.vue" && *code == Some(2345) }),
        "expected InlineHandlerError.vue to report TS2345, got: {snapshot:#?}"
    );
    assert!(
        snapshot.iter().any(|(file, code, message)| {
            file == "src/WrongEventHandler.vue"
                && *code == Some(2345)
                && message.contains("PointerEvent")
        }),
        "expected WrongEventHandler.vue to report PointerEvent mismatch, got: {snapshot:#?}"
    );
    assert!(
        snapshot.iter().any(|(file, code, _)| {
            file == "src/ComputedMemberHandler.vue" && *code == Some(2345)
        }),
        "expected ComputedMemberHandler.vue to report TS2345, got: {snapshot:#?}"
    );
    assert!(
        snapshot
            .iter()
            .all(|(file, code, _)| { file != "src/ClickPointerEvent.vue" || *code != Some(2740) }),
        "unexpected PointerEvent payload mismatch: {snapshot:#?}"
    );
    assert!(
        snapshot
            .iter()
            .all(|(file, code, _)| { file != "src/InlineArrowHandler.vue" || *code != Some(7006) }),
        "unexpected implicit any in inline arrow event handler: {snapshot:#?}"
    );
    assert!(
        snapshot.iter().all(|(file, code, _)| {
            file != "src/ComponentZeroArgHandler.vue" || *code != Some(2345)
        }),
        "unexpected zero-arg component event handler mismatch: {snapshot:#?}"
    );
    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn batch_type_checker_quirks_accepts_fallthrough_component_dom_event_handler() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "component-fallthrough-dom-event-handler",
        &[
            (
                "src/Child.vue",
                r#"<template>
  <input />
</template>
"#,
            ),
            (
                "src/App.vue",
                r#"<script setup lang="ts">
import Child from './Child.vue'

function eventHandler(event: Event) {
  void event
}
</script>

<template>
  <Child @keydown="eventHandler" />
</template>
"#,
            ),
        ],
    );

    let mut checker = match BatchTypeChecker::new(&project_root) {
        Ok(checker) => checker,
        Err(_) => {
            let _ = std::fs::remove_dir_all(&project_root);
            return;
        }
    };
    checker.set_template_syntax(vize_atelier_core::TemplateSyntaxMode::Quirks);
    checker.scan_project().unwrap();

    let result = match checker.check_project() {
        Ok(result) => result,
        Err(_) => {
            let _ = std::fs::remove_dir_all(&project_root);
            return;
        }
    };
    let relevant: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            relative_path(&project_root, &diagnostic.file) == "src/App.vue"
                && diagnostic.code == Some(2345)
        })
        .map(|diagnostic| diagnostic.message.clone())
        .collect();

    assert!(
        relevant.is_empty(),
        "fallthrough DOM event handlers should not report TS2345 in quirks mode: {relevant:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn batch_type_checker_multiline_inline_handler_points_into_offending_line() {
    // Regression: the inline-callback @event path used the directive span
    // (covering `@click="..."`) as the source-map src_range while emitting
    // only the value into virtual TS. The size mismatch made diagnostic
    // columns drift left — multi-line handler errors clamped to the line
    // indent instead of the failing statement.
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let project_root = create_project_case_without_node_modules(
        "multiline-handler-mapping",
        &[(
            "src/MultilineHandler.vue",
            r#"<script setup lang="ts">
function doA(): number { return 1 }
function doB(): string { return 'x' }
</script>

<template>
  <button @click="() => {
    doA();
    doB();
    const z: number = doB()
  }">click</button>
</template>
"#,
        )],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    let mismatch = snapshot
        .iter()
        .find(|(file, code, _)| file == "src/MultilineHandler.vue" && *code == Some(2322));
    let Some((_, _, message)) = mismatch else {
        let _ = std::fs::remove_dir_all(&project_root);
        panic!("expected MultilineHandler.vue TS2322 diagnostic, got: {snapshot:#?}");
    };

    let prefix = message.split(' ').next().unwrap_or("");
    let mut parts = prefix.split(':');
    let line: u32 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let column: u32 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);

    let expected_line = 10;
    assert_eq!(
        line, expected_line,
        "expected diagnostic on line {expected_line} of SFC, got line {line}; full: {message}"
    );
    assert!(
        column > 4,
        "expected diagnostic column past the 4-space indent on line {expected_line}, got col {column}; full: {message}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn batch_type_checker_multiline_statement_handler_does_not_parse_error() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let project_root = create_project_case_without_node_modules(
        "multiline-statement-handler",
        &[(
            "src/StatementHandler.vue",
            r#"<script setup lang="ts">
const keys = ['a']
function selectWord(key: string) {}
function editWord() {}
</script>

<template>
  <button
    v-for="key in keys"
    @click.stop="
      selectWord(key);
      editWord();
    "
  >edit</button>
</template>
"#,
        )],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(
        snapshot
            .iter()
            .all(|(file, code, _)| { file != "src/StatementHandler.vue" || *code != Some(1005) }),
        "unexpected TS1005 parse diagnostic for statement-list handler: {snapshot:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn batch_type_checker_narrows_same_element_event_handler_with_v_if() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let project_root = create_project_case_without_node_modules(
        "same-element-vif-handler-narrowing",
        &[(
            "src/SameElementVifHandler.vue",
            r#"<script setup lang="ts">
type UnionType = { type: "a" } | { type: "b", bSpecific: () => void }

const val = 0 as unknown as UnionType
</script>

<template>
  <div v-if="val.type === 'b'" @click="val.bSpecific"></div>
</template>
"#,
        )],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(
        snapshot.iter().all(|(file, code, _)| {
            file != "src/SameElementVifHandler.vue" || *code != Some(2339)
        }),
        "unexpected union member diagnostic for v-if narrowed event handler: {snapshot:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn batch_type_checker_narrows_v_for_source_by_enclosing_v_if() {
    // Regression for #1511: a v-for whose source depends on a value narrowed by
    // an *enclosing* v-if must type-check without a spurious diagnostic — the
    // iterated element type follows the narrowed source (`elems['b']` is
    // `string[]`, so `value` is `string`).
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let project_root = create_project_case_without_node_modules(
        "vfor-source-narrowed-by-enclosing-vif",
        &[(
            "src/VForNarrowedByVif.vue",
            r#"<script setup lang="ts">
const elems = { a: [1], b: ["a"] } as const;
const key = "a" as "a" | "b";
function wantsString(value: string) { void value; }
</script>

<template>
  <div v-if="key === 'b'">
    <button v-for="value in elems[key]" :key="value">
      {{ wantsString(value) }}
    </button>
  </div>
</template>
"#,
        )],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(
        snapshot
            .iter()
            .all(|(file, _, _)| file != "src/VForNarrowedByVif.vue"),
        "expected no diagnostics for v-for narrowed by enclosing v-if, got: {snapshot:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn batch_type_checker_keeps_v_for_error_without_enclosing_v_if() {
    // Companion guard for #1511: when there is NO enclosing v-if to narrow the
    // source, the unnarrowed union element type must still produce a diagnostic
    // (we must not over-narrow). `elems[key]` with `key: "a" | "b"` yields a
    // `(string | number)` element, so `wantsString(value)` is a real error.
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let project_root = create_project_case_without_node_modules(
        "vfor-source-not-narrowed-without-vif",
        &[(
            "src/VForNotNarrowed.vue",
            r#"<script setup lang="ts">
const elems = { a: [1], b: ["a"] } as const;
const key = "a" as "a" | "b";
function wantsString(value: string) { void value; }
</script>

<template>
  <button v-for="value in elems[key]" :key="value">
    {{ wantsString(value) }}
  </button>
</template>
"#,
        )],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(
        snapshot
            .iter()
            .any(|(file, code, _)| { file == "src/VForNotNarrowed.vue" && *code == Some(2345) }),
        "expected TS2345 for un-narrowed v-for source, got: {snapshot:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn batch_type_checker_uses_workspace_vue_runtime_without_node_modules() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let project_root = create_project_case_without_node_modules(
        "vue-runtime-stub",
        &[(
            "src/UseTemplateRefError.vue",
            r#"<script setup lang="ts">
import { ref, onMounted } from 'vue'

const inputRef = useTemplateRef<HTMLInputElement>('input')
const count = ref(0)

onMounted(() => {
  if (inputRef.value) {
    const num: number = inputRef.value.value
    inputRef.value.nonExistentMethod()
  }
})
</script>

<template>
  <input ref="input" />
  <span>{{ count }}</span>
</template>
"#,
        )],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(
        snapshot.iter().all(
            |(_, code, message)| *code != Some(2305) && !message.contains("no exported member")
        ),
        "unexpected bundled vue runtime export diagnostic: {snapshot:#?}"
    );
    assert!(
        snapshot.iter().any(|(file, code, _)| {
            file == "src/UseTemplateRefError.vue" && *code == Some(2322)
        }),
        "expected template ref value mismatch to remain reported, got: {snapshot:#?}"
    );
    assert!(
        snapshot.iter().any(|(file, code, _)| {
            file == "src/UseTemplateRefError.vue" && *code == Some(2339)
        }),
        "expected template ref method mismatch to remain reported, got: {snapshot:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn batch_type_checker_expands_vue_runtime_stubs_without_node_modules() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    with_workspace_node_modules_override(Some("__none__"), || {
        let project_root = create_project_case_without_node_modules(
            "vue-runtime-stub-expanded",
            &[(
                "src/FallbackVueRuntime.vue",
                r#"<script setup lang="ts">
import {
  computed,
  defineAsyncComponent,
  inject,
  markRaw,
  provide,
  reactive,
  type InjectionKey,
} from 'vue'

const countKey: InjectionKey<number> = Symbol('count')
provide(countKey, 1)

const count = inject(countKey, 0)
const state = reactive({ count })
const doubled = computed(() => state.count * 2)
const LazyPanel = markRaw(defineAsyncComponent(() => Promise.resolve({})))

const badCount: string = count
const badDoubled: string = doubled.value

void LazyPanel
</script>

<template>{{ doubled }}</template>
"#,
            )],
        );

        let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
            let _ = std::fs::remove_dir_all(&project_root);
            return;
        };

        assert!(
            snapshot
                .iter()
                .all(|(_, code, message)| *code != Some(2305)
                    && !message.contains("no exported member")),
            "unexpected bundled vue runtime export diagnostic: {snapshot:#?}"
        );
        assert!(
            snapshot
                .iter()
                .filter(|(file, code, _)| {
                    file == "src/FallbackVueRuntime.vue" && *code == Some(2322)
                })
                .count()
                >= 2,
            "expected typed fallback mismatches to remain reported, got: {snapshot:#?}"
        );

        let _ = std::fs::remove_dir_all(&project_root);
    });
}

/// Plain Options API component used by the `defineComponent`-wrap regression
/// tests: `this.count` (a `data` field) is accessed from a computed. Without
/// the wrap, `this` binds to the `computed` sub-object literal and TypeScript
/// reports a TS2339 false positive.
const OPTIONS_API_THIS_IN_COMPUTED_SFC: &str = r#"<script lang="ts">
export default {
  data() {
    return { count: 0 }
  },
  computed: {
    doubled(): number {
      return this.count * 2
    },
  },
}
</script>

<template>
  <div>static</div>
</template>
"#;

#[test]
fn batch_type_checker_accepts_options_api_this_data_access_in_computed() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let project_root = create_project_case(
        "options-api-this-computed",
        &[("src/App.vue", OPTIONS_API_THIS_IN_COMPUTED_SFC)],
    );

    // Real instance typing requires the real vue package (full
    // `defineComponent` overloads with `ThisType`). When the workspace only
    // provides the erased test stub (`defineComponent(options: any)`), skip —
    // stub behavior is covered by the facade-fallback test below.
    if !project_root.join("node_modules/vue/dist").exists() {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    }

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(
        snapshot.is_empty(),
        "expected `this.<dataField>` in a computed of a plain options object to check clean: {snapshot:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn batch_type_checker_reports_options_api_template_type_mismatch() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let project_root = create_project_case(
        "options-api-template-binding-type",
        &[(
            "src/App.vue",
            r#"<script lang="ts">
export default {
  data() {
    return { count: 0 }
  },
}
</script>

<template>
  <div>{{ count.toFixed(true) }}</div>
</template>
"#,
        )],
    );

    if !project_root.join("node_modules/vue/dist").exists() {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    }

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(
        snapshot
            .iter()
            .any(|(file, code, _)| { file == "src/App.vue" && *code == Some(2345) }),
        "expected Options API data binding to keep its number type in the template: {snapshot:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn batch_type_checker_options_api_wrap_adds_no_errors_in_facade_fallback() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    // Facade-fallback mode: no resolvable vue package, so the bundled vue
    // stub is materialized. Its `defineComponent(options: any)` signature
    // erases `ThisType`, so the pre-existing TS2339 false positive on
    // `this.count` remains until the stub gains a ThisType-aware signature
    // (issue #1388 PR6). The wrap must not introduce any NEW diagnostics
    // beyond that documented one.
    with_workspace_node_modules_override(Some("__none__"), || {
        let project_root = create_project_case_without_node_modules(
            "options-api-wrap-facade-fallback",
            &[("src/App.vue", OPTIONS_API_THIS_IN_COMPUTED_SFC)],
        );

        let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
            let _ = std::fs::remove_dir_all(&project_root);
            return;
        };

        assert!(
            snapshot
                .iter()
                .all(|(file, code, _)| file == "src/App.vue" && *code == Some(2339)),
            "expected the defineComponent wrap to add no new diagnostics in facade-fallback mode: {snapshot:#?}"
        );

        let _ = std::fs::remove_dir_all(&project_root);
    });
}

#[test]
fn batch_type_checker_snapshots_cross_file_vue_prop_error() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "cross-file-vue-props",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts">
defineProps<{
  count: number
}>()
</script>

<template>
  <div>{{ count }}</div>
</template>
"#,
            ),
            (
                "src/Parent.vue",
                r#"<script setup lang="ts">
import Child from './Child.vue'
</script>

<template>
  <Child :count="'oops'" />
</template>
"#,
            ),
        ],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    insta::with_settings!({
        snapshot_path => "../../snapshots"
    }, {
        insta::assert_debug_snapshot!("batch_type_checker_cross_file_vue_prop_error", snapshot);
    });

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn batch_type_checker_accepts_forwarded_optional_component_props() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "optional-component-props",
        &[
            (
                "src/Provider.vue",
                r#"<script lang="ts">
export type LinkBehavior = "window" | "browser" | null;
</script>

<script setup lang="ts">
defineProps<{
  behavior?: LinkBehavior;
}>();
</script>

<template>
  <a><slot /></a>
</template>
"#,
            ),
            (
                "src/Consumer.vue",
                r#"<script setup lang="ts">
import Provider from "./Provider.vue";
import type { LinkBehavior } from "./Provider.vue";

defineProps<{
  behavior?: LinkBehavior;
}>();
</script>

<template>
  <Provider :behavior="behavior" />
</template>
"#,
            ),
        ],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(
        snapshot.is_empty(),
        "forwarded optional component prop should type-check, got: {snapshot:?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn batch_type_checker_snapshots_generic_component_prop_error() {
    // #775: a wrongly-typed prop passed to a `<script setup generic="T">` child
    // must raise TS2322. The child's construct-signature `$props` collapses
    // `T` to its constraint, so the parent infers `T` across the boundary by
    // calling the child's `__vizeCheck<T>(props)` from its default export.
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "generic-component-props",
        &[
            (
                "src/GenericList.vue",
                r#"<script setup lang="ts" generic="T">
defineProps<{
  items: T[]
  selected: T
}>()
</script>

<template>
  <div>{{ selected }}</div>
</template>
"#,
            ),
            (
                "src/Parent.vue",
                r#"<script setup lang="ts">
import GenericList from './GenericList.vue'
</script>

<template>
  <GenericList :items="['a', 'b']" :selected="42" />
</template>
"#,
            ),
        ],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    insta::with_settings!({
        snapshot_path => "../../snapshots"
    }, {
        insta::assert_debug_snapshot!("batch_type_checker_generic_component_prop_error", snapshot);
    });

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn batch_type_checker_accepts_well_typed_generic_component_props() {
    // The dual of the test above: a correctly-typed generic prop must NOT
    // report, and the new functional check must not introduce spurious
    // diagnostics for the non-error case.
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "generic-component-props-ok",
        &[
            (
                "src/GenericList.vue",
                r#"<script setup lang="ts" generic="T">
defineProps<{
  items: T[]
  selected: T
}>()
</script>

<template>
  <div>{{ selected }}</div>
</template>
"#,
            ),
            (
                "src/Parent.vue",
                r#"<script setup lang="ts">
import GenericList from './GenericList.vue'
</script>

<template>
  <GenericList :items="['a', 'b']" :selected="'a'" />
</template>
"#,
            ),
        ],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(
        snapshot.is_empty(),
        "well-typed generic component props should not report diagnostics, got: {snapshot:?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn batch_type_checker_accepts_imported_intersection_template_props() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "imported-intersection-template-props",
        &[
            (
                "src/imported-options.ts",
                r#"export type PaginationOptions = {
  direction?: 'up' | 'down'
  autoLoad?: boolean
}
"#,
            ),
            (
                "src/ImportedIntersectionProps.vue",
                r#"<template>
  <div>{{ item.id }} {{ direction }} {{ autoLoad }}</div>
</template>

<script setup lang="ts" generic="T extends { id: string }">
import type { PaginationOptions } from './imported-options'

const props = withDefaults(defineProps<PaginationOptions & {
  item: T
}>(), {
  autoLoad: true,
  direction: 'down',
})

void props
</script>
"#,
            ),
        ],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(
        snapshot.is_empty(),
        "imported intersection props should be exposed to templates, got: {snapshot:?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn batch_type_checker_accepts_with_defaults_direct_template_prop_identifiers() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "with-defaults-direct-template-props",
        &[(
            "src/CounterButton.vue",
            r#"<script setup lang="ts">
const props = withDefaults(
  defineProps<{
    count?: number;
    label: string;
  }>(),
  { count: 0 },
);

const emit = defineEmits<{
  increment: [value: number];
}>();

void props;
</script>

<template>
  <button type="button" @click="emit('increment', count + 1)">
    {{ label }}: {{ count }}
  </button>
</template>
"#,
        )],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(
        snapshot.is_empty(),
        "defaulted direct template prop identifiers should not report diagnostics, got: {snapshot:?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn batch_type_checker_accepts_dynamic_runtime_emits() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "dynamic-runtime-emits",
        &[
            (
                "src/emits.ts",
                r#"export const dialogEmits = ['ok', 'hide'] as const;

export const emitObject = {
  ok: (payload: string) => payload.length > 0,
  hide: () => true,
} as const;
"#,
            ),
            (
                "src/DynamicArrayDialog.vue",
                r#"<script setup lang="ts">
import { dialogEmits } from './emits';

const emit = defineEmits([...dialogEmits]);
</script>

<template>
  <button type="button" @click="emit('ok')">OK</button>
  <button type="button" @click="emit('hide')">Hide</button>
</template>
"#,
            ),
            (
                "src/DynamicObjectDialog.vue",
                r#"<script setup lang="ts">
import { emitObject } from './emits';

const emit = defineEmits({ ...emitObject });

function submit() {
  emit('ok', 'saved');
}
</script>

<template>
  <button type="button" @click="submit">OK</button>
</template>
"#,
            ),
            (
                "src/App.vue",
                r#"<script setup lang="ts">
import DynamicArrayDialog from './DynamicArrayDialog.vue';
import DynamicObjectDialog from './DynamicObjectDialog.vue';

function handleOk() {}
function handleHide() {}
function handlePayload(payload: string) {
  payload.toUpperCase();
}
</script>

<template>
  <DynamicArrayDialog @ok="handleOk" @hide="handleHide" />
  <DynamicObjectDialog @ok="handlePayload" />
</template>
"#,
            ),
        ],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(
        snapshot.is_empty(),
        "dynamic runtime emits should be inferred without diagnostics, got: {snapshot:?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn batch_type_checker_accepts_reexported_vue_interface_template_props() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "reexported-vue-interface-template-props",
        &[
            (
                "src/Base.vue",
                r#"<script lang="ts">
export interface BaseProps {
  as?: string
  asChild?: boolean
}
</script>

<template><div></div></template>
"#,
            ),
            ("src/index.ts", r#"export * from "./Base.vue";"#),
            (
                "src/Child.vue",
                r#"<script setup lang="ts">
defineProps<{
  as?: string
  asChild?: boolean
}>()
</script>

<template><div></div></template>
"#,
            ),
            (
                "src/ParentWidget.vue",
                r#"<script lang="ts">
import type { BaseProps } from './index'

export interface ParentWidgetProps extends BaseProps {}
</script>

<script setup lang="ts">
import Child from './Child.vue'

const props = defineProps<ParentWidgetProps>()
</script>

<template>
  <Child
    :as="as"
    :as-child="props.asChild"
  />
</template>
"#,
            ),
        ],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(
        snapshot.is_empty(),
        "re-exported Vue interface props should resolve in Corsa diagnostics, got: {snapshot:?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn batch_type_checker_accepts_mixed_reexported_vue_interface_template_props() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "mixed-reexported-vue-interface-template-props",
        &[
            (
                "src/primitive.ts",
                r#"export type AsTag = 'div' | 'span' | ({} & string)

export interface PrimitiveProps {
  asChild?: boolean
  as?: AsTag
}
"#,
            ),
            (
                "src/content/Content.vue",
                r#"<script lang="ts">
import type { PrimitiveProps } from '../primitive'

export interface ContentProps extends PrimitiveProps {
  forceMount?: boolean
}
</script>

<script setup lang="ts">
defineProps<ContentProps>()
</script>

<template><div></div></template>
"#,
            ),
            (
                "src/content/index.ts",
                r#"export {
  default as Content,
  type ContentProps,
} from './Content.vue'
"#,
            ),
            (
                "src/Wrapper.vue",
                r#"<script lang="ts">
import type { ContentProps } from './content'

export interface WrapperProps extends ContentProps {}
</script>

<script setup lang="ts">
import { Content } from './content'

const props = defineProps<WrapperProps>()
</script>

<template>
  <Content
    :as-child="props.asChild"
    :as="as"
    :force-mount="props.forceMount"
  />
</template>
"#,
            ),
        ],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(
        snapshot.is_empty(),
        "mixed Vue type re-exports should resolve in Corsa diagnostics, got: {snapshot:?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn batch_type_checker_snapshots_ts_imports_vue_component() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "ts-imports-vue",
        &[
            (
                "src/App.vue",
                r#"<script setup lang="ts">
defineProps<{
  count: number
}>()
</script>

<template>
  <div>{{ count }}</div>
</template>
"#,
            ),
            (
                "src/main.ts",
                r#"import App from './App.vue'

type AppProps = InstanceType<typeof App>['$props']

const props: AppProps = {
  count: 'oops',
}

void props
"#,
            ),
        ],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    insta::with_settings!({
        snapshot_path => "../../snapshots"
    }, {
        insta::assert_debug_snapshot!("batch_type_checker_ts_imports_vue_component", snapshot);
    });

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn batch_type_checker_snapshots_ambient_dts_global_usage() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "ambient-dts",
        &[
            ("src/env.d.ts", r#"declare const APP_VERSION: string;"#),
            (
                "src/App.vue",
                r#"<template>
  <div>{{ APP_VERSION.toFixed(2) }}</div>
</template>
"#,
            ),
        ],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    insta::with_settings!({
        snapshot_path => "../../snapshots"
    }, {
        insta::assert_debug_snapshot!("batch_type_checker_ambient_dts_global_usage", snapshot);
    });

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn batch_type_checker_snapshots_declaration_emit_outputs() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "declaration-emit",
        &[
            (
                "src/App.vue",
                r#"<script setup lang="ts">
export interface PublicProps {
  count: number
}

const props = defineProps<PublicProps>()
</script>

<template>
  <div>{{ props.count }}</div>
</template>
"#,
            ),
            (
                "src/index.ts",
                r#"export { default as App } from './App.vue'
"#,
            ),
        ],
    );

    let mut checker = match BatchTypeChecker::new(&project_root) {
        Ok(checker) => checker,
        Err(_) => return,
    };
    checker.scan_project().unwrap();
    let out_dir = project_root.join("types");
    let emitted = checker
        .emit_declarations(&DeclarationEmitOptions::new(out_dir.clone()))
        .unwrap();
    let snapshot: Vec<_> = emitted
        .files
        .into_iter()
        .map(|file| (relative_path(&out_dir, &file.path), file.content))
        .collect();

    insta::with_settings!({
        snapshot_path => "../../snapshots"
    }, {
        insta::assert_debug_snapshot!("batch_type_checker_declaration_emit_outputs", snapshot);
    });

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn batch_type_checker_virtual_vue_component_satisfies_storybook_like_meta() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "virtual-vue-component-storybook-meta",
        &[
            (
                "src/App.vue",
                r#"<script setup lang="ts">
defineProps<{
  title: string
}>()
</script>

<template>
  <h1>{{ title }}</h1>
</template>
"#,
            ),
            (
                "src/story.ts",
                r#"import App from './App.vue'

type StorybookLikeMeta<C extends import('vue').ConcreteComponent> = {
  component: C
}

const meta = {
  component: App,
} satisfies StorybookLikeMeta<typeof App>

void meta
"#,
            ),
        ],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };
    assert!(
        snapshot.is_empty(),
        "virtual Vue components should satisfy Storybook-like component constraints, got: {snapshot:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn batch_type_checker_declaration_emit_keeps_paths_alias_imports_in_virtual_project() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "declaration-path-alias",
        &[
            (
                "src/App.vue",
                r#"<script setup lang="ts">
import { answer } from '@/helper'

const value = answer
</script>

<template>
  <div>{{ value }}</div>
</template>
"#,
            ),
            ("src/helper.ts", "export const answer = 42;\n"),
            (
                "src/index.ts",
                r#"export { default as App } from './App.vue'
export { answer } from '@/helper'
"#,
            ),
        ],
    );
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "baseUrl": ".",
    "paths": {
      "@/*": ["src/*"]
    },
    "noEmit": true
  },
  "include": ["src/**/*.ts", "src/**/*.vue"]
}"#,
    )
    .unwrap();

    let mut checker = match BatchTypeChecker::new(&project_root) {
        Ok(checker) => checker,
        Err(_) => return,
    };
    checker.scan_project().unwrap();
    let out_dir = project_root.join("types");
    let emitted = checker
        .emit_declarations(&DeclarationEmitOptions::new(out_dir.clone()))
        .unwrap();
    let mut paths: Vec<_> = emitted
        .files
        .into_iter()
        .map(|file| relative_path(&out_dir, &file.path))
        .collect();
    paths.sort();

    // `__vize_helpers.d.ts` carries the shared helper type aliases referenced
    // by the emitted `.vue.d.ts` files (via `/// <reference path>`).
    assert_eq!(
        paths,
        vec![
            "App.vue.d.ts",
            "__vize_helpers.d.ts",
            "helper.d.ts",
            "index.d.ts"
        ]
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

fn relative_path(root: &std::path::Path, file: &std::path::Path) -> String {
    file.strip_prefix(root)
        .map(|path| cstr!("{}", path.display()))
        .unwrap_or_else(|_| cstr!("{}", file.display()))
}

fn unique_case_dir(name: &str) -> PathBuf {
    static NEXT_CASE_ID: AtomicUsize = AtomicUsize::new(0);

    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist");
    let case_id = NEXT_CASE_ID.fetch_add(1, Ordering::Relaxed);
    workspace_root
        .join("target")
        .join("vize-tests")
        .join("tests")
        .join(cstr!("{name}-{}-{case_id}", std::process::id()).as_str())
}

fn create_project_case(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let project_root = unique_case_dir(name);
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(&project_root).unwrap();
    link_workspace_node_modules(&project_root).unwrap();
    write_project_tsconfig(&project_root);

    for (path, source) in files {
        let file_path = project_root.join(path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(file_path, source).unwrap();
    }

    project_root
}

fn create_project_case_without_node_modules(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let project_root = unique_case_dir(name);
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(&project_root).unwrap();
    write_project_tsconfig(&project_root);

    for (path, source) in files {
        let file_path = project_root.join(path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(file_path, source).unwrap();
    }

    project_root
}

fn write_project_tsconfig(project_root: &Path) {
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
    )
    .unwrap();
}

fn snapshot_project_diagnostics(project_root: &Path) -> Option<Vec<(String, Option<u32>, String)>> {
    let mut checker = BatchTypeChecker::new(project_root).ok()?;
    checker.scan_project().ok()?;
    let result = checker.check_project().ok()?;

    let mut snapshot: Vec<_> = result
        .diagnostics
        .into_iter()
        .map(|diagnostic| {
            (
                relative_path(project_root, &diagnostic.file),
                diagnostic.code,
                cstr!(
                    "{}:{}:{} {}",
                    diagnostic.line + 1,
                    diagnostic.column + 1,
                    match diagnostic.severity {
                        1 => "error",
                        2 => "warning",
                        3 => "info",
                        _ => "hint",
                    },
                    diagnostic.message
                ),
            )
        })
        .collect();
    snapshot.sort();
    Some(snapshot)
}

fn corsa_type_mismatch_snapshot(
    file_text: &str,
    declaration_marker: &str,
    initializer_marker: &str,
) -> Vec<(std::string::String, std::string::String)> {
    static NEXT_CASE_ID: AtomicUsize = AtomicUsize::new(0);

    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist");
    let case_id = NEXT_CASE_ID.fetch_add(1, Ordering::Relaxed);
    let project_root = workspace_root
        .join("target")
        .join("vize-tests")
        .join("tests")
        .join(format!("corsa-type-probe-{}-{case_id}", std::process::id()));
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(&project_root).expect("project root should exist");
    let src_dir = project_root.join("src");
    std::fs::create_dir_all(&src_dir).expect("src dir should exist");
    link_workspace_node_modules(&project_root).expect("workspace node_modules should link");
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["src/**/*.ts", "src/**/*.vue"]
}"#,
    )
    .expect("tsconfig should write");
    let file = src_dir.join("App.virtual.ts");
    std::fs::write(&file, file_text).expect("virtual ts should write");

    let corsa_path =
        resolve_test_tsgo_binary().expect("tsgo executable should resolve for corsa api tests");
    let config_wire = project_root.join("tsconfig.json").display().to_string();
    let file_wire = file.display().to_string();
    let declaration_offset = file_text
        .find(declaration_marker)
        .expect("declaration marker should exist");
    let initializer_offset = file_text
        .find(initializer_marker)
        .map(|offset| offset + initializer_marker.len().saturating_sub(1))
        .expect("initializer marker should exist");

    let result = block_on(async {
        let session = ProjectSession::spawn(
            ApiSpawnConfig::new(corsa_path)
                .with_mode(ApiMode::AsyncJsonRpcStdio)
                .with_cwd(project_root.as_path()),
            config_wire,
            None,
        )
        .await
        .expect("corsa project session should initialize");
        assert!(
            session
                .project()
                .root_files
                .iter()
                .any(|file| file.ends_with("App.virtual.ts")),
            "root files did not include App.virtual.ts: {:?}",
            session.project().root_files
        );
        let declaration = session
            .get_type_at_position(file_wire.as_str(), declaration_offset as u32)
            .await
            .expect("declaration type should load")
            .expect("declaration type should exist");
        let initializer = session
            .get_type_at_position(file_wire.as_str(), initializer_offset as u32)
            .await
            .expect("initializer type should load")
            .expect("initializer type should exist");
        let declaration_text = session
            .type_to_string(declaration.id, None, None)
            .await
            .expect("declaration type should render");
        let initializer_text = session
            .type_to_string(initializer.id, None, None)
            .await
            .expect("initializer type should render");
        session.close().await.expect("session should close");
        vec![
            ("declaration".into(), declaration_text),
            ("initializer".into(), initializer_text),
        ]
    });
    let _ = std::fs::remove_dir_all(&project_root);
    result
}

fn resolve_test_tsgo_binary() -> Option<PathBuf> {
    if std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some() {
        return None;
    }

    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)?;
    let sibling_cache = workspace_root.parent()?.join("corsa-bind/.cache/tsgo");
    if sibling_cache.exists() {
        return Some(sibling_cache);
    }

    vize_carton::corsa_resolver::discover_corsa_in_ancestors(workspace_root)
}

fn link_workspace_node_modules(project_root: &Path) -> std::io::Result<()> {
    let Some(workspace_root) = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
    else {
        return Err(std::io::Error::other("workspace root not found"));
    };
    let workspace_node_modules = resolve_workspace_node_modules(workspace_root);

    let target = project_root.join("node_modules");
    if target.is_symlink() || target.is_file() {
        std::fs::remove_file(&target)?;
    } else if target.exists() {
        std::fs::remove_dir_all(&target)?;
    }
    std::fs::create_dir_all(&target)?;

    if let Some(ref workspace_node_modules) = workspace_node_modules {
        link_or_stub_package(workspace_node_modules, &target, "vue", write_test_vue_stub)?;
        link_or_stub_package(
            workspace_node_modules,
            &target,
            "vite",
            write_test_vite_stub,
        )?;

        if let Some(vue_namespace) = resolve_test_vue_runtime_namespace(workspace_node_modules) {
            symlink_path(&vue_namespace, &target.join("@vue"))?;
        } else {
            write_test_vue_runtime_dom_stub(&target)?;
        }
    } else {
        write_test_vue_stub(&target)?;
        write_test_vite_stub(&target)?;
    }

    if let Some(source) = vize_carton::corsa_resolver::discover_corsa_in_ancestors(workspace_root)
        && source.exists()
    {
        let file_name = source.file_name().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "invalid corsa binary path",
            )
        })?;
        symlink_path(
            &source,
            &target
                .join("@typescript")
                .join("native-preview")
                .join("lib")
                .join(file_name),
        )?;
        symlink_path(&source, &target.join(".bin").join(file_name))?;
    }

    Ok(())
}

fn resolve_test_vue_runtime_namespace(workspace_node_modules: &Path) -> Option<PathBuf> {
    let vue_source = workspace_node_modules.join("vue");
    let adjacent = resolve_adjacent_vue_namespace(&vue_source);
    let ancestor = {
        let path = workspace_node_modules.join("@vue");
        path.exists().then_some(path)
    };

    adjacent
        .filter(|path| is_vue_runtime_namespace(path))
        .or_else(|| ancestor.filter(|path| is_vue_runtime_namespace(path)))
}

fn resolve_adjacent_vue_namespace(vue_source: &Path) -> Option<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(parent) = vue_source.parent() {
        candidates.push(parent.join("@vue"));
    }

    if let Ok(real_vue_source) = std::fs::canonicalize(vue_source)
        && let Some(parent) = real_vue_source.parent()
    {
        candidates.push(parent.join("@vue"));
    }

    candidates
        .into_iter()
        .find(|candidate| candidate.exists() && is_vue_runtime_namespace(candidate))
}

fn is_vue_runtime_namespace(path: &Path) -> bool {
    path.join("runtime-dom").exists() || path.join("runtime-core").exists()
}

fn link_or_stub_package(
    workspace_node_modules: &Path,
    target: &Path,
    package: &str,
    stub_writer: fn(&Path) -> std::io::Result<()>,
) -> std::io::Result<()> {
    let source = workspace_node_modules.join(package);
    if source.exists() {
        let link_source = package_link_source(&source, package);
        symlink_path(&link_source, &target.join(package))
    } else {
        stub_writer(target)
    }
}

fn package_link_source(source: &Path, package: &str) -> PathBuf {
    if package == "vue" {
        vize_carton::path::canonicalize_non_verbatim(source)
    } else {
        source.to_path_buf()
    }
}

fn resolve_workspace_node_modules(workspace_root: &Path) -> Option<PathBuf> {
    let override_path = std::env::var_os("VIZE_TEST_WORKSPACE_NODE_MODULES");
    if let Some(override_path) = override_path {
        let override_path = PathBuf::from(override_path);
        if override_path.as_os_str() == "__none__" {
            return None;
        }
        return override_path.exists().then_some(override_path);
    }

    let workspace_node_modules = workspace_root.join("node_modules");
    workspace_node_modules
        .exists()
        .then_some(workspace_node_modules)
}

fn with_workspace_node_modules_override<T>(value: Option<&str>, run: impl FnOnce() -> T) -> T {
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    struct EnvOverrideGuard {
        previous: Option<std::ffi::OsString>,
    }

    impl Drop for EnvOverrideGuard {
        fn drop(&mut self) {
            if let Some(previous) = self.previous.take() {
                unsafe { std::env::set_var("VIZE_TEST_WORKSPACE_NODE_MODULES", previous) };
            } else {
                unsafe { std::env::remove_var("VIZE_TEST_WORKSPACE_NODE_MODULES") };
            }
        }
    }

    let _lock = ENV_LOCK.lock().unwrap();
    let previous = std::env::var_os("VIZE_TEST_WORKSPACE_NODE_MODULES");
    match value {
        Some(value) => unsafe { std::env::set_var("VIZE_TEST_WORKSPACE_NODE_MODULES", value) },
        None => unsafe { std::env::remove_var("VIZE_TEST_WORKSPACE_NODE_MODULES") },
    }
    let _guard = EnvOverrideGuard { previous };
    run()
}

fn write_test_vue_stub(target: &Path) -> std::io::Result<()> {
    let vue_dir = target.join("vue");
    std::fs::create_dir_all(&vue_dir)?;
    std::fs::write(
        vue_dir.join("package.json"),
        r#"{
  "name": "vue",
  "types": "index.d.ts"
}"#,
    )?;
    std::fs::write(
        vue_dir.join("index.d.ts"),
        r#"export * from "@vue/runtime-dom";
"#,
    )?;
    write_test_vue_runtime_dom_stub(target)?;
    Ok(())
}

fn write_test_vue_runtime_dom_stub(target: &Path) -> std::io::Result<()> {
    let runtime_dom_dir = target.join("@vue").join("runtime-dom");
    std::fs::create_dir_all(&runtime_dom_dir)?;
    std::fs::write(
        runtime_dom_dir.join("package.json"),
        r#"{
  "name": "@vue/runtime-dom",
  "types": "index.d.ts"
}"#,
    )?;
    std::fs::write(
        runtime_dom_dir.join("index.d.ts"),
        r#"export interface ComponentPublicInstance<Props = {}> {
  $props: Props;
  $attrs: { [key: string]: unknown };
  $slots: { [key: string]: unknown };
  $refs: { [key: string]: unknown };
  $emit: (...args: any[]) => void;
}

export type DefineComponent<
  Props = {},
  RawBindings = {},
  D = {},
  C = {},
  M = {},
  Mixin = {},
  Extends = {},
  E = {},
  EE = string,
  PP = Props,
  PropsDefaults = {},
  MakeDefaultsOptional = true,
  Options = {},
  S = {}
> = {
  new (): ComponentPublicInstance<Props>;
} & ComponentOptions<Props, RawBindings, D, C, M>;

export type ComponentOptions<
  Props = {},
  RawBindings = any,
  D = any,
  C = any,
  M = any
> = {
  name?: string;
  __name?: string;
  __file?: string;
  __vccOpts?: any;
  props?: any;
  emits?: any;
  slots?: any;
  setup?: any;
  render?: Function;
  components?: any;
  directives?: any;
  inheritAttrs?: boolean;
  compatConfig?: any;
  call?: (this: unknown, ...args: unknown[]) => never;
  __isFragment?: never;
  __isTeleport?: never;
  __isSuspense?: never;
  __defaults?: any;
  __vapor?: boolean;
  __multiRoot?: boolean;
  __isKeepAlive?: boolean;
  __isBuiltIn?: boolean;
};

export interface FunctionalComponent<
  P = {},
  E = {},
  S = any
> {
  (props: P, ctx: any): any;
  props?: any;
  emits?: any;
  slots?: any;
  inheritAttrs?: boolean;
  displayName?: string;
  compatConfig?: any;
}

export type ConcreteComponent<
  Props = {},
  RawBindings = any,
  D = any,
  C = any,
  M = any,
  E = {},
  S = any
> = ComponentOptions<Props, RawBindings, D, C, M> | FunctionalComponent<Props, E, S>;

export interface Ref<T = unknown, _Raw = T> {
  value: T;
}

export interface ComputedRef<T = unknown> extends Ref<T> {
  readonly value: T;
}

export interface WritableComputedRef<T = unknown> extends Ref<T> {
  value: T;
}

export interface ShallowRef<T = unknown, _Raw = T> extends Ref<T, _Raw> {
  readonly __v_isShallow?: true;
}

export type InjectionKey<T> = symbol & { readonly __v_vlsInjection?: T };
export type PropType<T> = { new (...args: any[]): T & {} } | { (): T } | null;

export declare const Transition: DefineComponent;
export declare function defineComponent(options: any): DefineComponent;
export declare function defineAsyncComponent(source: any): DefineComponent;
export declare function defineProps<T = {}>(): T;
export declare function computed<T>(getter: () => T): ComputedRef<T>;
export declare function computed<T>(options: { get: () => T; set: (value: T) => void }): WritableComputedRef<T>;
export declare function ref<T>(value: T): Ref<T>;
export declare function reactive<T extends object>(target: T): T;
export declare function shallowRef<T>(value: T): ShallowRef<T>;
export declare function toRef<T extends object, K extends keyof T>(object: T, key: K): Ref<T[K]>;
export declare function useTemplateRef<T = unknown>(key: string): ShallowRef<T | null>;
export declare function useId(): string;
export declare function watch<T>(source: T, callback: (...args: any[]) => void, options?: any): void;
export declare function watchEffect(effect: (onCleanup: (cleanupFn: () => void) => void) => void): void;
export declare function onMounted(callback: () => void): void;
export declare function customRef<T>(factory: any): Ref<T>;
export declare function provide<T>(key: InjectionKey<T> | string | symbol, value: T): void;
export declare function inject<T>(key: InjectionKey<T> | string | symbol): T | undefined;
export declare function inject<T>(key: InjectionKey<T> | string | symbol, defaultValue: T): T;
export declare function markRaw<T extends object>(value: T): T;
export declare function createApp(root: any): {
  config: {
    globalProperties: { [key: string]: any };
  };
};
"#,
    )?;
    Ok(())
}

fn write_test_vite_stub(target: &Path) -> std::io::Result<()> {
    let vite_dir = target.join("vite");
    std::fs::create_dir_all(&vite_dir)?;
    std::fs::write(
        vite_dir.join("package.json"),
        r#"{
  "name": "vite",
  "types": "client.d.ts"
}"#,
    )?;
    std::fs::write(vite_dir.join("client.d.ts"), "")?;
    Ok(())
}

fn symlink_path(source: &Path, target: &Path) -> std::io::Result<()> {
    if target.is_symlink() || target.is_file() {
        std::fs::remove_file(target)?;
    } else if target.exists() {
        std::fs::remove_dir_all(target)?;
    }
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(source, target)
    }
    #[cfg(windows)]
    {
        let metadata = std::fs::metadata(source)?;
        if metadata.is_dir() {
            std::os::windows::fs::symlink_dir(source, target)
        } else {
            std::os::windows::fs::symlink_file(source, target)
        }
    }
}
