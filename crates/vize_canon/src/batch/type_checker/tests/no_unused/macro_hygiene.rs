use super::super::{
    create_project_case_without_node_modules, resolve_test_tsgo_binary,
    snapshot_project_diagnostics,
};
use super::write_no_unused_tsconfig;

#[test]
fn only_real_component_macro_results_are_consumed_by_the_public_signature() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let root = create_project_case_without_node_modules(
        "macro-result-ownership",
        &[
            (
                "src/Component.vue",
                r#"<script setup lang="ts">
const props = defineProps<{ value: string }>();
const emit = defineEmits<{ change: [value: string] }>();
const ordinary = 1;
</script><template><div /></template>"#,
            ),
            (
                "src/Local.vue",
                r#"<script setup lang="ts">
const defineProps = () => ({ value: 1 });
const result = defineProps();
</script><template><div /></template>"#,
            ),
            (
                "src/Imported.vue",
                r#"<script setup lang="ts">
import { defineEmits } from './helpers';
const result = defineEmits();
</script><template><div /></template>"#,
            ),
            (
                "src/helpers.ts",
                "export function defineEmits() { return () => 1; }",
            ),
        ],
    );
    write_no_unused_tsconfig(&root);
    let actual = snapshot_project_diagnostics(&root).unwrap();
    let expected = [
        (
            "src/Component.vue",
            Some(6133),
            "4:7:error 'ordinary' is declared but its value is never read.",
        ),
        (
            "src/Imported.vue",
            Some(6133),
            "3:7:error 'result' is declared but its value is never read.",
        ),
        (
            "src/Local.vue",
            Some(6133),
            "3:7:error 'result' is declared but its value is never read.",
        ),
    ]
    .map(|(file, code, message)| (file.into(), code, message.into()));
    assert_eq!(actual, expected);
    std::fs::remove_dir_all(root).unwrap();
}
