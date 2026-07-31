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

#[test]
fn same_line_style_directive_cannot_leak_into_script_setup() {
    let sfc = r#"<style>/* eslint-disable vue/no-unused-properties */</style><script setup lang="ts">defineProps<{ msg: string }>();</script><template><div>hi</div></template>"#;
    assert_msg_reported(sfc);
}

#[test]
fn same_line_plain_script_directive_cannot_leak_into_script_setup() {
    let sfc = r#"<script>/* eslint-disable vue/no-unused-properties */</script><script setup lang="ts">defineProps<{ msg: string }>();</script><template><div>hi</div></template>"#;
    assert_msg_reported(sfc);
}

#[test]
fn regex_braces_do_not_hide_a_following_real_directive() {
    let sfc = r#"<script setup lang="ts">
const marker = `${/\{/.test('{')}`
// eslint-disable-next-line vue/no-unused-properties
defineProps<{ msg: string }>();
</script>
<template><div>hi</div></template>
"#;
    assert_eq!(owned(&lint_sfc(sfc)), Vec::new());
}

#[test]
fn tsx_raw_text_marker_is_not_a_comment_directive() {
    let sfc = r#"<script setup lang="tsx">
const vnode = <div>
// eslint-disable-next-line vue/no-unused-properties
</div>
defineProps<{ msg: string }>();
</script>
<template><div>hi</div></template>
"#;
    assert_msg_reported(sfc);
}

#[test]
fn division_at_the_start_of_a_line_does_not_hide_a_real_directive() {
    let sfc = r#"<script setup lang="ts">
const half = 10
  / 2
// eslint-disable-next-line vue/no-unused-properties
defineProps<{ msg: string }>();
</script>
<template><div>hi</div></template>
"#;
    assert_eq!(owned(&lint_sfc(sfc)), Vec::new());
}

#[test]
fn directive_on_the_script_tag_line_disables_the_next_line() {
    let sfc = r#"<script setup lang="ts">// eslint-disable-next-line vue/no-unused-properties
defineProps<{ msg: string }>();
</script>
<template><div>hi</div></template>
"#;
    assert_eq!(owned(&lint_sfc(sfc)), Vec::new());
}

#[test]
fn division_after_a_postfix_increment_does_not_hide_a_real_directive() {
    let sfc = r#"<script setup lang="ts">
let index = 1
const half = index++ / 2
// eslint-disable-next-line vue/no-unused-properties
defineProps<{ msg: string }>();
</script>
<template><div>{{ half }}</div></template>
"#;
    assert_eq!(owned(&lint_sfc(sfc)), Vec::new());
}

#[test]
fn tsx_generic_arrow_does_not_hide_a_real_directive() {
    let sfc = r#"<script setup lang="tsx">
const identity = <T,>(value: T) => value
// eslint-disable-next-line vue/no-unused-properties
defineProps<{ msg: string }>();
</script>
<template><div>hi</div></template>
"#;
    assert_eq!(owned(&lint_sfc(sfc)), Vec::new());
}

#[test]
fn multiline_tsx_generic_arrow_does_not_hide_a_real_directive() {
    let sfc = r#"<script setup lang="tsx">
const identity = <T,>(
  value: T,
) => value
// eslint-disable-next-line vue/no-unused-properties
defineProps<{ msg: string }>();
</script>
<template><div>hi</div></template>
"#;
    assert_eq!(owned(&lint_sfc(sfc)), Vec::new());
}

#[test]
fn division_after_postfix_increment_does_not_hide_a_real_directive() {
    let sfc = r#"<script setup lang="ts">
let value = 10
value++
  / 2 // eslint-disable-next-line vue/no-unused-properties
defineProps<{ msg: string }>();
</script>
<template><div>hi</div></template>
"#;
    assert_eq!(owned(&lint_sfc(sfc)), Vec::new());
}

#[test]
fn division_after_postfix_decrement_does_not_hide_a_real_directive() {
    let sfc = r#"<script setup lang="ts">
let value = 10
value--
  / 2 // eslint-disable-next-line vue/no-unused-properties
defineProps<{ msg: string }>();
</script>
<template><div>hi</div></template>
"#;
    assert_eq!(owned(&lint_sfc(sfc)), Vec::new());
}

#[test]
fn division_after_typescript_non_null_assertion_does_not_hide_a_real_directive() {
    let sfc = r#"<script setup lang="ts">
declare const value: number
const half = value!
  / 2 // eslint-disable-next-line vue/no-unused-properties
defineProps<{ msg: string }>();
</script>
<template><div>{{ half }}</div></template>
"#;
    assert_eq!(owned(&lint_sfc(sfc)), Vec::new());
}

