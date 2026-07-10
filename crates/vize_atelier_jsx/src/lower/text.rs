//! JSX text whitespace handling.
//!
//! JSX text is cleaned with the same rule `@vue/babel-plugin-jsx` inherits from
//! Babel: lines are split, tabs become spaces, whitespace adjacent to a newline
//! is trimmed, blank lines are dropped, and the remaining lines are joined with
//! a single space. Leading whitespace on the first line and trailing whitespace
//! on the last line are preserved.

use oxc_ast::ast::JSXText;
use vize_carton::Box;
use vize_relief::{TemplateChildNode, TextNode};

use super::Lowerer;
use crate::syntax::text::clean_jsx_text;

impl<'a, 'm, 's> Lowerer<'a, 'm, 's> {
    /// Lower a JSX text child, returning `None` if it cleans to nothing.
    pub(crate) fn lower_text(&mut self, text: &JSXText<'_>) -> Option<TemplateChildNode<'a>> {
        let cleaned = clean_jsx_text(text.value.as_str());
        if cleaned.is_empty() {
            return None;
        }
        let loc = self.mapper().location(text.span);
        Some(TemplateChildNode::Text(Box::new_in(
            TextNode::new(cleaned, loc),
            self.bump(),
        )))
    }
}
