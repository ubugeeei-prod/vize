use super::*;

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

    // Real instance typing requires the real Vue package. The facade path is
    // covered separately below so both resolution modes keep the same shape.
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
            .any(|(file, code, _)| file == "src/App.vue" && *code == Some(2345)),
        "expected Options API data binding to keep its number type in the template: {snapshot:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn batch_type_checker_options_api_wrap_adds_no_errors_in_facade_fallback() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    // With no resolvable Vue package, the bundled facade must still provide
    // contextual `this` for nested Options API objects.
    with_workspace_node_modules_override(Some("__none__"), || {
        let project_root = create_project_case_without_node_modules(
            "options-api-wrap-facade-fallback",
            &[("src/App.vue", OPTIONS_API_THIS_IN_COMPUTED_SFC)],
        );

        let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
            let _ = std::fs::remove_dir_all(&project_root);
            return;
        };

        assert!(snapshot.is_empty(), "{snapshot:#?}");
        let _ = std::fs::remove_dir_all(&project_root);
    });
}

#[test]
fn batch_type_checker_facade_fallback_contextually_types_options_api() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    with_workspace_node_modules_override(Some("__none__"), || {
        let project_root = create_project_case_without_node_modules(
            "options-api-setup-facade-fallback",
            &[(
                "src/App.vue",
                r#"<script lang="ts">
import { defineComponent, type PropType } from "vue"

type Item = { id: string }
export default defineComponent({
  props: {
    items: { type: Array as PropType<Item[]>, required: true },
  },
  emits: { select: (_item: Item) => true },
  setup(props, { emit }) {
    emit("select", props.items[0])
    return {}
  },
  methods: {
    selectFirst() {
      this.$emit("select", this.items[0])
    },
  },
})
</script>
"#,
            )],
        );

        let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
            let _ = std::fs::remove_dir_all(&project_root);
            return;
        };

        assert!(
            snapshot.is_empty(),
            "fallback defineComponent must type setup parameters and nested instance methods: {snapshot:#?}"
        );

        let _ = std::fs::remove_dir_all(&project_root);
    });
}
