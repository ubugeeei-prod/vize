//! SFC type checking functionality for Vue Single File Components.
//!
//! This module provides AST-based type analysis for Vue SFCs.
//! It leverages croquis for semantic analysis and scope tracking.
//!
//! ## Features
//!
//! - Props type validation (defineProps)
//! - Emits type validation (defineEmits)
//! - Template binding validation (undefined references)
//! - Virtual TypeScript generation with scope-aware code
//!
//! ## Architecture
//!
//! ```text
//! Vue SFC (.vue)
//!     |
//!     v
//! +-------------------------------------+
//! |  vize_atelier_sfc::parse_sfc        |
//! +-------------------------------------+
//!     |
//!     v
//! +-------------------------------------+
//! |  vize_croquis::Analyzer             |
//! |  - Script analysis (bindings)       |
//! |  - Template analysis (scopes)       |
//! |  - Macro tracking (defineProps)     |
//! +-------------------------------------+
//!     |
//!     v
//! +-------------------------------------+
//! |  type_check_sfc()                   |
//! |  - check_props_typing()             |
//! |  - check_emits_typing()             |
//! |  - check_template_bindings()        |
//! |  - generate_virtual_ts_with_scopes()|
//! +-------------------------------------+
//! ```

mod analysis;
mod checks;
mod runner;
mod virtual_ts;

pub use analysis::{
    SfcRelatedLocation, SfcTypeCheckOptions, SfcTypeCheckResult, SfcTypeDiagnostic, SfcTypeSeverity,
};
pub use runner::type_check_sfc;

#[cfg(test)]
mod tests {
    use super::{
        type_check_sfc, SfcTypeCheckOptions, SfcTypeCheckResult, SfcTypeDiagnostic, SfcTypeSeverity,
    };

    fn stable_snapshot_result(mut result: SfcTypeCheckResult) -> SfcTypeCheckResult {
        if result.analysis_time_ms.is_some() {
            result.analysis_time_ms = Some(0.0);
        }
        result
    }

