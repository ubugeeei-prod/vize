use super::{BatchTypeChecker, create_project_case, resolve_test_tsgo_binary};
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
