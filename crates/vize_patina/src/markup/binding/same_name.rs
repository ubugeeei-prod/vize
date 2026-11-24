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
        if self.kind() != MarkupBindingKind::Bind || !self.is_authored_directive() {
            return None;
        }
        let argument = self.arg_name()?;
        if !self.is_static_unqualified_arg_exact(argument) {
            return None;
        }
        let spelling = if self.has_value() {
            let value = self.expression()?;
            if !is_simple_identifier(value) || !reads_argument(argument, value) {
                return None;
            }
            SameNameSpelling::Explicit
        } else if self.is_valueless() && is_simple_identifier(&camelize(argument)) {
            SameNameSpelling::Shorthand
        } else {
            return None;
        };
        Some(SameNameBinding { spelling, argument })
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
    use super::reads_argument;

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
