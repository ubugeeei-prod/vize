//! vue/no-bare-strings-in-template
//!
//! Disallow raw (bare) human-readable text in the template that should be
//! internationalized.
//!
//! Raw text written directly in a template — whether as an element's text
//! content or inside a user-facing attribute such as `title`, `alt`,
//! `placeholder`, or `aria-label` — bakes a single language into the markup. In
//! an internationalized application that text should instead flow through a
//! translation function (e.g. `{{ $t('...') }}` / `:title="$t('...')"`).
//!
//! The eslint-plugin-vue default punctuation allowlist is honored, while other
//! non-whitespace text (including numbers) is reported. Mustache interpolations
//! `{{ }}` (already dynamic) and bound attributes (`:title="..."`) are skipped.
//! Text nodes inside `<script>` / `<style>` elements are checked just like the
//! upstream rule's unqualified `VText` visitor.
//!
//! Mirrors eslint's `vue/no-bare-strings-in-template` (with its default target
//! attributes). It is opt-in: it only runs when a project explicitly enables the
//! i18n rule set.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div>hello</div>
//! <img alt="a cat" />
//! <input placeholder="Search" />
//! <button title="Close">x</button>
//! ```
//!
//! ### Valid
//! ```vue
//! <div>{{ $t('hello') }}</div>
//! <img :alt="$t('cat')" />
//! <div>-</div>
//! <button :title="$t('close')">{{ $t('x') }}</button>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::ir::ByteRange;
use crate::markup::{
    MarkupBinding, MarkupBindingKind, MarkupContext, MarkupElement, MarkupNode, MarkupRule,
    MarkupText, MarkupTraversal,
};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use crate::rules::a11y::helpers::string_literal_value;
use regex::Regex;
use vize_l0::{String, cstr};
use vize_relief::{ElementNode, RootNode, TemplateChildNode};

static META: RuleMeta = RuleMeta {
    name: "vue/no-bare-strings-in-template",
    description: "Disallow raw human-readable text in the template that should be internationalized",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Attribute names whose raw string value is user-facing and should be
/// internationalized. Matches eslint `vue/no-bare-strings-in-template`'s default
/// target attributes.
const COMMON_TARGET_ATTRIBUTES: &[&str] = &[
    "title",
    "aria-label",
    "aria-placeholder",
    "aria-roledescription",
    "aria-valuetext",
];

/// Disallow raw human-readable text directly in the template.
pub struct NoBareStringsInTemplate {
    allowlist: BareStringAllowlist,
}

enum BareStringAllowlist {
    Default,
    Custom {
        literals: Option<Regex>,
        patterns: Vec<Regex>,
    },
}

#[expect(
    non_upper_case_globals,
    reason = "the unit-like rule value intentionally shares the rule type name"
)]
/// Default `vue/no-bare-strings-in-template` rule using the upstream allowlist.
pub const NoBareStringsInTemplate: NoBareStringsInTemplate = NoBareStringsInTemplate {
    allowlist: BareStringAllowlist::Default,
};

impl Default for NoBareStringsInTemplate {
    fn default() -> Self {
        NoBareStringsInTemplate
    }
}

impl NoBareStringsInTemplate {
    pub(crate) fn with_allowlist(allowlist: Option<Vec<vize_l0::String>>) -> Self {
        let Some(allowlist) = allowlist else {
            return Self::default();
        };
        Self {
            allowlist: compile_allowlist(&allowlist),
        }
    }

    fn check_bindings(&self, ctx: &mut LintContext<'_>, element: &MarkupElement<'_>) {
        // Bare string in a user-facing static attribute. Bound attributes
        // (`:title="..."`) are directives, so they are skipped here.
        element.walk_bindings(&mut |binding| {
            if binding.kind() == MarkupBindingKind::Attribute
                && is_target_attribute(element.tag(), &binding)
                && let Some(value) = binding.static_value()
                && self.has_bare_string(value)
            {
                ctx.warn_at_with_help(
                    ctx.t("vue/no-bare-strings-in-template.message"),
                    value_range(ctx.source, binding),
                    ctx.t("vue/no-bare-strings-in-template.help"),
                );
            } else if binding.kind() == MarkupBindingKind::Custom
                && binding.is_unqualified_arg_exact("text")
                && let Some(value) = binding.literal_string_value().or_else(|| {
                    binding
                        .expression()
                        .and_then(parenthesized_string_literal_value)
                })
                && self.has_bare_string(value)
            {
                ctx.warn_at_with_help(
                    ctx.t("vue/no-bare-strings-in-template.message"),
                    value_range(ctx.source, binding),
                    ctx.t("vue/no-bare-strings-in-template.help"),
                );
            }
        });
    }

