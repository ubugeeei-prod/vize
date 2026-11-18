#![expect(clippy::string_slice, reason = "tests assert by panicking")]

//! `vue/prefer-pug-lang`: a `lang="jade"` template reports its `lang` and
//! every construct whose jade 1.x meaning differs from pug 3, each at its
//! authored bytes with its rewrite; the `lang` is fixed to `pug` only when
//! nothing jade-only is left. A `lang="pug"` template reports nothing.

use vize_patina::{LintResult, Linter};

const RULE: &str = "vue/prefer-pug-lang";

fn sfc(lang: &str, body: &str) -> String {
    format!(
        "<script setup>\nconst id = 1;\n</script>\n<template lang=\"{lang}\">\n{body}</template>\n"
    )
}

fn lint(source: &str) -> LintResult {
    Linter::new().lint_sfc(source, "Jade.vue")
}

/// `(authored text, rewrite)` of every `vue/prefer-pug-lang` finding.
fn findings<'s>(source: &'s str, result: &LintResult) -> Vec<(&'s str, Option<String>)> {
    result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == RULE)
        .map(|diagnostic| {
            let fixed = diagnostic
                .fix
                .as_ref()
                .map(|fix| fix.apply(source).to_string());
            (
                &source[diagnostic.start as usize..diagnostic.end as usize],
                fixed,
            )
        })
        .collect()
}

#[test]
fn each_jade_only_construct_is_reported_with_its_rewrite() {
    let rows = [
        (
            "ul\n  - each n in [1, 2]\n    li= n\n",
            "- ",
            "ul\n  each n in [1, 2]\n    li= n\n",
        ),
        (
            "mixin item(x)\n  p= x\nmixin item('a')\n",
            "mixin item('a')",
            "mixin item(x)\n  p= x\n+item('a')\n",
        ),
        (
            "a(href=\"/users/#{id}\") x\n",
            "#{id}",
            "a(:href=\"`/users/${id}`\") x\n",
        ),
    ];
    for (body, construct, rewritten) in rows {
        let source = sfc("jade", body);
        let result = lint(&source);
        let found = findings(&source, &result);
        assert_eq!(found.len(), 2, "{body}: {found:?}");
        // The lang has no fix while a jade-only construct is left.
        assert_eq!(found[0], ("jade", None), "{body}");
        assert_eq!(found[1].0, construct, "{body}");
        assert_eq!(
            found[1].1.as_deref(),
            Some(sfc("jade", rewritten).as_str()),
            "{body}"
        );
    }
}

#[test]
fn each_of_is_reported_without_a_rewrite() {
    let source = sfc("jade", "ul\n  each n of [1, 2]\n    li= n\n");
    let found = findings(&source, &lint(&source));
    assert_eq!(found, [("jade", None), ("each n of [1, 2]", None)]);
}

#[test]
fn a_template_with_nothing_jade_only_gets_the_safe_lang_fix() {
    let body = "ul\n  each n in [1, 2]\n    li= n\n";
    let source = sfc("jade", body);
    let found = findings(&source, &lint(&source));
    assert_eq!(found, [("jade", Some(sfc("pug", body)))]);
    // After the fix the rule is silent, and the template lints like before.
    let fixed = sfc("pug", body);
    assert_eq!(findings(&fixed, &lint(&fixed)), []);
}

#[test]
fn the_rule_is_configurable_and_pug_reports_nothing() {
    let source = sfc("jade", "ul\n  - each n in [1, 2]\n    li= n\n");
    let off = Linter::new().with_disabled_rules(vec![RULE.into()]);
    let result = off.lint_sfc(&source, "Jade.vue");
    assert_eq!(findings(&source, &result), []);
    let pug = sfc("pug", "ul\n  each n of [1, 2]\n    li= n\n");
    assert_eq!(findings(&pug, &lint(&pug)), []);
}
