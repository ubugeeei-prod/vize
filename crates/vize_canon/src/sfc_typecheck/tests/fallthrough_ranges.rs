use super::{SfcTypeCheckOptions, type_check_sfc};

#[test]
fn fallthrough_diagnostic_range_uses_authored_template_offsets() {
    let source = r#"<script setup lang="ts">
const marker = 1;
</script>

<template>
  <header>top</header>
  <main>body</main>
</template>
"#;
    let result = type_check_sfc(source, &SfcTypeCheckOptions::new("MultiRoot.vue"));
    let diagnostic = result
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code.as_deref() == Some("fallthrough-attrs"))
        .expect("fallthrough attrs diagnostic should be reported");
    let template_start = source.find("<template>").unwrap() as u32;
    let script_start = source.find("<script").unwrap() as u32;
    let header_start = source.find("<header>").unwrap() as u32;

    assert!(
        diagnostic.start > template_start,
        "diagnostic must point inside the authored template: {diagnostic:?}"
    );
    assert!(
        diagnostic.start >= header_start,
        "diagnostic should start at the first root template node: {diagnostic:?}"
    );
    assert!(
        diagnostic.start > script_start && diagnostic.end > diagnostic.start,
        "diagnostic must not collapse onto the leading script block: {diagnostic:?}"
    );
}