    fn check_text(&self, ctx: &mut LintContext<'_>, text: &MarkupText<'_>) {
        if self.has_bare_string(text.content()) {
            ctx.warn_at_with_help(
                ctx.t("vue/no-bare-strings-in-template.message"),
                text.range(),
                ctx.t("vue/no-bare-strings-in-template.help"),
            );
        }
    }

    fn check_element_legacy(&self, ctx: &mut LintContext<'_>, element: &MarkupElement<'_>) {
        // The legacy Relief visitor has no text callback. Keep its direct-child
        // projection here while the unified markup lane uses `enter_text`.
        element.walk_children(&mut |child| {
            if let MarkupNode::Text(text) = child {
                self.check_text(ctx, &text);
            }
        });
        self.check_bindings(ctx, element);
    }

    fn has_bare_string(&self, text: &str) -> bool {
        let text = trim_javascript(text);
        match &self.allowlist {
            BareStringAllowlist::Default => default_has_bare_string(text),
            BareStringAllowlist::Custom { literals, patterns } => {
                let mut remaining: String = literals.as_ref().map_or_else(
                    || text.into(),
                    |literals| literals.replace_all(text, "").into_owned().into(),
                );
                for pattern in patterns {
                    remaining = pattern.replace_all(&remaining, "").into_owned().into();
                }
                !trim_javascript(&remaining).is_empty()
            }
        }
    }
}

impl MarkupRule for NoBareStringsInTemplate {
    fn name(&self) -> &'static str {
        META.name
    }

    fn traversal(&self) -> MarkupTraversal {
        MarkupTraversal::ElementsAndText
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        // JSX diagnostics follow authored source order: attributes precede
        // children. Vue keeps the legacy visitor's element-exit ordering below.
        if ctx.is_jsx() {
            self.check_bindings(ctx.lint(), element);
        }
    }

    fn exit_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        if ctx.is_template() {
            self.check_element_legacy(ctx.lint(), element);
        }
    }

    fn enter_text<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, text: &MarkupText<'a>) {
        if ctx.is_jsx() || ctx.current_element().is_none() {
            self.check_text(ctx.lint(), text);
        }
    }
}

impl Rule for NoBareStringsInTemplate {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn jsx_needs_lowering(&self) -> bool {
        // The legacy JSX fallback turns string expression children such as
        // `{'Hello'}` into text nodes. Keep JSX on the lowered markup lane so
        // the i18n rule does not silently stop reporting those authored strings.
        true
    }

    fn exit_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        self.check_element_legacy(ctx, &MarkupElement::new(element));
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        // Root-level text has no owning element, so the legacy element-exit
        // visitor cannot observe it.
        for child in &root.children {
            let TemplateChildNode::Text(text) = child else {
                continue;
            };
            if self.has_bare_string(text.content) {
                ctx.warn_at_with_help(
                    ctx.t("vue/no-bare-strings-in-template.message"),
                    ByteRange::new(text.loc.span.start, text.loc.span.end),
                    ctx.t("vue/no-bare-strings-in-template.help"),
                );
            }
        }
    }
}

fn is_target_attribute(tag: &str, binding: &MarkupBinding<'_>) -> bool {
    COMMON_TARGET_ATTRIBUTES
        .iter()
        .any(|target| binding.is_unqualified_arg_exact(target))
        || (tag == "input" && binding.is_unqualified_arg_exact("placeholder"))
        || (tag == "img" && binding.is_unqualified_arg_exact("alt"))
}

fn default_has_bare_string(text: &str) -> bool {
    const ALLOWLIST: &[char] = &[
        '(', ')', ',', '.', '&', '+', '-', '=', '*', '/', '#', '%', '!', '?', ':', '[', ']', '{',
        '}', '<', '>', '·', '•', '‐', '–', '—', '−', '|',
    ];
    text.chars()
        .any(|character| !is_javascript_whitespace(character) && !ALLOWLIST.contains(&character))
}

fn compile_allowlist(entries: &[String]) -> BareStringAllowlist {
    let mut literals = Vec::new();
    let mut patterns = Vec::new();
    for entry in entries {
        if let Some(pattern) = compile_pattern(entry) {
            patterns.push(pattern);
        } else if !entry.is_empty() {
            literals.push(entry.as_str());
        }
    }
    literals
        .sort_unstable_by(|left, right| right.len().cmp(&left.len()).then_with(|| left.cmp(right)));
    let literals = (!literals.is_empty())
        .then(|| {
            literals
                .into_iter()
                .map(regex::escape)
                .collect::<Vec<_>>()
                .join("|")
        })
        .and_then(|pattern| Regex::new(&pattern).ok());
    BareStringAllowlist::Custom { literals, patterns }
}

