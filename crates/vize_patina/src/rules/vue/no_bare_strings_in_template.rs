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
    MarkupBinding, MarkupBindingKind, MarkupContext, MarkupElement, MarkupRule, MarkupText,
};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ElementNode;

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
pub struct NoBareStringsInTemplate;

impl NoBareStringsInTemplate {
    fn check_bindings(ctx: &mut LintContext<'_>, element: &MarkupElement<'_>) {
        // Bare string in a user-facing static attribute. Bound attributes
        // (`:title="..."`) are directives, so they are skipped here.
        element.walk_bindings(&mut |binding| {
            if binding.kind() == MarkupBindingKind::Attribute
                && is_target_attribute(element.tag(), &binding)
                && let Some(value) = binding.static_value()
                && has_bare_string(value)
            {
                ctx.warn_at_with_help(
                    ctx.t("vue/no-bare-strings-in-template.message"),
                    value_range(ctx.source, binding),
                    ctx.t("vue/no-bare-strings-in-template.help"),
                );
            } else if binding.kind() == MarkupBindingKind::Custom
                && binding.is_unqualified_arg_exact("text")
                && let Some(value) = binding.literal_string_value()
                && has_bare_string(value)
            {
                ctx.warn_at_with_help(
                    ctx.t("vue/no-bare-strings-in-template.message"),
                    value_range(ctx.source, binding),
                    ctx.t("vue/no-bare-strings-in-template.help"),
                );
            }
        });
    }

    fn check_text(ctx: &mut LintContext<'_>, text: &MarkupText<'_>) {
        if has_bare_string(text.content()) {
            ctx.warn_at_with_help(
                ctx.t("vue/no-bare-strings-in-template.message"),
                text.range(),
                ctx.t("vue/no-bare-strings-in-template.help"),
            );
        }
    }

    fn check_element_legacy(ctx: &mut LintContext<'_>, element: &MarkupElement<'_>) {
        Self::check_bindings(ctx, element);
    }
}

impl MarkupRule for NoBareStringsInTemplate {
    fn name(&self) -> &'static str {
        META.name
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        // JSX diagnostics follow authored source order: attributes precede
        // children. Vue keeps the legacy visitor's element-exit ordering below.
        if ctx.is_jsx() {
            Self::check_bindings(ctx.lint(), element);
        }
    }

    fn exit_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        if ctx.is_template() {
            Self::check_element_legacy(ctx.lint(), element);
        }
    }

    fn enter_text<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, text: &MarkupText<'a>) {
        Self::check_text(ctx.lint(), text);
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
        Self::check_element_legacy(ctx, &MarkupElement::new(element));
    }
}

fn is_target_attribute(tag: &str, binding: &MarkupBinding<'_>) -> bool {
    COMMON_TARGET_ATTRIBUTES
        .iter()
        .any(|target| binding.is_unqualified_arg_exact(target))
        || (tag == "input" && binding.is_unqualified_arg_exact("placeholder"))
        || (tag == "img" && binding.is_unqualified_arg_exact("alt"))
}

fn has_bare_string(text: &str) -> bool {
    const ALLOWLIST: &[char] = &[
        '(', ')', ',', '.', '&', '+', '-', '=', '*', '/', '#', '%', '!', '?', ':', '[', ']', '{',
        '}', '<', '>', '·', '•', '‐', '–', '—', '−', '|',
    ];
    text.trim()
        .chars()
        .any(|character| !character.is_whitespace() && !ALLOWLIST.contains(&character))
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
