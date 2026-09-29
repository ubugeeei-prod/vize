use super::Linter;

#[test]
fn eslint_disable_next_line_suppresses_script_rules() {
    let linter = Linter::new().with_enabled_rules(Some(vec!["script/no-with-defaults".into()]));
    let suppressed = linter.lint_sfc(
        r#"<script setup lang="ts">
// eslint-disable-next-line script/no-with-defaults
const props = withDefaults(defineProps<{ a?: number }>(), { a: 1 });
</script>
<template><p>{{ props.a }}</p></template>"#,
        "Counter.vue",
    );
    assert!(
        suppressed
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.rule_name != "script/no-with-defaults"),
        "{:?}",
        suppressed.diagnostics
    );

    let reported = linter.lint_sfc(
        r#"<script setup lang="ts">
const props = withDefaults(defineProps<{ a?: number }>(), { a: 1 });
</script>
<template><p>{{ props.a }}</p></template>"#,
        "Counter.vue",
    );
    assert!(
        reported
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.rule_name == "script/no-with-defaults"),
        "{:?}",
        reported.diagnostics
    );
}

#[test]
fn style_disable_comments_suppress_only_the_marked_declarations() {
    let linter = Linter::new().with_enabled_rules(Some(vec!["css/no-important".into()]));
    let source = r#"<template>
  <div class="a" />
</template>

<style scoped>
.a { margin: 0 !important; }
/* eslint-disable-next-line css/no-important */
.b { margin: 0 !important; }
.c { margin: 0 !important; } /* eslint-disable-line css/no-important */
/* eslint-disable css/no-important */
.d { margin: 0 !important; }
/* eslint-enable css/no-important */
.e { margin: 0 !important; }
.f { margin: 0 !important; }
.g { margin: 0 !important; } /* vize-disable-line css/no-important */
.h { margin: 0 !important; }
</style>"#;
    let result = linter.lint_sfc(source, "Card.vue");
    let lines: Vec<usize> = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == "css/no-important")
        .map(|diagnostic| {
            source
                .get(..diagnostic.start as usize)
                .unwrap_or("")
                .bytes()
                .filter(|byte| *byte == b'\n')
                .count()
                + 1
        })
        .collect();
    assert_eq!(lines, vec![6, 13, 14, 16], "{lines:?}");
}
