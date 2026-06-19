use super::{BatchTypeChecker, create_project_case, relative_path, resolve_test_tsgo_binary};
use crate::batch::TypeChecker;

#[test]
fn accepts_legacy_vue2_required_options_props_in_setup() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let project_root = create_project_case(
        "legacy-vue2-required-options-props",
        &[(
            "src/App.vue",
            r#"<script lang="ts">
import { defineComponent, type PropType } from 'vue'

const componentProps = {
  items: {
    type: Array as PropType<Array<{ id: string }>>,
    required: true,
  },
}

export default defineComponent({
  props: componentProps,
  setup(props) {
    props.items.findIndex((item) => item.id)
    props.items[0]
    return {}
  },
})
</script>
"#,
        )],
    );

    if !project_root.join("node_modules/vue/dist").exists() {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    }

    let mut checker = match BatchTypeChecker::new(&project_root) {
        Ok(checker) => checker,
        Err(_) => {
            let _ = std::fs::remove_dir_all(&project_root);
            return;
        }
    };
    checker.enable_legacy_vue2();
    checker.scan_project().unwrap();
    let result = checker.check_project().unwrap();
    let unexpected: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.file.ends_with("App.vue")
                && matches!(diagnostic.code, Some(18048 | 2532 | 7031))
        })
        .collect();

    assert!(
        unexpected.is_empty(),
        "expected required Vue 2 Options API props to be non-optional in setup(): {unexpected:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn accepts_legacy_vue2_options_prop_type_shape_matrix() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let project_root = create_project_case(
        "legacy-vue2-options-prop-type-shapes",
        &[
            (
                "src/types.ts",
                r#"export type ImportedItem = { id: string; count: number }
export type ImportedStatus = "ready" | "draft"
"#,
            ),
            (
                "src/App.vue",
                r#"<script lang="ts">
import { defineComponent, type PropType as VuePropType } from 'vue'
import type { ImportedItem, ImportedStatus } from './types'

type LocalItem = ImportedItem & { local: boolean }
type LocalPropType<T> = VuePropType<T>
type NestedShape = { nested: { id: string; status: ImportedStatus } }

const nestedObjectProp = {
  type: Object as LocalPropType<NestedShape>,
  required: true,
}

export default defineComponent({
  props: {
    status: { type: String as VuePropType<ImportedStatus | "archived">, required: true },
    selected: { type: Object as VuePropType<ImportedItem & { enabled: boolean }>, required: true },
    items: { type: Array as LocalPropType<ReadonlyArray<LocalItem>>, required: true },
    readonlyItems: { type: Array as VuePropType<readonly ImportedItem[]>, required: true },
    formatter: { type: Function as VuePropType<(item: ImportedItem) => string>, required: true },
    nestedObject: nestedObjectProp,
  },
  setup(props) {
    const formatted = props.formatter(props.selected)
    const firstId = props.items[0]?.id
    const readonlyCount = props.readonlyItems[0]?.count ?? 0
    const nestedId = props.nestedObject?.nested.id ?? ""
    return { formatted, firstId, readonlyCount, nestedId }
  },
})
</script>

<template>
  <div>
    {{ status }}
    {{ selected.id }}
    {{ items[0]?.local }}
    {{ readonlyItems[0]?.count }}
    {{ nestedObject?.nested.status }}
    {{ formatted }}
    {{ firstId }}
    {{ readonlyCount }}
    {{ nestedId }}
  </div>
</template>
"#,
            ),
        ],
    );

    if !project_root.join("node_modules/vue/dist").exists() {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    }

    let mut checker = match BatchTypeChecker::new(&project_root) {
        Ok(checker) => checker,
        Err(_) => {
            let _ = std::fs::remove_dir_all(&project_root);
            return;
        }
    };
    checker.enable_legacy_vue2();
    checker.scan_project().unwrap();
    let result = checker.check_project().unwrap();
    let unexpected = result
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.file.ends_with("App.vue")
                && matches!(
                    diagnostic.code,
                    Some(1128 | 1131 | 18048 | 2339 | 2345 | 2532 | 7006 | 7031)
                )
        })
        .map(|diagnostic| {
            (
                relative_path(&project_root, &diagnostic.file),
                diagnostic.code,
                diagnostic.line,
                diagnostic.column,
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        unexpected,
        Vec::<(vize_carton::String, Option<u32>, u32, u32)>::new()
    );

    let _ = std::fs::remove_dir_all(&project_root);
}