fn compile_pattern(entry: &str) -> Option<Regex> {
    if let Some((pattern, flags)) = javascript_regex(entry) {
        let inline_flags: String = flags
            .chars()
            .filter(|flag| matches!(flag, 'i' | 'm' | 's'))
            .collect();
        let candidate = if inline_flags.is_empty() {
            cstr!("(?:{pattern})")
        } else {
            cstr!("(?{inline_flags}:{pattern})")
        };
        if let Ok(pattern) = Regex::new(&candidate) {
            return Some(pattern);
        }
    }
    None
}

fn trim_javascript(value: &str) -> &str {
    value.trim_matches(is_javascript_whitespace)
}

fn is_javascript_whitespace(character: char) -> bool {
    matches!(
        character,
        '\u{0009}'..='\u{000d}'
            | '\u{0020}'
            | '\u{00a0}'
            | '\u{1680}'
            | '\u{2000}'..='\u{200a}'
            | '\u{2028}'
            | '\u{2029}'
            | '\u{202f}'
            | '\u{205f}'
            | '\u{3000}'
            | '\u{feff}'
    )
}

fn parenthesized_string_literal_value(mut value: &str) -> Option<&str> {
    value = value.trim();
    while value.starts_with('(') && value.ends_with(')') {
        value = value.get(1..value.len() - 1)?.trim();
    }
    string_literal_value(value)
}

fn javascript_regex(value: &str) -> Option<(&str, &str)> {
    if !value.starts_with('/') {
        return None;
    }
    let end = value.rfind('/')?;
    if end == 0 {
        return None;
    }
    let flags = value.get(end + 1..)?;
    if !flags
        .chars()
        .all(|flag| matches!(flag, 'd' | 'g' | 'i' | 'm' | 's' | 'u' | 'y'))
    {
        return None;
    }
    Some((value.get(1..end)?, flags))
}

fn value_range(source: &str, binding: MarkupBinding<'_>) -> ByteRange {
    let range = binding.range();
    let Some(authored) = source.get(range.start as usize..range.end as usize) else {
        return range;
    };
    let Some(equal) = authored.find('=') else {
        return range;
    };
    let Some(value) = authored.get(equal + 1..) else {
        return range;
    };
    let whitespace = value.bytes().take_while(u8::is_ascii_whitespace).count();
    ByteRange::new(range.start + (equal + 1 + whitespace) as u32, range.end)
}

#[cfg(test)]
mod allowlist_tests;

#[cfg(test)]
mod tests {
    use super::NoBareStringsInTemplate;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoBareStringsInTemplate));
        Linter::with_registry(registry)
    }

    #[test]
    fn reports_bare_text_content() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>hello</div>"#, "App.vue");
        assert_eq!(result.warning_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn reports_bare_text_in_nested_element() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div><span>Save</span></div>"#, "App.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn reports_bare_text_at_the_template_root() {
        let result = create_linter().lint_template(
            "Base Empty <CompSwitch v-model:value=\"loadData\" />",
            "App.vue",
        );
        assert_eq!(result.warning_count, 1, "{:?}", result.diagnostics);
    }

    #[test]
    fn reports_non_ascii_bare_text() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>こんにちは</div>"#, "App.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn reports_bare_alt_attribute() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<img alt="a cat" />"#, "App.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn reports_bare_title_attribute() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<button title="Close">x</button>"#, "App.vue");
        // Both the title attribute and the "x"-free text? "x" is alphabetic, so
        // the button text "x" also reports. Expect 2.
        assert_eq!(result.warning_count, 2);
    }

    #[test]
    fn reports_bare_placeholder_attribute() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<input placeholder="Search" />"#, "App.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn reports_bare_aria_label_attribute() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div aria-label="Menu"></div>"#, "App.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn allows_interpolation() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>{{ $t('hello') }}</div>"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_bound_attribute() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<img :alt="$t('cat')" />"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_whitespace_only_text() {
        let linter = create_linter();
        let result = linter.lint_template("<div>   </div>", "App.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_punctuation_and_symbol_only_text() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>-</div>"#, "App.vue");
        assert_eq!(result.warning_count, 0);
        let result = linter.lint_template(r#"<span>|</span>"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn reports_number_only_text() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>123</div>"#, "App.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn limits_alt_and_placeholder_to_the_upstream_default_tags() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div alt="copy" placeholder="copy"></div><img alt="copy"><input placeholder="copy">"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 2);
    }

    #[test]
    fn reports_the_static_attribute_value_range() {
        let linter = create_linter();
        let source = r#"<img alt="a cat">"#;
        let result = linter.lint_template(source, "App.vue");
        assert_eq!(result.warning_count, 1);
        let start = source.find('"').unwrap() as u32;
        assert_eq!(result.diagnostics[0].start, start);
        assert_eq!(result.diagnostics[0].end, source.len() as u32 - 1);
    }

    #[test]
    fn reports_static_v_text_values() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<p v-text="'Save'"></p>"#, "App.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn allows_non_target_attribute() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div class="container"></div>"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_id_attribute() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<input type="text" name="q" />"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }
}
