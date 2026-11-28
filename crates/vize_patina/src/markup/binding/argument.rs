//! Authored argument locations from the existing parse, without re-scanning.

use super::{MarkupBinding, MarkupBindingInner};
use crate::ir::ByteRange;
use crate::markup::l2::surface::SurfaceDirective;
use oxc_ast::ast::JSXAttributeName;
use oxc_span::GetSpan;
use vize_relief::ExpressionNode;

impl MarkupBinding<'_> {
    /// Staticness of the argument, retaining qualified JSX attribute names.
    pub(crate) fn argument_is_static(&self) -> bool {
        match self.inner {
            MarkupBindingInner::ReliefDirective(node) => matches!(
                node.arg.as_ref(), Some(ExpressionNode::Simple(arg)) if arg.is_static
            ),
            MarkupBindingInner::L2Binding(binding) => {
                binding.arg().is_some_and(|(_, is_static)| is_static)
            }
            MarkupBindingInner::Surface {
                attr, static_name, ..
            } => {
                static_name.is_some()
                    || SurfaceDirective::parse(attr.name.text)
                        .is_some_and(|directive| directive.arg_static && directive.arg.is_some())
            }
            _ => true,
        }
    }

    /// The authored argument range, when the backend retains its spelling.
    /// Dynamic brackets and directive prefixes are outside this range.
    pub fn argument_range(&self) -> Option<ByteRange> {
        let attr = match self.inner {
            MarkupBindingInner::ReliefDirective(node) => {
                return match node.arg.as_ref()? {
                    ExpressionNode::Simple(arg) => Some(crate::markup::loc_to_range(&arg.loc)),
                    _ => None,
                };
            }
            MarkupBindingInner::L2Binding(binding) => binding.surface?,
            MarkupBindingInner::Surface {
                attr,
                static_name: None,
                ..
            } => attr,
            _ => return None,
        };
        let argument = SurfaceDirective::parse(attr.name.text)?.arg?;
        let offset = argument.as_ptr() as usize - attr.name.text.as_ptr() as usize;
        let start = self.range().start + offset as u32;
        Some(ByteRange::new(start, start + argument.len() as u32))
    }
}

impl<'a> MarkupBinding<'a> {
    /// Preserve a JSX namespace rather than mistaking its local name for a DOM sink.
    pub(crate) fn authored_argument<'b>(&'b self, source: &'b str) -> Option<&'b str> {
        if let MarkupBindingInner::Jsx { node, offset } = self.inner
            && let JSXAttributeName::NamespacedName(name) = &node.name
        {
            if name.namespace.name.as_str() == "v-bind" {
                return Some(name.name.name.as_str());
            }
            let span = node.name.span();
            return source.get(
                span.start.checked_add(offset)? as usize..span.end.checked_add(offset)? as usize,
            );
        }
        self.arg_name()
    }

    /// Borrow the expression text for this callback, including a direct OXC view.
    pub(crate) fn source_expression<'b>(&'b self, source: &'b str) -> Option<&'b str> {
        if let Some(expression) = self.expression() {
            return Some(expression);
        }
        let retained = self.retained_expression()?;
        let range = retained.authored_range(retained.ast.span())?;
        source.get(range.start as usize..range.end as usize)
    }
}
