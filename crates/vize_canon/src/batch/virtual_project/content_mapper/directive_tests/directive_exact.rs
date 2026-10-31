use super::*;

#[test]
fn expect_directive_targets_the_next_template_line() {
    let source = "<script setup lang=\"ts\">\nconst count = 1\n</script>\n<template>\n  <!-- @vue-expect-error -->\n  {{ count.bad }}\n</template>\n";
    let result = transform(source);
    let directives = result.diagnostic_directives.unwrap();

    assert_eq!(
        directives.unused_expect_directive_diagnostics.len(),
        1,
        "{directives:?}"
    );
    assert_eq!(directives.unused_expect_directive_diagnostics[0].code, 4);
    assert_eq!(directives.directives.len(), 1, "{directives:?}");
    let [
        original_start,
        original_len,
        virtual_start,
        virtual_end,
        policy,
        unused_index,
    ] = directives.directives[0].0;
    assert_eq!(policy, DIRECTIVE_POLICY_EXPECT);
    assert_eq!(unused_index, 0);
    assert_eq!(
        &source[original_start..original_start + original_len],
        "@vue-expect-error"
    );
    let suppressed = &result.text.as_str()[virtual_start..virtual_end];
    assert_eq!(suppressed.matches("count").count(), 1, "{suppressed}");
}
