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

#[test]
fn script_and_style_pragmas_report_from_the_same_final_usage_state() {
    let linter = Linter::with_preset(crate::LintPreset::Opinionated)
        .with_enabled_rules(Some(vec![
            "vue/comment-directive".into(),
            "script/no-with-defaults".into(),
            "css/no-important".into(),
        ]))
        .with_report_unused_disable_directives(true);
    let cases = [
        (
            r#"<script setup lang="ts">
// eslint-disable-next-line script/no-with-defaults
const props = defineProps<{ a?: number }>();
</script>
<template><p>{{ props.a }}</p></template>"#,
            "script/no-with-defaults",
        ),
        (
            r#"<template><div class="a" /></template>
<style scoped>
/* oxlint-disable-next-line css/no-important */
.a { margin: 0; }
</style>"#,
            "css/no-important",
        ),
    ];

    for (source, rule) in cases {
        let result = linter.lint_sfc(source, "Unused.vue");
        let unused = result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.rule_name == "vue/comment-directive")
            .collect::<Vec<_>>();
        assert_eq!(unused.len(), 1, "{rule}: {:?}", result.diagnostics);
        assert_eq!(
            unused[0].message,
            format!("Unused eslint-disable directive (no problems were reported from '{rule}').")
        );
    }
}

#[test]
fn named_vize_forget_reports_unused_only_when_no_final_diagnostic_matches() {
    let linter = Linter::new()
        .with_enabled_rules(Some(vec![
            "vue/comment-directive".into(),
            "vue/require-v-for-key".into(),
        ]))
        .with_report_unused_disable_directives(true);
    let unused = linter.lint_sfc(
        r#"<template>
<!-- @vize:forget vue/require-v-for-key -->
<div>clean</div>
</template>"#,
        "Unused.vue",
    );
    assert_eq!(
        unused
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.rule_name == "vue/comment-directive")
            .count(),
        1,
        "{:?}",
        unused.diagnostics
    );

    let used = linter.lint_sfc(
        r#"<template>
<!-- @vize:forget vue/require-v-for-key -->
<li v-for="item in items">{{ item }}</li>
</template>"#,
        "Used.vue",
    );
    assert!(used.diagnostics.is_empty(), "{:?}", used.diagnostics);
}

#[test]
fn stylus_spellings_read_line_comment_directives_alike() {
    let linter = Linter::new()
        .with_enabled_rules(Some(vec![
            "vue/comment-directive".into(),
            "css/no-important".into(),
        ]))
        .with_report_unused_disable_directives(true);
    for lang in ["stylus", "styl"] {
        let source = vize_l0::cstr!(
            "<template><div /></template>\n<style lang=\"{lang}\">\n// eslint-disable-next-line css/no-important\n.a\n  color red\n</style>"
        );
        let unused = linter
            .lint_sfc(&source, "Card.vue")
            .diagnostics
            .into_iter()
            .filter(|diagnostic| diagnostic.rule_name == "vue/comment-directive")
            .count();
        assert_eq!(unused, 1, "{lang}");
    }
}
