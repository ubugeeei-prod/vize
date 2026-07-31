//! SFC directive lexer coverage for JavaScript template interpolation.

use super::{lint_sfc, owned, unused};

fn assert_msg_reported(sfc: &str) {
    assert_eq!(
        owned(&lint_sfc(sfc)),
        vec![unused(sfc, "msg", "msg: string")]
    );
}

#[test]
fn nested_template_marker_is_not_a_comment_directive() {
    let sfc = r#"<script setup lang="ts">
const marker = `outer ${`// eslint-disable-next-line vue/no-unused-properties`}`
defineProps<{ msg: string }>();
</script>

<template><div>hi</div></template>
"#;
    assert_msg_reported(sfc);
}

#[test]
fn interpolation_comment_remains_a_real_directive() {
    let sfc = r#"<script setup lang="ts">
const marker = `outer ${ /* eslint-disable-next-line vue/no-unused-properties */ 1 }`
defineProps<{ msg: string }>();
</script>

<template><div>hi</div></template>
"#;
    assert_eq!(owned(&lint_sfc(sfc)), Vec::new());
}

#[test]
fn template_text_marker_cannot_disable_a_script_diagnostic() {
    let sfc = r#"<template><div>// eslint-disable vue/no-unused-properties</div></template>
<script setup lang="ts">
defineProps<{ msg: string }>();
</script>
"#;
    assert_msg_reported(sfc);
}

#[test]
fn multiline_html_attribute_marker_is_not_a_comment_directive() {
    let sfc = r#"<template><div title="prefix
// eslint-disable vue/no-unused-properties
suffix"></div></template>
<script setup lang="ts">
defineProps<{ msg: string }>();
</script>
"#;
    assert_msg_reported(sfc);
}

#[test]
fn escaped_newline_keeps_the_marker_inside_a_script_string() {
    let sfc = r#"<script setup lang="ts">
const marker = "prefix\
// eslint-disable-next-line vue/no-unused-properties"
defineProps<{ msg: string }>();
</script>
<template><div>hi</div></template>
"#;
    assert_msg_reported(sfc);
}
