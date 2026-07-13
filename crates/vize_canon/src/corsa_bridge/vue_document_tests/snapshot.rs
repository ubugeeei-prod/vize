use super::{CorsaVueVirtualDocumentOptions, build_vue_virtual_project};

#[test]
fn retains_parse_once_script_validation() {
    let project = tempfile::TempDir::new().expect("temp project");
    let host_path = project.path().join("Bad.vue");
    let source = r#"<script setup lang="ts">
const { msg = 0 } = defineProps<{ msg?: string }>();
</script>"#;
    let virtual_project = build_vue_virtual_project(
        &host_path,
        source,
        CorsaVueVirtualDocumentOptions::default(),
    )
    .expect("virtual project");
    let syntax = virtual_project
        .host
        .script_syntax
        .as_deref()
        .expect("script syntax snapshot");
    let error = syntax
        .validate_script_setup_semantics(source)
        .expect_err("invalid default should remain in the snapshot");

    assert_eq!(
        error.code.as_deref(),
        Some("DEFINE_PROPS_DESTRUCTURE_DEFAULT_TYPE")
    );
}
