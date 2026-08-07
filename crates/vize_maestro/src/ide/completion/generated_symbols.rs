//! Filtering of virtual-TS machinery out of checker-backed completions.

/// Whether a completion label names virtual-TS machinery a template author can
/// never reference: generated `__vize*`/`__Vize*` helpers, the `__`-prefixed
/// type captures, or the setup-scope compiler macro shims (#3911).
#[cfg(feature = "native")]
pub(super) fn is_generated_template_symbol(label: &str) -> bool {
    label.starts_with("__")
        || matches!(
            label,
            "defineProps"
                | "defineEmits"
                | "defineExpose"
                | "defineModel"
                | "defineSlots"
                | "withDefaults"
                | "useTemplateRef"
        )
}

#[cfg(all(test, feature = "native"))]
mod tests {
    #[test]
    fn shims_and_dunder_names_are_filtered_authored_names_stay() {
        assert!(super::is_generated_template_symbol("__vForList"));
        assert!(super::is_generated_template_symbol("__VizeTemplateRefs"));
        assert!(super::is_generated_template_symbol("defineProps"));
        assert!(!super::is_generated_template_symbol("users"));
        assert!(!super::is_generated_template_symbol("_key"));
    }
}
