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
//! To stay conservative this rule only reports text that contains an actual
//! letter (any Unicode alphabetic character). Whitespace-, punctuation-, symbol-
//! and number-only text (`-`, `|`, `/`, `123`, `&nbsp;`, …) is ignored, as are
//! mustache interpolations `{{ }}` (already dynamic), bound attributes
//! (`:title="..."`), and the content of `<script>` / `<style>` blocks.
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
//! <div>123</div>
//! <button :title="$t('close')">{{ $t('x') }}</button>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{ElementNode, PropNode, TemplateChildNode};

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
const TARGET_ATTRIBUTES: &[&str] = &[
    "alt",
    "title",
    "placeholder",
    "aria-label",
    "aria-placeholder",
    "aria-roledescription",
    "aria-valuetext",
];

/// Disallow raw human-readable text directly in the template.
pub struct NoBareStringsInTemplate;

impl Rule for NoBareStringsInTemplate {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        // The content of <script>/<style> is code/CSS, never user-facing copy.
        if is_raw_text_element(element.tag.as_str()) {
            return;
        }

        // Bare text content of the element (direct text children only; nested
        // elements are visited on their own).
        for child in element.children.iter() {
            if let TemplateChildNode::Text(text) = child
                && has_bare_string(text.content.as_str())
            {
                ctx.warn_with_help(
                    ctx.t("vue/no-bare-strings-in-template.message"),
                    &text.loc,
                    ctx.t("vue/no-bare-strings-in-template.help"),
                );
            }
        }

        // Bare string in a user-facing static attribute. Bound attributes
        // (`:title="..."`) are directives, so they are skipped here.
        for prop in element.props.iter() {
            if let PropNode::Attribute(attr) = prop
                && is_target_attribute(attr.name.as_str())
                && let Some(value) = &attr.value
                && has_bare_string(value.content.as_str())
            {
                ctx.warn_with_help(
                    ctx.t("vue/no-bare-strings-in-template.message"),
                    &attr.loc,
                    ctx.t("vue/no-bare-strings-in-template.help"),
                );
            }
        }
    }
}

/// Whether `tag` is an element whose text content is raw (non-markup) data and
/// therefore never user-facing copy.
fn is_raw_text_element(tag: &str) -> bool {
    tag.eq_ignore_ascii_case("script") || tag.eq_ignore_ascii_case("style")
}

/// Whether `name` is one of the user-facing attributes whose literal value
/// should be internationalized.
fn is_target_attribute(name: &str) -> bool {
    TARGET_ATTRIBUTES
        .iter()
        .any(|target| name.eq_ignore_ascii_case(target))
}

/// Whether `text` contains human-readable copy: at least one alphabetic
/// character (any language). Whitespace-, punctuation-, symbol- and number-only
/// text is treated as non-translatable and ignored.
fn has_bare_string(text: &str) -> bool {
    text.chars().any(char::is_alphabetic)
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
    fn allows_number_only_text() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div>123</div>"#, "App.vue");
        assert_eq!(result.warning_count, 0);
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
