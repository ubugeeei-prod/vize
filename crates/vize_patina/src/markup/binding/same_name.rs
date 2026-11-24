//! Vue's same-name `v-bind`: `:foo` reads `foo`, exactly as `:foo="foo"` does.
//!
//! The compiler expands a valueless bind with a static argument to the
//! camelized argument. A binding is same-name when that expansion is an
//! identifier and the authored value, if any, is that identifier.

use super::{MarkupBinding, MarkupBindingInner, MarkupBindingKind};
use vize_l0::{String, camelize, is_simple_identifier};

/// How a same-name binding is written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SameNameSpelling {
    /// `:foo`, the valueless Vue 3.4 shorthand.
    Shorthand,
    /// `:foo="foo"`, the value written out.
    Explicit,
}

/// A `v-bind` whose value reads its own static argument.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SameNameBinding<'a> {
    /// The authored spelling.
    pub spelling: SameNameSpelling,
    argument: &'a str,
}

impl SameNameBinding<'_> {
    /// The identifier both spellings read: the camelized argument.
    pub fn operand(&self) -> String {
        camelize(self.argument)
    }
}

impl<'a> MarkupBinding<'a> {
    /// Classify this binding as a same-name `v-bind`, when it is one.
    ///
    /// Modifiers do not change the value a binding reads, so `:foo.prop` and
    /// `.foo` classify as `:foo` does. A dynamic argument has no name to read.
    pub fn same_name_binding(&self) -> Option<SameNameBinding<'a>> {
        if self.kind() != MarkupBindingKind::Bind {
            return None;
        }
        self.same_name_as(if self.has_value() {
            SameNameSpelling::Explicit
        } else {
            SameNameSpelling::Shorthand
        })
    }

    /// This binding as a same-name `v-bind` written as `spelling`.
    ///
    /// A binding written the other way is answered from its value alone, so a
    /// rule that looks for one spelling does none of the other's work.
    pub fn same_name_binding_written(
        &self,
        spelling: SameNameSpelling,
    ) -> Option<SameNameBinding<'a>> {
        if self.kind() != MarkupBindingKind::Bind
            || self.has_value() != (spelling == SameNameSpelling::Explicit)
        {
            return None;
        }
        self.same_name_as(spelling)
    }

    /// The one detection: a static argument whose camelized form is the
    /// identifier this `v-bind`, written as `spelling`, reads.
    fn same_name_as(&self, spelling: SameNameSpelling) -> Option<SameNameBinding<'a>> {
        let argument = self.arg_name()?;
        if !self.is_static_unqualified_arg_exact(argument) {
            return None;
        }
        let reads_own_argument = match spelling {
            SameNameSpelling::Explicit => {
                let value = self.expression()?;
                is_simple_identifier(value) && reads_argument(argument, value)
            }
            SameNameSpelling::Shorthand => {
                self.is_valueless() && is_simple_identifier(&camelize(argument))
            }
        };
        (reads_own_argument && self.is_authored_directive())
            .then_some(SameNameBinding { spelling, argument })
    }

    /// Whether a Vue template spells this directive. An attribute lowered
    /// from JSX has neither same-name spelling.
    fn is_authored_directive(&self) -> bool {
        match self.inner {
            MarkupBindingInner::ReliefDirective(node) => node.raw_name.is_some(),
            MarkupBindingInner::L2Binding(binding) => binding.surface.is_some(),
            MarkupBindingInner::Surface { .. } => true,
            MarkupBindingInner::ReliefAttribute(_)
            | MarkupBindingInner::L2Attribute { .. }
            | MarkupBindingInner::Jsx { .. } => false,
        }
    }

    /// Whether the bind was written without `=`. An empty value (`:foo=""`)
    /// is written, and is not the shorthand.
    fn is_valueless(&self) -> bool {
        match self.inner {
            MarkupBindingInner::ReliefDirective(node) => node.shorthand,
            MarkupBindingInner::L2Binding(binding) => {
                binding.surface.is_some_and(|attr| attr.value.is_none())
            }
            MarkupBindingInner::Surface { attr, .. } => attr.value.is_none(),
            MarkupBindingInner::ReliefAttribute(_)
            | MarkupBindingInner::L2Attribute { .. }
            | MarkupBindingInner::Jsx { .. } => false,
        }
    }
}