#[test]
fn line_initial_logical_not_starts_an_expression() {
    let sfc = r#"<script setup lang="ts">
declare const input: string
const previous = input
!/[/*] @vize:expected */.test(input)
defineProps<{ msg: string }>();
</script>
<template><div>{{ previous }}</div></template>
"#;
    assert_msg_reported(sfc);
}

#[test]
fn inequality_operators_keep_regex_literals_out_of_comment_directives() {
    for operator in ["!=", "!=="] {
        let sfc = format!(
            r#"<script setup lang="ts">
declare const input: string
const differs = input {operator} /[/*] @vize:expected */.source
defineProps<{{ msg: string }}>();
</script>
<template><div>{{{{ differs }}}}</div></template>
"#
        );
        assert_msg_reported(&sfc);
    }
}

#[test]
fn constrained_multiline_tsx_generic_arrow_does_not_hide_a_real_directive() {
    let sfc = r#"<script setup lang="tsx">
const identity = <T extends unknown>(
  value: T,
) => value
// eslint-disable-next-line vue/no-unused-properties
defineProps<{ msg: string }>();
</script>
<template><div>{{ identity('ok') }}</div></template>
"#;
    assert_eq!(owned(&lint_sfc(sfc)), Vec::new());
}

fn assert_regex_marker_is_not_a_directive(statement: &str) {
    let sfc = format!(
        r#"<script setup lang="ts">
declare const ok: boolean
declare const input: string
{statement}
defineProps<{{ msg: string }}>();
</script>
<template><div>hi</div></template>
"#
    );
    assert_msg_reported(&sfc);
}

#[test]
fn regex_after_control_condition_stays_out_of_comment_directives() {
    assert_regex_marker_is_not_a_directive("if (ok) /[/*] @vize:expected */.test(input)");
}

#[test]
fn control_paren_survives_an_intervening_comment() {
    assert_regex_marker_is_not_a_directive(
        "if /* keep control state */ (ok) /[/*] @vize:expected */.test(input)",
    );
}

#[test]
fn for_await_control_paren_starts_a_regex_statement() {
    assert_regex_marker_is_not_a_directive(
        "for await (const value of input) /[/*] @vize:expected */.test(value)",
    );
}

#[test]
fn regex_after_else_stays_out_of_comment_directives() {
    assert_regex_marker_is_not_a_directive(
        "if (ok) input\nelse /[/*] @vize:expected */.test(input)",
    );
}

#[test]
fn regex_after_do_stays_out_of_comment_directives() {
    assert_regex_marker_is_not_a_directive("do /[/*] @vize:expected */.test(input); while (ok)");
}

#[test]
fn division_after_an_expression_paren_does_not_hide_a_real_directive() {
    let sfc = r#"<script setup lang="ts">
declare const input: number
const half = Number(input)
  / 2 // eslint-disable-next-line vue/no-unused-properties
defineProps<{ msg: string }>();
</script>
<template><div>{{ half }}</div></template>
"#;
    assert_eq!(owned(&lint_sfc(sfc)), Vec::new());
}

#[test]
fn jsx_after_control_condition_keeps_raw_text_out_of_comment_directives() {
    let sfc = r#"<script setup lang="tsx">
declare const ok: boolean
if (ok) <div>
  // eslint-disable-next-line vue/no-unused-properties
</div>
defineProps<{ msg: string }>();
</script>
<template><div>hi</div></template>
"#;
    assert_msg_reported(sfc);
}

#[test]
fn constrained_jsx_tag_is_not_a_typescript_generic_arrow() {
    let sfc = r#"<script setup lang="jsx">
const vnode = <T extends unknown>(
  // eslint-disable vue/no-unused-properties
)</T>
defineProps(['msg']);
</script>
<template><div>{{ vnode }}</div></template>
"#;
    assert_eq!(owned(&lint_sfc(sfc)), vec![unused(sfc, "msg", "'msg'")]);
}

#[test]
fn jsx_after_return_keeps_raw_text_out_of_comment_directives() {
    let sfc = r#"<script setup lang="tsx">
function render() {
  return <div>
    // eslint-disable-next-line vue/no-unused-properties
  </div>
}
defineProps<{ msg: string }>();
</script>
<template><div>hi</div></template>
"#;
    assert_msg_reported(sfc);
}
