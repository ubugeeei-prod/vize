//! Checker facts on statements that carry a JSDoc comment, as
//! typescript-eslint reads them.
use super::{corsa_available, lint_sfc_with_corsa};
use crate::{LintPreset, Linter};

const RULES: &[&str] = &["type/no-unsafe-call"];

/// A JSDoc comment is part of the statement's leading trivia. The callee
/// after it is still the node the checker types.
const JSDOC: &str = r#"<script setup lang="ts">
declare function settle(callback: () => void): void;
declare const loose: any;

/** Settle once mounted. */
settle(() => {});

/** An untyped call stays unsafe. */
loose();
</script>
"#;

#[test]
fn a_leading_jsdoc_comment_keeps_the_callee_typed() {
    if !corsa_available() {
        return;
    }
    let linter = Linter::with_preset(LintPreset::Opinionated);
    for (source, filename, expected) in
        [(JSDOC, "JsDocCall.vue", &[(9, 1, "type/no-unsafe-call")][..])]
    {
        let result = lint_sfc_with_corsa(&linter, source, filename);
        let actual: Vec<_> = result
            .diagnostics
            .iter()
            .filter(|diagnostic| RULES.contains(&diagnostic.rule_name))
            .map(|diagnostic| {
                let (line, column) = line_column(source, diagnostic.start);
                (line, column, diagnostic.rule_name)
            })
            .collect();
        assert_eq!(actual, expected, "{filename}");
    }
}

fn line_column(source: &str, offset: u32) -> (u32, u32) {
    let before = source.get(..offset as usize).unwrap_or_default();
    let line = before.matches('\n').count() as u32 + 1;
    let column = before
        .rsplit('\n')
        .next()
        .map_or(0, |tail| tail.chars().count()) as u32
        + 1;
    (line, column)
}