/// Whether `value` is the camelized `argument`. Only a hyphen changes under
/// camelization, so the common spellings compare without allocating.
fn reads_argument(argument: &str, value: &str) -> bool {
    argument == value || (argument.contains('-') && camelize(argument).as_str() == value)
}

#[cfg(test)]
mod tests {
    use super::{SameNameSpelling, reads_argument};
    use crate::context::LintContext;
    use crate::ir::TemplateSyntax;
    use crate::markup::{
        MarkupBinding, MarkupContext, MarkupDocument, MarkupElement, MarkupRule, MarkupTraversal,
    };
    use std::cell::RefCell;
    use vize_l0::Allocator;

    /// Records each binding's classification, after checking that the two
    /// spelling-specific queries give the same answer.
    #[derive(Default)]
    struct Classified(RefCell<Vec<(std::string::String, Option<SameNameSpelling>)>>);

    impl MarkupRule for Classified {
        fn name(&self) -> &'static str {
            "test/same-name"
        }

        fn traversal(&self) -> MarkupTraversal {
            MarkupTraversal::Bindings
        }

        fn enter_binding<'a>(
            &self,
            _ctx: &mut MarkupContext<'_, 'a>,
            _element: &MarkupElement<'a>,
            binding: &MarkupBinding<'a>,
        ) {
            let classified = binding.same_name_binding().map(|same| same.spelling);
            for spelling in [SameNameSpelling::Shorthand, SameNameSpelling::Explicit] {
                assert_eq!(
                    binding
                        .same_name_binding_written(spelling)
                        .map(|same| same.spelling),
                    classified.filter(|found| *found == spelling)
                );
            }
            self.0.borrow_mut().push((
                binding.arg_name().unwrap_or_default().to_owned(),
                classified,
            ));
        }
    }

    #[test]
    fn one_detection_answers_the_classifier_and_both_spelling_queries() {
        let source = r#"<div :a="a" :b-c="bC" :d :e-f :g.prop="g" .h="h" .i v-bind:j="j" v-bind:k :l="m" :n="n()" :o="" :[p]="p" :[q] v-bind="r" :1="1" s="s" @t="t" />"#;
        let allocator = Allocator::with_capacity(source.len() * 4 + 1024);
        let (root, _errors) = vize_armature::Parser::new(&allocator, source).parse();
        let document = MarkupDocument::new(&root, TemplateSyntax::Vue);
        let mut lint = LintContext::new(&allocator, source, "test.vue");
        let mut ctx = MarkupContext::new(&mut lint, &document);
        let classified = Classified::default();
        document.visit_with(&classified, &mut ctx);

        let explicit = Some(SameNameSpelling::Explicit);
        let shorthand = Some(SameNameSpelling::Shorthand);
        assert_eq!(
            classified.0.into_inner(),
            [
                ("a", explicit),
                ("b-c", explicit),
                ("d", shorthand),
                ("e-f", shorthand),
                ("g", explicit),
                ("h", explicit),
                ("i", shorthand),
                ("j", explicit),
                ("k", shorthand),
                ("l", None),
                ("n", None),
                ("o", None),
                ("p", None),
                ("q", None),
                ("", None),
                ("1", None),
                ("s", None),
                ("t", None),
            ]
            .map(|(argument, spelling)| (argument.to_owned(), spelling))
        );
    }

    #[test]
    fn a_value_reads_the_argument_only_as_its_camelized_name() {
        for (argument, value, expected) in [
            ("foo", "foo", true),
            ("fooBar", "fooBar", true),
            ("foo-bar", "fooBar", true),
            ("aria-label", "ariaLabel", true),
            ("foo_bar", "foo_bar", true),
            ("Foo-bar", "FooBar", true),
            ("foo", "bar", false),
            ("fooBar", "foobar", false),
            ("foo-bar", "foo_bar", false),
            ("foo_bar", "fooBar", false),
            ("foo-bar", "foobar", false),
        ] {
            assert_eq!(
                reads_argument(argument, value),
                expected,
                "{argument} {value}"
            );
        }
    }
}
