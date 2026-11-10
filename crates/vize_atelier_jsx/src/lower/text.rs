//! JSX text children. The whitespace rule is the shared L1→L2 lowering's.

use oxc_ast::ast::JSXText;
use vize_l0::Box;
use vize_l1_to_l2::framework::vue::jsx::clean_text as clean_jsx_text;
use vize_relief::{TemplateChildNode, TextNode};

use super::Lowerer;

impl<'a, 'm, 's: 'a> Lowerer<'a, 'm, 's> {
    /// Lower a JSX text child, returning `None` if it cleans to nothing.
    pub(crate) fn lower_text(&mut self, text: &JSXText<'_>) -> Option<TemplateChildNode<'a>> {
        let cleaned = clean_jsx_text(text.value.as_str());
        if cleaned.is_empty() {
            return None;
        }
        let loc = self.mapper().location(text.span);
        Some(TemplateChildNode::Text(Box::new_in(
            TextNode::new(self.bump().alloc_str(&cleaned), loc),
            &self.bump(),
        )))
    }
}
