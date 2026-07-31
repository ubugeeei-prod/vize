//! SFC directive lexer coverage for JavaScript template interpolation.

use super::{lint_sfc, owned, unused};

#[test]
fn nested_template_marker_is_not_a_comment_directive() {
    let sfc = r#"<script setup lang="ts">
const marker = `outer ${`// eslint-disable-next-line vue/no-unused-properties`}`
defineProps<{ msg: string }>();
</script>

<template><div>hi</div></template>
"#;
    assert_eq!(
        owned(&lint_sfc(sfc)),
        vec![unused(sfc, "msg", "msg: string")]
    );
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
