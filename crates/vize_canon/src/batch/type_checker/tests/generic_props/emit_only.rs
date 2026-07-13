use super::*;

fn assert_no_unused_generic_diagnostics(
    case_name: &str,
    files: &[(&str, &str)],
    failure_message: &str,
) {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let project_root = create_project_case(case_name, files);
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "target": "ES2023",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "noEmit": true,
    "noUnusedParameters": true,
    "jsx": "preserve",
    "jsxImportSource": "vue",
    "lib": ["ES2023", "DOM", "DOM.Iterable"],
    "skipLibCheck": true
  },
  "include": ["src/**/*.vue", "src/**/*.ts"]
}"#,
    )
    .unwrap();

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(snapshot.is_empty(), "{failure_message}: {snapshot:#?}");
    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn retains_generic_parameters_used_only_by_emits() {
    assert_no_unused_generic_diagnostics(
        "emit-only-generic-parameter",
        &[(
            "src/Emitter.vue",
            r#"<script setup lang="ts" generic="T extends string">
const emit = defineEmits<{ change: [value: T] }>();

emit("change", "valid" as T);
// @ts-expect-error numbers are outside the generic payload constraint
emit("change", 1);
</script>
"#,
        )],
        "emit-only SFC generics must not produce unused Props parameters",
    );
}

#[test]
fn retains_each_generic_parameter_used_by_props_or_emits() {
    assert_no_unused_generic_diagnostics(
        "split-props-and-emits-generic-parameters",
        &[
            (
                "src/types.ts",
                r#"export interface LocalProps<T> {
  value: T;
}
"#,
            ),
            (
                "src/Emitter.vue",
                r#"<script setup lang="ts" generic="T extends string, U extends number">
import type { LocalProps } from "./types";

defineProps<LocalProps<T>>();
const emit = defineEmits<{ change: [value: U] }>();

emit("change", 1 as U);
// @ts-expect-error strings are outside the generic payload constraint
emit("change", "invalid");
</script>
"#,
            ),
        ],
        "every SFC generic must be retained across props and emits",
    );
}
