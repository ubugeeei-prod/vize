//! Runtime and mixin props keep Vue's public requiredness across SFC imports.

use super::super::super::{
    BatchTypeChecker, TypeChecker, create_project_case, relative_path, resolve_test_tsgo_binary,
};

#[test]
fn imported_runtime_and_mixin_props_preserve_requiredness() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "imported-runtime-mixin-required-props",
        &[
            (
                "src/RuntimeRequired.vue",
                r#"<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  props: { count: { type: Number, required: true } },
})
</script>
"#,
            ),
            (
                "src/RuntimeOptional.vue",
                r#"<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({ props: { count: Number } })
</script>
"#,
            ),
            (
                "src/MixinRequired.vue",
                r#"<script lang="ts">
import { defineComponent } from 'vue'
const base = defineComponent({
  props: { count: { type: Number, required: true } },
})
export default defineComponent({ mixins: [base] })
</script>
"#,
            ),
            (
                "src/MixinOptional.vue",
                r#"<script lang="ts">
import { defineComponent } from 'vue'
const base = defineComponent({ props: { count: Number } })
export default defineComponent({ mixins: [base] })
</script>
"#,
            ),
            (
                "src/DirectDefault.vue",
                r#"<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  props: { label: { type: String, default: 'x' } },
})
</script>
"#,
            ),
            (
                "src/DirectAndMixin.vue",
                r#"<script lang="ts">
import { defineComponent } from 'vue'
const base = defineComponent({
  props: { inherited: { type: Number, required: true } },
})
export default defineComponent({
  mixins: [base],
  props: { own: String },
})
</script>
"#,
            ),
            (
                "src/Parent.vue",
                r#"<script setup lang="ts">
import RuntimeRequired from './RuntimeRequired.vue'
import RuntimeOptional from './RuntimeOptional.vue'
import MixinRequired from './MixinRequired.vue'
import MixinOptional from './MixinOptional.vue'
import DirectDefault from './DirectDefault.vue'
import DirectAndMixin from './DirectAndMixin.vue'
</script>
<template>
  <RuntimeRequired />
  <RuntimeOptional />
  <MixinRequired />
  <MixinOptional />
  <DirectDefault />
  <DirectAndMixin />
</template>
"#,
            ),
        ],
    );
    if !project_root.join("node_modules/vue/dist").exists() {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    }

    let mut checker = BatchTypeChecker::new(&project_root).unwrap();
    checker.enable_options_api();
    checker.scan_project().unwrap();
    let snapshot = checker.check_project().unwrap();
    let _ = std::fs::remove_dir_all(&project_root);
    let actual: Vec<_> = snapshot
        .diagnostics
        .iter()
        .map(|diagnostic| {
            (
                relative_path(&project_root, &diagnostic.file).to_string(),
                diagnostic.code,
                diagnostic.line + 1,
                diagnostic.column + 1,
            )
        })
        .collect();
    assert_eq!(
        actual,
        [
            ("src/Parent.vue".to_string(), Some(2345), 10, 4),
            ("src/Parent.vue".to_string(), Some(2345), 12, 4),
            ("src/Parent.vue".to_string(), Some(2345), 15, 4),
        ],
        "required runtime and mixin props must cross the SFC boundary: {snapshot:#?}"
    );
}
