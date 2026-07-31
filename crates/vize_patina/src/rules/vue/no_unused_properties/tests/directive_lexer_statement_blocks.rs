//! Statement-block and expression-brace boundaries for script directives.

use super::{lint_sfc, owned, unused};

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
    assert_eq!(
        owned(&lint_sfc(&sfc)),
        vec![unused(&sfc, "msg", "msg: string")],
        "{statement}"
    );
}

#[test]
fn regex_after_if_statement_block_is_not_a_directive() {
    assert_regex_marker_is_not_a_directive("if (ok) {}\n/[/*] @vize:expected */.test(input)");
}

#[test]
fn regex_after_function_statement_block_is_not_a_directive() {
    for declaration in [
        "function check() {}",
        "function check<T>() {}",
        "function check(): void {}",
        "function check<T>(): { value: T } {}",
        "function check<T extends (...args: never[]) => unknown>(): boolean {}",
    ] {
        assert_regex_marker_is_not_a_directive(&format!(
            "{declaration}\n/[/*] @vize:expected */.test(input)"
        ));
    }
}

#[test]
fn regex_after_loop_statement_blocks_is_not_a_directive() {
    assert_regex_marker_is_not_a_directive(
        "while (ok) { break }\n/[/*] @vize:expected */.test(input)",
    );
    assert_regex_marker_is_not_a_directive(
        "for (let index = 0; index < 1; index++) {}\n/[/*] @vize:expected */.test(input)",
    );
}

#[test]
fn regex_after_try_statement_blocks_is_not_a_directive() {
    assert_regex_marker_is_not_a_directive("try {} catch {}\n/[/*] @vize:expected */.test(input)");
    assert_regex_marker_is_not_a_directive(
        "try {} finally {}\n/[/*] @vize:expected */.test(input)",
    );
}

#[test]
fn regex_after_standalone_and_class_blocks_is_not_a_directive() {
    for statement in [
        "{}",
        "label: {}",
        "label\n: {}",
        "label\n/* comment */\n: {}",
        "async: {}",
        "class Check {}",
        "class Check extends Foo.Bar {}",
        "class Check extends mixin(foo < bar) {}",
        "class Check<T extends { value: string }> {}",
    ] {
        assert_regex_marker_is_not_a_directive(&format!(
            "{statement}\n/[/*] @vize:expected */.test(input)"
        ));
    }
}

#[test]
fn division_after_object_literal_keeps_a_real_directive() {
    let sfc = r#"<script setup lang="ts">
const object = { value: 10 }
  / 2 // eslint-disable-next-line vue/no-unused-properties
defineProps<{ msg: string }>();
</script>
<template><div>{{ object }}</div></template>
"#;
    assert_eq!(owned(&lint_sfc(sfc)), Vec::new());
}

#[test]
fn division_inside_destructuring_default_keeps_a_real_directive() {
    let sfc = r#"<script setup lang="ts">
declare const source: { value?: unknown }
const { value = { nested: 10 }
  / 2 // eslint-disable vue/no-unused-properties
} = source
defineProps<{ msg: string }>();
</script>
<template><div>{{ value }}</div></template>
"#;
    assert_eq!(owned(&lint_sfc(sfc)), Vec::new());
}

#[test]
fn division_after_function_and_class_expressions_keeps_a_real_directive() {
    for expression in [
        "function check() {}",
        "function check(): void {}",
        "() => {}",
        "class {}",
        "class extends Foo.Bar {}",
        "class<T extends { value: string }> {}",
    ] {
        let sfc = format!(
            r#"<script setup lang="ts">
const check = {expression}
  / 2 // eslint-disable-next-line vue/no-unused-properties
defineProps<{{ msg: string }}>();
</script>
<template><div>{{{{ check }}}}</div></template>
"#
        );
        assert_eq!(owned(&lint_sfc(&sfc)), Vec::new(), "{expression}");
    }
}

#[test]
fn division_after_default_exported_object_keeps_a_real_directive() {
    let sfc = r#"<script setup lang="ts">
export default { value: 1 }
  / 2 // eslint-disable-next-line vue/no-unused-properties
defineProps<{ msg: string }>();
</script>
<template><div>hi</div></template>
"#;
    assert_eq!(owned(&lint_sfc(sfc)), Vec::new());
}
