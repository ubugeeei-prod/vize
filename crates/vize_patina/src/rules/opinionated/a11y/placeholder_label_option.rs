//! a11y/placeholder-label-option
//!
//! Require a placeholder label option in a required `<select>`.
//!
//! Follows markuplint's `placeholder-label-option` and the HTML spec's
//! [placeholder label option](https://html.spec.whatwg.org/multipage/form-elements.html#placeholder-label-option):
//! a `<select>` that is `required`, not `multiple` and has a display size of 1
//! must start with an `<option>` whose value is the empty string, as a direct
//! child. That empty choice makes `required` meaningful: the select starts with
//! no value until the user picks one. The option may be `disabled` or
//! `hidden`, but need not be. A select that is not `required` has no
//! placeholder label option, so an empty first option there is an ordinary
//! choice (such as "Any"). Options produced by `v-for`, `v-if` or bound values
//! are not judged.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <template>
//!   <select required>
//!     <option value="jp">Japan</option>
//!     <option value="fr">France</option>
//!   </select>
//! </template>
//! ```
//!
//! ### Valid
//! ```vue
//! <template>
//!   <select required>
//!     <option value="">Choose a country</option>
//!     <option value="jp">Japan</option>
//!   </select>
//! </template>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::ir::ByteRange;
use crate::markup::{MarkupBindingKind, MarkupContext, MarkupElement, MarkupNode, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ElementNode;

use crate::rules::a11y::helpers::string_literal_value;

static META: RuleMeta = RuleMeta {
    name: "a11y/placeholder-label-option",
    description: "Require a placeholder label option in a required select",
    category: RuleCategory::Accessibility,
    fixable: false,
    default_severity: Severity::Warning,
};

#[derive(Default)]
pub struct PlaceholderLabelOption;

/// The first entry of a select's list of options, as far as it is known.
enum FirstOption<'a> {
    /// The first option is a direct child of the select.
    Direct(MarkupElement<'a>),
    /// The first option sits in an `<optgroup>`, so it cannot be a placeholder.
    Grouped(ByteRange),
    /// No option at all.
    Missing,
    /// Directives, components, slots or JSX expressions decide the options.
    Unknown,
}

/// What a binding says about one attribute.
enum Attr<'a> {
    Absent,
    Static(Option<&'a str>),
    Literal(&'a str),
    Dynamic,
}

impl PlaceholderLabelOption {
    fn attribute<'a>(element: &MarkupElement<'a>, name: &str) -> Attr<'a> {
        let mut found = Attr::Absent;
        element.walk_bindings(&mut |binding| match binding.kind() {
            MarkupBindingKind::Attribute if binding.is_unqualified_arg_exact(name) => {
                if matches!(found, Attr::Absent) {
                    found = Attr::Static(binding.static_value());
                }
            }
            MarkupBindingKind::Bind if binding.arg_name().is_none() => found = Attr::Dynamic,
            MarkupBindingKind::Bind if binding.is_unqualified_arg_exact(name) => {
                found = match binding.expression().and_then(string_literal_value) {
                    Some(value) => Attr::Literal(value),
                    None => Attr::Dynamic,
                };
            }
            _ => {}
        });
        found
    }

    fn is_structural(element: &MarkupElement<'_>) -> bool {
        let mut structural = false;
        for name in ["if", "else-if", "else", "for"] {
            element.walk_authored_directive_ranges(name, &mut |_| structural = true);
        }
        structural
    }

    /// `required`, not `multiple` and a display size of 1, all statically.
    fn has_placeholder_label_option_slot(select: &MarkupElement<'_>) -> bool {
        let required = matches!(Self::attribute(select, "required"), Attr::Static(_));
        let single = matches!(Self::attribute(select, "multiple"), Attr::Absent);
        let size_one = match Self::attribute(select, "size") {
            Attr::Absent => true,
            Attr::Static(Some(size)) | Attr::Literal(size) => size.trim() == "1",
            Attr::Static(None) | Attr::Dynamic => false,
        };
        required && single && size_one
    }

    fn first_option<'a>(
        parent: &MarkupElement<'a>,
        transparent_fragments: bool,
    ) -> FirstOption<'a> {
        let mut first = None;
        parent.walk_children(&mut |child| {
            if first.is_some() {
                return;
            }
            match child {
                MarkupNode::Comment(_) | MarkupNode::Text(_) | MarkupNode::Interpolation(_) => {}
                MarkupNode::Element(element) if Self::is_structural(&element) => {
                    first = Some(FirstOption::Unknown);
                }
                MarkupNode::Element(element) if element.is_unqualified_tag_exact("option") => {
                    first = Some(FirstOption::Direct(element));
                }
                MarkupNode::Element(element) if element.is_unqualified_tag_exact("optgroup") => {
                    first = Some(match Self::first_option(&element, transparent_fragments) {
                        FirstOption::Direct(option) => FirstOption::Grouped(option.range()),
                        FirstOption::Missing => return,
                        other => other,
                    });
                }
                MarkupNode::Element(element)
                    if transparent_fragments && element.tag().is_empty() =>
                {
                    match Self::first_option(&element, transparent_fragments) {
                        FirstOption::Missing => {}
                        other => first = Some(other),
                    }
                }
                _ => first = Some(FirstOption::Unknown),
            }
        });
        first.unwrap_or(FirstOption::Missing)
    }

    /// Whether the option's value is the empty string; `None` when unknown.
    fn has_empty_value(option: &MarkupElement<'_>) -> Option<bool> {
        match Self::attribute(option, "value") {
            Attr::Static(value) => return Some(value.is_none_or(str::is_empty)),
            Attr::Literal(value) => return Some(value.is_empty()),
            Attr::Dynamic => return None,
            Attr::Absent => {}
        }
        // Without `value`, the value is the text, with whitespace collapsed.
        let mut empty = true;
        let mut known = true;
        option.walk_children(&mut |child| match child {
            MarkupNode::Text(text) => empty &= text.content().trim().is_empty(),
            MarkupNode::Comment(_) => {}
            _ => known = false,
        });
        known.then_some(empty)
    }

    fn check_element(
        ctx: &mut LintContext<'_>,
        element: &MarkupElement<'_>,
        transparent_fragments: bool,
    ) {
        if element.is_component()
            || !element.is_unqualified_tag_exact("select")
            || !Self::has_placeholder_label_option_slot(element)
        {
            return;
        }
        let range = match Self::first_option(element, transparent_fragments) {
            FirstOption::Direct(option) => match Self::has_empty_value(&option) {
                Some(false) => option.range(),
                Some(true) | None => return,
            },
            FirstOption::Grouped(range) => range,
            FirstOption::Missing | FirstOption::Unknown => return,
        };
        let message = ctx.t("a11y/placeholder-label-option.message");
        let help = ctx.t("a11y/placeholder-label-option.help");
        ctx.warn_at_with_help(message, range, help);
    }
}