    #[test]
    fn test_type_check_empty_sfc() {
        let source = "<template><div>Hello</div></template>";
        let options = SfcTypeCheckOptions::new("test.vue");
        let result = type_check_sfc(source, &options);
        assert!(!result.has_errors());
        assert_eq!(result.error_count, 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_type_check_result() {
        let mut result = SfcTypeCheckResult::empty();
        assert_eq!(result.error_count, 0);
        assert!(!result.has_errors());

        result.add_diagnostic(SfcTypeDiagnostic {
            severity: SfcTypeSeverity::Error,
            message: "test".into(),
            start: 0,
            end: 0,
            code: None,
            help: None,
            related: Vec::new(),
        });

        assert_eq!(result.error_count, 1);
        assert!(result.has_errors());
    }

    #[test]
    fn test_type_check_options_default() {
        let options = SfcTypeCheckOptions::new("test.vue");
        assert_eq!(options.filename, "test.vue");
        assert!(!options.strict);
        assert!(options.check_props);
        assert!(options.check_emits);
        assert!(options.check_template_bindings);
        assert!(!options.include_virtual_ts);
    }

    #[test]
    fn test_type_check_options_strict() {
        let options = SfcTypeCheckOptions::new("test.vue").strict();
        assert!(options.strict);
    }

    #[test]
    fn test_type_check_options_with_virtual_ts() {
        let options = SfcTypeCheckOptions::new("test.vue").with_virtual_ts();
        assert!(options.include_virtual_ts);
    }

    #[test]
    fn test_type_check_with_typed_props() {
        let source = r#"<script setup lang="ts">
interface Props {
    count: number;
    name: string;
}
const props = defineProps<Props>();
</script>
<template>
    <div>{{ props.count }} - {{ props.name }}</div>
</template>"#;
        let options = SfcTypeCheckOptions::new("test.vue");
        let result = type_check_sfc(source, &options);
        assert!(!result
            .diagnostics
            .iter()
            .any(|d| d.code.as_deref() == Some("untyped-prop")));
    }

    #[test]
    fn test_type_check_with_untyped_props_non_strict() {
        let source = r#"<script setup>
const props = defineProps(['count', 'name']);
</script>
<template>
    <div>{{ props.count }}</div>
</template>"#;
        let options = SfcTypeCheckOptions::new("test.vue");
        let result = type_check_sfc(source, &options);
        let has_untyped_prop_warning = result.diagnostics.iter().any(|d| {
            d.code.as_deref() == Some("untyped-prop") && d.severity == SfcTypeSeverity::Warning
        });
        assert!(has_untyped_prop_warning);
    }

    #[test]
    fn test_type_check_with_untyped_props_strict() {
        let source = r#"<script setup>
const props = defineProps(['count', 'name']);
</script>
<template>
    <div>{{ props.count }}</div>
</template>"#;
        let options = SfcTypeCheckOptions::new("test.vue").strict();
        let result = type_check_sfc(source, &options);
        let has_untyped_prop_error = result.diagnostics.iter().any(|d| {
            d.code.as_deref() == Some("untyped-prop") && d.severity == SfcTypeSeverity::Error
        });
        assert!(has_untyped_prop_error);
    }

    #[test]
    fn test_type_check_with_typed_emits() {
        let source = r#"<script setup lang="ts">
const emit = defineEmits<{
    (e: 'update', value: number): void;
    (e: 'close'): void;
}>();
</script>
<template>
    <button @click="emit('close')">Close</button>
</template>"#;
        let options = SfcTypeCheckOptions::new("test.vue");
        let result = type_check_sfc(source, &options);
        assert!(!result
            .diagnostics
            .iter()
            .any(|d| d.code.as_deref() == Some("untyped-emit")));
    }

    #[test]
    fn test_type_check_disabled_props_check() {
        let source = r#"<script setup>
const props = defineProps(['count']);
</script>
<template>
    <div>{{ props.count }}</div>
</template>"#;
        let mut options = SfcTypeCheckOptions::new("test.vue");
        options.check_props = false;
        let result = type_check_sfc(source, &options);
        assert!(!result
            .diagnostics
            .iter()
            .any(|d| d.code.as_deref() == Some("untyped-prop")));
    }

    #[test]
    fn test_type_check_undefined_binding() {
        let source = r#"<script setup>
const count = ref(0);
</script>
<template>
    <div>{{ undefinedVar }}</div>
</template>"#;
        let options = SfcTypeCheckOptions::new("test.vue");
        let result = type_check_sfc(source, &options);
        let has_undefined_error = result
            .diagnostics
            .iter()
            .any(|d| d.code.as_deref() == Some("undefined-binding"));
        assert!(has_undefined_error);
    }

    #[test]
    fn test_type_check_defined_binding() {
        let source = r#"<script setup>
const count = ref(0);
</script>
<template>
    <div>{{ count }}</div>
</template>"#;
        let options = SfcTypeCheckOptions::new("test.vue");
        let result = type_check_sfc(source, &options);
        insta::assert_debug_snapshot!(stable_snapshot_result(result));
    }

    #[test]
    fn test_type_check_virtual_ts_generation() {
        let source = r#"<script setup lang="ts">
const props = defineProps<{ count: number }>();
const message = ref('Hello');
</script>
<template>
    <div>{{ props.count }} - {{ message }}</div>
</template>"#;
        let options = SfcTypeCheckOptions::new("test.vue").with_virtual_ts();
        let result = type_check_sfc(source, &options);
        assert!(result.virtual_ts.is_some());
        let virtual_ts = result.virtual_ts.unwrap();
        insta::assert_snapshot!(virtual_ts.as_str());
    }

    #[test]
    fn test_type_severity_serialization() {
        assert_eq!(
            serde_json::to_string(&SfcTypeSeverity::Error).unwrap(),
            "\"error\""
        );
        assert_eq!(
            serde_json::to_string(&SfcTypeSeverity::Warning).unwrap(),
            "\"warning\""
        );
        assert_eq!(
            serde_json::to_string(&SfcTypeSeverity::Info).unwrap(),
            "\"info\""
        );
        assert_eq!(
            serde_json::to_string(&SfcTypeSeverity::Hint).unwrap(),
            "\"hint\""
        );
    }

    #[test]
    fn test_type_check_result_warning_count() {
        let mut result = SfcTypeCheckResult::empty();

        result.add_diagnostic(SfcTypeDiagnostic {
            severity: SfcTypeSeverity::Warning,
            message: "warning 1".into(),
            start: 0,
            end: 0,
            code: None,
            help: None,
            related: Vec::new(),
        });

        result.add_diagnostic(SfcTypeDiagnostic {
            severity: SfcTypeSeverity::Warning,
            message: "warning 2".into(),
            start: 0,
            end: 0,
            code: None,
            help: None,
            related: Vec::new(),
        });

        assert_eq!(result.error_count, 0);
        assert_eq!(result.warning_count, 2);
        assert!(!result.has_errors());
    }

    #[test]
    fn test_type_check_result_mixed_diagnostics() {
        let mut result = SfcTypeCheckResult::empty();

        result.add_diagnostic(SfcTypeDiagnostic {
            severity: SfcTypeSeverity::Error,
            message: "error".into(),
            start: 0,
            end: 0,
            code: None,
            help: None,
            related: Vec::new(),
        });

        result.add_diagnostic(SfcTypeDiagnostic {
            severity: SfcTypeSeverity::Warning,
            message: "warning".into(),
            start: 0,
            end: 0,
            code: None,
            help: None,
            related: Vec::new(),
        });

        result.add_diagnostic(SfcTypeDiagnostic {
            severity: SfcTypeSeverity::Info,
            message: "info".into(),
            start: 0,
            end: 0,
            code: None,
            help: None,
            related: Vec::new(),
        });

        assert_eq!(result.error_count, 1);
        assert_eq!(result.warning_count, 1);
        assert_eq!(result.diagnostics.len(), 3);
        assert!(result.has_errors());
    }

    #[test]
    fn test_type_check_v_for_destructuring() {
        let source = r#"<script setup lang="ts">
import { ref } from 'vue'
interface Item { id: number; name: string }
const items = ref<Item[]>([])
</script>
<template>
  <div v-for="{ id, name } in items" :key="id">
    {{ id }}: {{ name }}
  </div>
</template>"#;
        let options = SfcTypeCheckOptions::new("test.vue").with_virtual_ts();
        let result = type_check_sfc(source, &options);
        assert!(result.virtual_ts.is_some());
        insta::assert_debug_snapshot!(stable_snapshot_result(result));
    }

    #[test]
    fn test_type_check_nested_v_if_v_else() {
        let source = r#"<script setup lang="ts">
import { ref } from 'vue'
const show = ref(true)
const count = ref(0)
const message = ref('')
</script>
<template>
  <div v-if="show">{{ count }}</div>
  <div v-else>{{ message }}</div>
</template>"#;
        let options = SfcTypeCheckOptions::new("test.vue").with_virtual_ts();
        let result = type_check_sfc(source, &options);
        // All bindings are defined, should have no undefined binding errors
        assert!(!result
            .diagnostics
            .iter()
            .any(|d| d.code.as_deref() == Some("undefined-binding")));
    }

    #[test]
    fn test_type_check_scoped_slots() {
        let source = r#"<script setup lang="ts">
import MyList from './MyList.vue'
</script>
<template>
  <MyList>
    <template #default="{ item }">
      {{ item }}
    </template>
  </MyList>
</template>"#;
        let options = SfcTypeCheckOptions::new("test.vue").with_virtual_ts();
        let result = type_check_sfc(source, &options);
        assert!(result.virtual_ts.is_some());
    }

    #[test]
    fn test_type_check_v_model() {
        let source = r#"<script setup lang="ts">
import { ref } from 'vue'
const text = ref('')
</script>
<template>
  <input v-model="text" />
</template>"#;
        let options = SfcTypeCheckOptions::new("test.vue").with_virtual_ts();
        let result = type_check_sfc(source, &options);
        assert!(!result
            .diagnostics
            .iter()
            .any(|d| d.code.as_deref() == Some("undefined-binding")));
    }

    #[test]
    fn test_type_check_template_refs() {
        let source = r#"<script setup lang="ts">
import { ref } from 'vue'
const inputRef = ref<HTMLInputElement | null>(null)
</script>
<template>
  <input ref="inputRef" />
</template>"#;
        let options = SfcTypeCheckOptions::new("test.vue").with_virtual_ts();
        let result = type_check_sfc(source, &options);
        assert!(result.virtual_ts.is_some());
    }

    #[test]
    fn test_type_check_generic_component() {
        let source = r#"<script setup lang="ts" generic="T extends string">
const props = defineProps<{ items: T[] }>()
</script>
<template>
  <div v-for="item in props.items" :key="item">{{ item }}</div>
</template>"#;
        let options = SfcTypeCheckOptions::new("test.vue").with_virtual_ts();
        let result = type_check_sfc(source, &options);
        assert!(result.virtual_ts.is_some());
    }

    #[test]
    fn test_type_check_multiple_script_blocks() {
        let source = r#"<script lang="ts">
export default {
  name: 'MyComponent'
}
</script>
<script setup lang="ts">
import { ref } from 'vue'
const count = ref(0)
</script>
<template>
  <div>{{ count }}</div>
</template>"#;
        let options = SfcTypeCheckOptions::new("test.vue").with_virtual_ts();
        let result = type_check_sfc(source, &options);
        assert!(result.virtual_ts.is_some());
    }

    #[test]
    fn test_type_check_empty_template() {
        let source = r#"<script setup lang="ts">
import { ref } from 'vue'
const count = ref(0)
</script>
<template></template>"#;
        let options = SfcTypeCheckOptions::new("test.vue");
        let result = type_check_sfc(source, &options);
        assert!(!result.has_errors());
    }

    #[test]
    fn test_type_check_component_with_props_and_events() {
        let source = r#"<script setup lang="ts">
interface Props {
  title: string
  disabled?: boolean
}
interface Emits {
  (e: 'update:title', value: string): void
  (e: 'submit'): void
}
const props = defineProps<Props>()
const emit = defineEmits<Emits>()
</script>
<template>
  <div>
    <h1>{{ props.title }}</h1>
    <button :disabled="props.disabled" @click="emit('submit')">Submit</button>
  </div>
</template>"#;
        let options = SfcTypeCheckOptions::new("test.vue").with_virtual_ts();
        let result = type_check_sfc(source, &options);
        assert!(result.virtual_ts.is_some());
        let vts = result.virtual_ts.unwrap();
        insta::assert_snapshot!(vts.as_str());
    }

    // ========== Reactivity Loss Tests ==========

    #[test]
    fn test_check_reactivity_destructure_detected() {
        let source = r#"<script setup>
import { reactive } from 'vue'
const state = reactive({ count: 0 })
const { count } = state
</script>
<template><div>{{ count }}</div></template>"#;
        let options = SfcTypeCheckOptions::new("test.vue");
        let result = type_check_sfc(source, &options);
        let has_reactivity = result
            .diagnostics
            .iter()
            .any(|d| d.code.as_deref() == Some("reactivity-loss"));
        assert!(
            has_reactivity,
            "Should detect reactivity loss from destructuring"
        );
    }

    #[test]
    fn test_check_reactivity_no_issue() {
        let source = r#"<script setup>
import { ref } from 'vue'
const count = ref(0)
</script>
<template><div>{{ count }}</div></template>"#;
        let options = SfcTypeCheckOptions::new("test.vue");
        let result = type_check_sfc(source, &options);
        let has_reactivity = result
            .diagnostics
            .iter()
            .any(|d| d.code.as_deref() == Some("reactivity-loss"));
        assert!(!has_reactivity, "Should not detect reactivity loss");
    }

    #[test]
    fn test_check_reactivity_strict_severity() {
        let source = r#"<script setup>
import { reactive } from 'vue'
const state = reactive({ count: 0 })
const { count } = state
</script>
<template><div>{{ count }}</div></template>"#;
        let options = SfcTypeCheckOptions::new("test.vue").strict();
        let result = type_check_sfc(source, &options);
        let has_error = result.diagnostics.iter().any(|d| {
            d.code.as_deref() == Some("reactivity-loss") && d.severity == SfcTypeSeverity::Error
        });
        assert!(has_error, "Strict mode should report as Error");
    }

    // ========== Invalid Export Tests ==========

    #[test]
    fn test_check_invalid_exports_detected() {
        let source = r#"<script setup>
export const foo = 'bar'
</script>
<template><div>Hello</div></template>"#;
        let options = SfcTypeCheckOptions::new("test.vue");
        let result = type_check_sfc(source, &options);
        let has_invalid = result
            .diagnostics
            .iter()
            .any(|d| d.code.as_deref() == Some("invalid-export"));
        assert!(has_invalid, "Should detect invalid export");
    }

    #[test]
    fn test_check_invalid_exports_type_export_ok() {
        let source = r#"<script setup>
export type Foo = { name: string }
const count = ref(0)
</script>
<template><div>{{ count }}</div></template>"#;
        let options = SfcTypeCheckOptions::new("test.vue");
        let result = type_check_sfc(source, &options);
        let has_invalid = result
            .diagnostics
            .iter()
            .any(|d| d.code.as_deref() == Some("invalid-export"));
        assert!(!has_invalid, "Type exports should be allowed");
    }

    #[test]
    fn test_check_invalid_exports_disabled() {
        let source = r#"<script setup>
export const foo = 'bar'
</script>
<template><div>Hello</div></template>"#;
        let mut options = SfcTypeCheckOptions::new("test.vue");
        options.check_invalid_exports = false;
        let result = type_check_sfc(source, &options);
        let has_invalid = result
            .diagnostics
            .iter()
            .any(|d| d.code.as_deref() == Some("invalid-export"));
        assert!(!has_invalid, "Should not check when disabled");
    }

    // ========== Fallthrough Attrs Tests ==========

    #[test]
    fn test_check_fallthrough_attrs_multi_root() {
        let source = r#"<script setup>
const msg = 'hello'
</script>
<template>
  <div>first</div>
  <div>second</div>
</template>"#;
        let options = SfcTypeCheckOptions::new("test.vue");
        let result = type_check_sfc(source, &options);
        let has_fallthrough = result
            .diagnostics
            .iter()
            .any(|d| d.code.as_deref() == Some("fallthrough-attrs"));
        assert!(has_fallthrough, "Should detect multi-root fallthrough");
    }

    #[test]
    fn test_check_fallthrough_attrs_single_root_ok() {
        let source = r#"<script setup>
const msg = 'hello'
</script>
<template>
  <div>{{ msg }}</div>
</template>"#;
        let options = SfcTypeCheckOptions::new("test.vue");
        let result = type_check_sfc(source, &options);
        let has_fallthrough = result
            .diagnostics
            .iter()
            .any(|d| d.code.as_deref() == Some("fallthrough-attrs"));
        assert!(!has_fallthrough, "Single root should not warn");
    }

    #[test]
    fn test_check_fallthrough_attrs_strict() {
        let source = r#"<script setup>
const msg = 'hello'
</script>
<template>
  <div>first</div>
  <div>second</div>
</template>"#;
        let options = SfcTypeCheckOptions::new("test.vue").strict();
        let result = type_check_sfc(source, &options);
        let has_error = result.diagnostics.iter().any(|d| {
            d.code.as_deref() == Some("fallthrough-attrs") && d.severity == SfcTypeSeverity::Error
        });
        assert!(has_error, "Strict mode should report as Error");
    }
}
