//! Setup bindings a `<style>` block consumes through CSS `v-bind()`.
//!
//! Under `noUnusedLocals`/`noUnusedParameters` the generator narrows its
//! `void <binding>;` anchors to names the template actually references, so a
//! genuinely unused user binding still reports `TS6133`. Template scope is
//! discovered from the template AST and the Croquis expression tables, neither
//! of which sees `<style>`, so a binding referenced *only* from
//! `div { color: v-bind(color); }` looked unreferenced and every such binding
//! was published as `TS6133` — a false positive `vue-tsc` never reports,
//! because the SFC compiler turns those expressions into a real `useCssVars`
//! read (#1876).
//!
//! The expressions come from [`SfcDescriptor::css_vars`], which the SFC parser
//! fills from `extract_css_vars`. That extractor already skips `v-bind(...)`
//! runs inside CSS comments and string literals, so a commented-out `v-bind`
//! contributes nothing here and its binding keeps reporting `TS6133` exactly
//! like `vue-tsc`.

use vize_atelier_sfc::SfcDescriptor;
use vize_carton::{FxHashSet, String as CompactString};

/// Identifiers referenced by every CSS `v-bind()` expression in `descriptor`.
///
/// Expressions are arbitrary JavaScript (`v-bind("height + 'px'")`), so each is
/// parsed for identifiers rather than taken as a name.
pub(super) fn collect_css_var_referenced_names(
    descriptor: &SfcDescriptor<'_>,
) -> FxHashSet<CompactString> {
    let mut names = FxHashSet::default();
    for expression in &descriptor.css_vars {
        for identifier in vize_croquis::drawer::extract_identifiers_oxc(expression.as_ref()) {
            names.insert(CompactString::new(identifier.as_str()));
        }
    }
    names
}

#[cfg(test)]
mod tests {
    use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
    use vize_carton::{FxHashSet, String as CompactString};

    use super::collect_css_var_referenced_names;

    fn names(source: &str) -> Vec<CompactString> {
        let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
        let mut names: Vec<_> = collect_css_var_referenced_names(&descriptor)
            .into_iter()
            .collect();
        names.sort_unstable();
        names
    }

    fn expected(values: &[&str]) -> Vec<CompactString> {
        let mut expected: Vec<_> = values
            .iter()
            .map(|value| CompactString::new(*value))
            .collect();
        expected.sort_unstable();
        expected
    }

    #[test]
    fn plain_and_expression_v_binds_contribute_their_identifiers() {
        assert_eq!(
            names(
                r#"<template><div /></template>
<style scoped>
div {
  color: v-bind(fg);
  background: v-bind('bg');
  height: v-bind("size + 'px'");
  width: v-bind("Math.min(100, ratio) + '%'");
}
</style>
"#
            ),
            expected(&["fg", "bg", "size", "Math", "ratio"])
        );
    }

    #[test]
    fn commented_out_and_string_v_binds_contribute_nothing() {
        assert_eq!(
            names(
                r#"<template><div /></template>
<style scoped>
/* color: v-bind(commented); */
div {
  content: "v-bind(quoted)";
  color: red;
}
</style>
"#
            ),
            Vec::<CompactString>::new()
        );
    }

    #[test]
    fn an_sfc_without_styles_has_no_css_var_names() {
        let descriptor =
            parse_sfc("<template><div /></template>\n", SfcParseOptions::default()).unwrap();
        assert_eq!(
            collect_css_var_referenced_names(&descriptor),
            FxHashSet::default()
        );
    }
}