impl MarkupRule for PlaceholderLabelOption {
    fn name(&self) -> &'static str {
        META.name
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        let transparent_fragments = ctx.is_jsx();
        Self::check_element(ctx.lint(), element, transparent_fragments);
    }
}

impl Rule for PlaceholderLabelOption {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        Self::check_element(ctx, &MarkupElement::new(element), false);
    }
}

#[cfg(test)]
mod tests {
    use super::PlaceholderLabelOption;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn warnings(template: &str) -> usize {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(PlaceholderLabelOption));
        let result = Linter::with_registry(registry).lint_template(template, "test.vue");
        result.warning_count
    }

    #[test]
    fn selects_that_are_not_required_have_no_placeholder() {
        for template in [
            r#"<select aria-label="Fruit"><option value="">Any</option><option value="apple">Apple</option></select>"#,
            r#"<select><option value="a">A</option></select>"#,
            r#"<select required multiple><option value="a">A</option></select>"#,
            r#"<select required size="3"><option value="a">A</option></select>"#,
            r#"<select :required="strict"><option value="a">A</option></select>"#,
        ] {
            assert_eq!(warnings(template), 0, "{template}");
        }
    }

    #[test]
    fn a_required_select_starting_with_an_empty_option_is_valid() {
        for template in [
            r#"<select required><option value="">Choose</option><option value="a">A</option></select>"#,
            r#"<select required><option value="" disabled>Choose</option></select>"#,
            r#"<select required size="1"><option value="" hidden>Choose</option></select>"#,
            r#"<select required><option value>Choose</option></select>"#,
            r#"<select required><option></option><option value="a">A</option></select>"#,
            r#"<select required><option :value="''">Choose</option></select>"#,
            r#"<select required>
  <!-- prompt -->
  <option value="">Choose</option>
</select>"#,
        ] {
            assert_eq!(warnings(template), 0, "{template}");
        }
    }

    #[test]
    fn a_required_select_without_a_placeholder_is_reported() {
        for template in [
            r#"<select required><option value="a">A</option><option value="b">B</option></select>"#,
            r#"<select required><option>Choose</option></select>"#,
            r#"<select required><option :value="'a'">A</option></select>"#,
            r#"<select required><optgroup label="G"><option value="">Choose</option></optgroup></select>"#,
        ] {
            assert_eq!(warnings(template), 1, "{template}");
        }
    }

    #[test]
    fn options_decided_at_runtime_are_not_judged() {
        for template in [
            r#"<select required><option v-for="o in options" :value="o">{{ o }}</option></select>"#,
            r#"<select required><option v-if="prompt" value="a">A</option></select>"#,
            r#"<select required><option :value="first">First</option></select>"#,
            r#"<select required><option>{{ label }}</option></select>"#,
            r#"<select required><slot /></select>"#,
            r#"<select required><MyOption value="a" /></select>"#,
            r#"<select required v-bind="attrs"><option value="a">A</option></select>"#,
            r#"<select required></select>"#,
        ] {
            assert_eq!(warnings(template), 0, "{template}");
        }
    }
}
