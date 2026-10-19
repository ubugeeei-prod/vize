//! vue/no-multiple-template-root
//!
//! Require exactly one rendered root in templates that opt into a single-root
//! contract, including Nuxt layouts, pages, and server components.
//!
//! Vue 3 normally permits fragments, so this rule is deliberately opt-in. Once
//! enabled it mirrors `eslint-plugin-vue@10.9.2`: comments and blank text are
//! ignored, a `v-if`/`v-else-if`/`v-else` chain is one logical root, and any
//! other element or meaningful text is an additional root. A lone `<slot>`,
//! `<template>`, or `v-for` root is also rejected because it may render a shape
//! other than one element.
//!
//! The upstream `disallowComments` option is outside Patina's current
//! severity-only rule configuration. Nuxt uses the default (`false`), so root
//! comments remain accepted here.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l0::ensure_sufficient_stack;
use vize_relief::{ElementNode, PropNode, RootNode, SourceLocation, TemplateChildNode};

static META: RuleMeta = RuleMeta {
    name: "vue/no-multiple-template-root",
    description: "Disallow multiple root nodes in a template",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Disallow template shapes that do not render exactly one element root.
pub struct NoMultipleTemplateRoot;

#[inline]
fn has_directive(element: &ElementNode<'_>, name: &str) -> bool {
    element
        .props
        .iter()
        .any(|prop| matches!(prop, PropNode::Directive(directive) if directive.name == name))
}

/// Recover the start-tag range, matching vue-eslint-parser's reported span.
/// Quoted `>` bytes do not terminate the tag.
fn start_tag_loc(source: &str, element: &ElementNode<'_>) -> SourceLocation {
    let start = element.loc.span.start as usize;
    let end = element.loc.span.end as usize;
    let Some(element_source) = source.get(start..end) else {
        return element.loc.clone();
    };

    let mut quote = None;
    for (relative, &byte) in element_source.as_bytes().iter().enumerate() {
        match (quote, byte) {
            (Some(open), close) if open == close => quote = None,
            (Some(_), _) => {}
            (None, b'\'' | b'"') => quote = Some(byte),
            (None, b'>') => {
                let mut loc = element.loc.clone();
                loc.span.end = (start + relative + 1) as u32;
                return loc;
            }
            (None, _) => {}
        }
    }

    element.loc.clone()
}

/// Recover the whole element range. Relief intentionally keeps the start tag
/// in `ElementNode::loc`, so the closing tag is found after the complete AST
/// subtree. Starting after every child also handles nested elements with the
/// same tag without a lexical nesting counter.
fn full_element_loc(source: &str, element: &ElementNode<'_>) -> SourceLocation {
    let mut loc = element.loc.clone();
    if element.is_self_closing {
        return loc;
    }

    let search_from = element
        .children
        .iter()
        .fold(loc.span.end as usize, |end, child| {
            let child_end = match child {
                TemplateChildNode::Element(child) => {
                    ensure_sufficient_stack(|| full_element_loc(source, child).span.end as usize)
                }
                other => other.loc().span.end as usize,
            };
            end.max(child_end)
        });
    let bytes = source.as_bytes();
    let tag = element.tag.as_bytes();
    let mut cursor = search_from;

    while cursor + tag.len() + 3 <= bytes.len() {
        let Some(relative) = bytes
            .get(cursor..)
            .and_then(|rest| rest.iter().position(|&byte| byte == b'<'))
        else {
            break;
        };
        let start = cursor + relative;
        let name_start = start + 2;
        let name_end = name_start + tag.len();
        if bytes.get(start + 1) == Some(&b'/')
            && bytes
                .get(name_start..name_end)
                .is_some_and(|name| name.eq_ignore_ascii_case(tag))
            && bytes
                .get(name_end)
                .is_some_and(|byte| byte.is_ascii_whitespace() || *byte == b'>')
            && let Some(end_relative) = bytes
                .get(name_end..)
                .and_then(|rest| rest.iter().position(|&byte| byte == b'>'))
        {
            loc.span.end = (name_end + end_relative + 1) as u32;
            return loc;
        }
        cursor = start + 1;
    }

    loc
}

#[inline]
fn is_non_blank(source: &str, loc: &SourceLocation) -> bool {
    source
        .get(loc.span.start as usize..loc.span.end as usize)
        .is_some_and(|text| !text.trim().is_empty())
}

/// Results from the first, allocation-free pass over the template root.
struct RootShape<'ast, 'root> {
    has_element: bool,
    extra_element: Option<&'root ElementNode<'ast>>,
    extra_text: Option<&'root SourceLocation>,
}

impl<'ast, 'root> RootShape<'ast, 'root> {
    fn classify(source: &str, root: &'root RootNode<'ast>) -> Self {
        let mut shape = Self {
            has_element: false,
            extra_element: None,
            extra_text: None,
        };
        let mut chain_open = false;

        for child in &root.children {
            match child {
                TemplateChildNode::Element(element) if !shape.has_element => {
                    shape.has_element = true;
                    chain_open = has_directive(element, "if");
                }
                TemplateChildNode::Element(element)
                    if chain_open && has_directive(element, "else-if") => {}
                TemplateChildNode::Element(element)
                    if chain_open && has_directive(element, "else") =>
                {
                    chain_open = false;
                }
                TemplateChildNode::Element(element) => shape.extra_element = Some(element),
                TemplateChildNode::Comment(_) => {}
                other if is_non_blank(source, other.loc()) => {
                    shape.extra_text = Some(other.loc());
                }
                _ => {}
            }
        }

        shape
    }
}

impl Rule for NoMultipleTemplateRoot {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        let shape = RootShape::classify(ctx.source, root);

        // Upstream gives meaningful text precedence over every element report.
        if let Some(loc) = shape.extra_text {
            ctx.error_with_help(
                ctx.t("vue/no-multiple-template-root.text_root"),
                loc,
                ctx.t("vue/no-multiple-template-root.help"),
            );
            return;
        }

        // Only the last additional element is reported.
        if let Some(element) = shape.extra_element {
            let loc = full_element_loc(ctx.source, element);
            ctx.error_with_help(
                ctx.t("vue/no-multiple-template-root.multiple_root"),
                &loc,
                ctx.t("vue/no-multiple-template-root.help"),
            );
            return;
        }

        if !shape.has_element {
            return;
        }

        // With no additional root, every element is either the first root or a
        // branch in its conditional chain. Each branch receives the root-kind
        // and root-v-for checks, exactly as upstream does.
        for child in &root.children {
            let TemplateChildNode::Element(element) = child else {
                continue;
            };
            let loc = start_tag_loc(ctx.source, element);
            let tag = element.tag;

            if matches!(tag, "template" | "slot") {
                ctx.error_with_help(
                    ctx.t_fmt(
                        "vue/no-multiple-template-root.disallowed_element",
                        &[("tag", tag)],
                    ),
                    &loc,
                    ctx.t("vue/no-multiple-template-root.help"),
                );
            }
            if has_directive(element, "for") {
                ctx.error_with_help(
                    ctx.t("vue/no-multiple-template-root.disallowed_directive"),
                    &loc,
                    ctx.t("vue/no-multiple-template-root.help"),
                );
            }
        }
    }
}

