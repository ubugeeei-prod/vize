//! TSX generic-arrow lookahead coverage for script directive scanning.

use super::{lint_sfc, owned, unused};

fn assert_msg_reported(sfc: &str) {
    assert_eq!(
        owned(&lint_sfc(sfc)),
        vec![unused(sfc, "msg", "msg: string")]
    );
}

#[test]
fn type_parameter_close_on_a_later_line_is_not_jsx() {
    let sfc = r#"<script setup lang="tsx">
const identity = <T
  extends unknown
>(value: T) => value
// eslint-disable-next-line vue/no-unused-properties
defineProps<{ msg: string }>();
</script>
<template><div>{{ identity('ok') }}</div></template>
"#;
    assert_eq!(owned(&lint_sfc(sfc)), Vec::new());
}

#[test]
fn multiline_constraint_and_default_are_not_jsx() {
    let sfc = r#"<script setup lang="tsx">
const identity = <T
  extends { value: string } = { value: string }
>(value: T): T => value
// eslint-disable-next-line vue/no-unused-properties
defineProps<{ msg: string }>();
</script>
<template><div>{{ identity({ value: 'ok' }) }}</div></template>
"#;
    assert_eq!(owned(&lint_sfc(sfc)), Vec::new());
}

#[test]
fn jsx_with_a_constraint_shaped_attribute_stays_jsx() {
    let sfc = r#"<script setup lang="tsx">
const vnode = <T
  extends="unknown"
>(
  // eslint-disable vue/no-unused-properties
)</T>
defineProps<{ msg: string }>();
</script>
<template><div>{{ vnode }}</div></template>
"#;
    assert_msg_reported(sfc);
}

#[test]
fn relational_less_than_does_not_start_jsx_lookahead() {
    let sfc = r#"<script setup lang="tsx">
declare const value: number
declare const T: number
const compared = value < T
  ? value
  : T
// eslint-disable-next-line vue/no-unused-properties
defineProps<{ msg: string }>();
</script>
<template><div>{{ compared }}</div></template>
"#;
    assert_eq!(owned(&lint_sfc(sfc)), Vec::new());
}
