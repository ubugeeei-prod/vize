//! Hoisted editor modules keep `withDefaults` prop JSDoc on their own aliases (#12461).
use super::*;

const DEFAULTS: &str = r#"<script setup lang="ts">
const props = withDefaults(defineProps<{
  /** Visible title. */
  title?: string
  open?: boolean
}>(), { title: 'none' })
console.log(props.title, props.open)
</script>"#;

#[test]
fn editor_projection_infers_documented_defaults_through_its_local_props_alias() {
    let root = tempfile::tempdir().unwrap();
    let host = root.path().join("Defaults.vue");
    std::fs::write(&host, DEFAULTS).unwrap();
    let session = EditorMirrorSession::new();
    let options = CorsaVueVirtualDocumentOptions {
        preserve_event_navigation: true,
        ..Default::default()
    };
    let project = build_vue_virtual_workspace_project(
        &host,
        DEFAULTS,
        options,
        &[],
        &[],
        environment(&session),
    )
    .unwrap();
    let code = project.host.pre_rewrite_code.as_str();
    for (needle, count) in [
        // The module keeps a local owner alias for public props...
        (
            "export type { __RuntimePropShape, __LooseRequired, __DefineProps,",
            1,
        ),
        // ...so both macros resolve `__DefineProps` in this module, and the
        // documented result keeps the authored declarations without changing
        // any value type.
        ("  function defineProps<_T = unknown>(): __DefineProps<", 1),
        (
            "): __WithDefaultsResult<_T, _D, _BKeys> & { readonly [K in keyof _T]: unknown };",
            1,
        ),
        ("const defineProps = __vize_defineProps;", 0),
        ("const withDefaults = __vize_withDefaults;", 0),
        ("const defineEmits = __vize_defineEmits;", 1),
    ] {
        assert_eq!(code.matches(needle).count(), count, "{needle}\n{code}");
    }
}
