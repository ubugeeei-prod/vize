use super::{SfcTypeCheckOptions, SfcTypeSeverity, type_check_sfc};

#[test]
fn fallthrough_attrs_multi_root_observing_attrs_warns() {
    let source = r#"<script setup>
const msg = 'hello'
</script>
<template>
  <div :class="$attrs.class">first</div>
  <div>second</div>
</template>"#;
    let result = type_check_sfc(source, &SfcTypeCheckOptions::new("test.vue"));
    let has_fallthrough = result
        .diagnostics
        .iter()
        .any(|d| d.code.as_deref() == Some("fallthrough-attrs"));
    assert!(has_fallthrough, "Should detect multi-root fallthrough");
}

#[test]
fn fallthrough_attrs_unobserved_multi_root_ok() {
    let source = r#"<script setup>
const msg = 'hello'
</script>
<template>
  <div>first</div>
  <div>second</div>
</template>"#;
    let result = type_check_sfc(source, &SfcTypeCheckOptions::new("test.vue"));
    let has_fallthrough = result
        .diagnostics
        .iter()
        .any(|d| d.code.as_deref() == Some("fallthrough-attrs"));
    assert!(
        !has_fallthrough,
        "plain Vue 3 fragments should not warn until attrs are observed"
    );
}

#[test]
fn fallthrough_attrs_single_root_ok() {
    let source = r#"<script setup>
const msg = 'hello'
</script>
<template>
  <div>{{ msg }}</div>
</template>"#;
    let result = type_check_sfc(source, &SfcTypeCheckOptions::new("test.vue"));
    let has_fallthrough = result
        .diagnostics
        .iter()
        .any(|d| d.code.as_deref() == Some("fallthrough-attrs"));
    assert!(!has_fallthrough, "Single root should not warn");
}

#[test]
fn fallthrough_attrs_inherit_attrs_false_ok() {
    let source = r#"<script setup>
defineOptions({ inheritAttrs: false })
</script>
<template>
  <header>first</header>
  <main>second</main>
</template>"#;
    let result = type_check_sfc(source, &SfcTypeCheckOptions::new("test.vue"));
    let has_fallthrough = result
        .diagnostics
        .iter()
        .any(|d| d.code.as_deref() == Some("fallthrough-attrs"));
    assert!(
        !has_fallthrough,
        "inheritAttrs: false intentionally disables automatic fallthrough"
    );
}

#[test]
fn fallthrough_attrs_strict_reports_error() {
    let source = r#"<script setup>
const msg = 'hello'
</script>
<template>
  <div :class="$attrs.class">first</div>
  <div>second</div>
</template>"#;
    let result = type_check_sfc(source, &SfcTypeCheckOptions::new("test.vue").strict());
    let has_error = result.diagnostics.iter().any(|d| {
        d.code.as_deref() == Some("fallthrough-attrs") && d.severity == SfcTypeSeverity::Error
    });
    assert!(has_error, "Strict mode should report as Error");
}

#[test]
fn fallthrough_diagnostic_range_uses_authored_template_offsets() {
    let source = r#"<script setup lang="ts">
const marker = 1;
</script>

<template>
  <header :class="$attrs.class">top</header>
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
    let header_start = source.find("<header").unwrap() as u32;

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