#[cfg(test)]
#[expect(clippy::string_slice, reason = "tests assert by panicking")]
mod tests {
    //! Differential cases captured from `eslint-plugin-vue@10.9.2`.
    //!
    //! Each fragment was wrapped in an SFC `<template>`, linted through ESLint's
    //! flat `Linter` API with `vue-eslint-parser@10.4.1`, then translated back to
    //! template-local byte offsets. Mostly ASCII fixtures make that translation
    //! exact and keep the expected upstream messages and spans reviewable offline;
    //! a multi-byte fixture pins the byte offsets real templates produce.

    use super::NoMultipleTemplateRoot;
    use crate::linter::{LintResult, Linter};
    use crate::preset::LintPreset;
    use crate::rule::RuleRegistry;
    use vize_l0::String;

    const RULE: &str = "vue/no-multiple-template-root";
    const PARSER: &str = "parser/template";
    const MULTIPLE_ROOT: &str = "The template root requires exactly one element.";
    const TEXT_ROOT: &str = "The template root requires an element rather than texts.";
    const V_FOR_ROOT: &str = "The template root disallows 'v-for' directives.";
    type ExpectedFinding = (&'static str, &'static str, u32, u32);
    type DifferentialCase<'a> = (&'a str, &'a [ExpectedFinding]);

