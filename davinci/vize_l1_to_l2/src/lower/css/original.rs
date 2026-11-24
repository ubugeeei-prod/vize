//! One retained semantic carrier shared by emission and type checking.
use super::{BindingOp, Box, ElementOp, Namespace, Op, Region, Vec, VueCssBindOp, css_bind_expr};
use vize_l0::{Allocator, SourceBlock};
use vize_l1::container::vue::StyleView;

/// Every binding belongs to this exact original style and is parsed once.
pub struct OriginalStyleBindings<'a> {
    block: SourceBlock<'a>,
    bindings: Vec<'a, BindingOp<'a>>,
}

impl<'a> OriginalStyleBindings<'a> {
    #[must_use]
    pub fn read(allocator: &'a Allocator, style: StyleView<'_, 'a>) -> Self {
        Self::from_block(allocator, style.block())
    }

    /// Lower a style already selected by the compiler's L1 container parser.
    #[must_use]
    #[inline]
    pub fn from_block(allocator: &'a Allocator, block: SourceBlock<'a>) -> Self {
        let mut style = Self {
            block,
            bindings: Vec::new_in(&allocator),
        };
        if !block.source().is_empty() {
            style.read_calls(allocator);
        }
        style
    }

    fn read_calls(&mut self, allocator: &'a Allocator) {
        for call in vize_l1::css::bind::binding_calls_in(self.block) {
            self.bindings.push(BindingOp::VueCssBind(Box::new_in(
                VueCssBindOp {
                    value: css_bind_expr(allocator, call.expression, call.argument),
                    span: call.call,
                },
                &allocator,
            )));
        }
    }

    #[must_use]
    pub fn block(&self) -> SourceBlock<'a> {
        self.block
    }

    /// Whether this block contributed a completed binding, including a recovered prefix.
    #[must_use]
    pub fn has_calls(&self) -> bool {
        !self.bindings.is_empty()
    }

    pub fn calls(&self) -> impl Iterator<Item = &VueCssBindOp<'a>> {
        self.bindings.iter().filter_map(|binding| match binding {
            BindingOp::VueCssBind(call) => Some(call.as_ref()),
            _ => None,
        })
    }

    /// Publish the same carrier as ordinary style lowering, retaining the
    /// expression nodes and moving the already built binding storage into it.
    #[must_use]
    pub fn into_op(self, allocator: &'a Allocator) -> Op<'a> {
        Op::Element(Box::new_in(
            ElementOp {
                tag: "style",
                namespace: Namespace::Html,
                attributes: Vec::new_in(&allocator),
                bindings: self.bindings,
                children: Region {
                    ops: Vec::new_in(&allocator),
                },
                span: self.block.span(),
            },
            &allocator,
        ))
    }
}
