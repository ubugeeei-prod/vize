//! Attribute lowering specific to Babel JSX compatibility mode.

use oxc_span::Span;
use vize_carton::{Box, String};
use vize_relief::{DirectiveNode, PropNode, SourceLocation};

use crate::lower::Lowerer;

impl<'a, 'm, 's> Lowerer<'a, 'm, 's> {
    /// Normalize the camel-cased SVG prop exactly as Babel does (#3391).
    pub(super) fn compat_attribute_name(&self, name_span: Span) -> String {
        let authored = self.mapper().slice(name_span);
        String::from(if self.uses_babel_compat() && authored == "xlinkHref" {
            "xlink:href"
        } else {
            authored
        })
    }

    /// A valueless JSX attribute is a boolean `true` in Babel's JSX
    /// semantics. Native Vize lowering deliberately keeps its established
    /// template-style empty-string value.
    pub(super) fn valueless_attr(
        &self,
        name: String,
        name_span: Span,
        name_loc: SourceLocation,
        loc: SourceLocation,
    ) -> PropNode<'a> {
        if !self.uses_babel_compat() {
            return self.boolean_attr(name, name_loc, loc);
        }

        let mut directive = DirectiveNode::new(self.bump(), "bind", loc);
        directive.arg = Some(self.static_expr(&name, name_span));
        directive.exp = Some(self.constant_expr("true", name_span));
        PropNode::Directive(Box::new_in(directive, self.bump()))
    }
}