    fn linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoMultipleTemplateRoot));
        Linter::with_registry(registry)
    }

    fn reported(result: &LintResult) -> Vec<(&'static str, String, u32, u32)> {
        result
            .diagnostics
            .iter()
            .map(|diagnostic| {
                (
                    diagnostic.rule_name,
                    diagnostic.message.clone(),
                    diagnostic.start,
                    diagnostic.end,
                )
            })
            .collect()
    }

    fn lint(source: &str) -> Vec<(&'static str, String, u32, u32)> {
        reported(&linter().lint_template(source, "test.vue"))
    }

    fn expected(items: &[ExpectedFinding]) -> Vec<(&'static str, String, u32, u32)> {
        items
            .iter()
            .map(|&(rule, message, start, end)| (rule, String::from(message), start, end))
            .collect()
    }

    #[test]
    fn eslint_vue_10_9_2_differential_roots() {
        let cases: &[DifferentialCase<'_>] = &[
            ("<div></div>", &[]),
            ("\n <!-- comment --> <div></div> \n", &[]),
            ("", &[]),
            ("<div></div><div></div>", &[(RULE, MULTIPLE_ROOT, 11, 22)]),
            (
                "<div></div><div></div><div></div>",
                &[(RULE, MULTIPLE_ROOT, 22, 33)],
            ),
            (
                "<div></div><section><div></div></section>",
                &[(RULE, MULTIPLE_ROOT, 11, 41)],
            ),
            (
                "<div></div><section><!-- </section> --><p>x</p></section>",
                &[(RULE, MULTIPLE_ROOT, 11, 57)],
            ),
            ("<slot></slot><div></div>", &[(RULE, MULTIPLE_ROOT, 13, 24)]),
            (r#"<div v-if="a"></div><div v-else></div>"#, &[]),
            (r#"<div v-if="a"></div><!-- gap --><div v-else></div>"#, &[]),
            (r#"<c1 v-if="a"/><c2 v-else-if="b"/><c3 v-else/>"#, &[]),
            (
                r#"<div v-if="a"></div><div v-else></div><p></p>"#,
                &[(RULE, MULTIPLE_ROOT, 38, 45)],
            ),
            (
                r#"<div></div><div v-else-if="a"></div>"#,
                &[(RULE, MULTIPLE_ROOT, 11, 36)],
            ),
            // Multi-byte attribute and text content must not shift the span.
            (
                r#"<div title="日本語">あ</div><section>x</section>"#,
                &[(RULE, MULTIPLE_ROOT, 32, 52)],
            ),
            // An unclosed extra root falls back to its start-tag span.
            (
                "<div></div><section>",
                &[
                    (PARSER, "Element is missing end tag.", 11, 20),
                    (RULE, MULTIPLE_ROOT, 11, 20),
                ],
            ),
        ];

        for &(source, expected_reports) in cases {
            assert_eq!(lint(source), expected(expected_reports), "source: {source}");
        }
    }

    #[test]
    fn eslint_vue_10_9_2_differential_text_roots() {
        let cases: &[DifferentialCase<'_>] = &[
            ("{{ a }}", &[(RULE, TEXT_ROOT, 0, 7)]),
            ("hello", &[(RULE, TEXT_ROOT, 0, 5)]),
            ("<div></div>text<div></div>", &[(RULE, TEXT_ROOT, 11, 15)]),
            ("<slot></slot>text", &[(RULE, TEXT_ROOT, 13, 17)]),
        ];

        for &(source, expected_reports) in cases {
            assert_eq!(lint(source), expected(expected_reports), "source: {source}");
        }
    }

    #[test]
    fn eslint_vue_10_9_2_differential_disallowed_roots() {
        let cases: &[DifferentialCase<'_>] = &[
            (
                "<slot></slot>",
                &[(RULE, "The template root disallows '<slot>' elements.", 0, 6)],
            ),
            (
                "<template></template>",
                &[(
                    RULE,
                    "The template root disallows '<template>' elements.",
                    0,
                    10,
                )],
            ),
            (
                r#"<div v-for="x in xs"></div>"#,
                &[(RULE, V_FOR_ROOT, 0, 21)],
            ),
            (
                r#"<template v-for="x in xs"><li/></template>"#,
                &[
                    (
                        RULE,
                        "The template root disallows '<template>' elements.",
                        0,
                        26,
                    ),
                    (RULE, V_FOR_ROOT, 0, 26),
                ],
            ),
            (
                r#"<div v-if="a" v-for="x in xs"></div><div v-else v-for="y in ys"></div>"#,
                &[(RULE, V_FOR_ROOT, 0, 30), (RULE, V_FOR_ROOT, 36, 64)],
            ),
            (
                r#"<div v-for="a in b" :title="a > b"></div>"#,
                &[(RULE, V_FOR_ROOT, 0, 35)],
            ),
        ];

        for &(source, expected_reports) in cases {
            assert_eq!(lint(source), expected(expected_reports), "source: {source}");
        }
    }

    #[test]
    fn sfc_diagnostics_use_absolute_offsets() {
        let source = "<template>\n  <div>a</div>\n  <div>b</div>\n</template>\n";
        let result = linter().lint_sfc(source, "test.vue");
        assert_eq!(
            reported(&result),
            expected(&[(RULE, MULTIPLE_ROOT, 28, 40)])
        );
        assert_eq!(&source[28..40], "<div>b</div>");
    }

    #[test]
    fn stays_opt_in_for_general_vue_but_can_be_enabled_by_name() {
        assert!(!RuleRegistry::default().has_rule(RULE));
        assert!(!RuleRegistry::with_essential().has_rule(RULE));
        assert!(RuleRegistry::with_opt_in_rules().has_rule(RULE));

        let result = Linter::with_preset(LintPreset::Incremental)
            .with_enabled_rules(Some(vec![String::from(RULE)]))
            .lint_template("<div></div><div></div>", "page.vue");
        assert_eq!(
            reported(&result),
            expected(&[(RULE, MULTIPLE_ROOT, 11, 22)])
        );
    }
}
