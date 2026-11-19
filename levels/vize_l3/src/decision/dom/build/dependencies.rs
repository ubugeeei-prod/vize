//! Commit semantic demands when the original node transformation closes.

use super::{DomBuilder, DomChildren, DomDependency, DomExpressionFacts, DomNode, Op};
use crate::decision::dom::vue::policy::FileReads;

impl<'owner, 'arena, F: DomExpressionFacts, R: FileReads<'owner, 'arena>>
    DomBuilder<'_, 'owner, 'arena, F, R>
{
    pub(super) fn complete_dependencies(
        &mut self,
        node: &DomNode<'_, '_>,
        has_text: bool,
        normalize_class: bool,
        normalize_style: bool,
    ) {
        if has_text && matches!(node.children, DomChildren::Array(_)) {
            self.demand(DomDependency::TextValue);
        }
        if normalize_class {
            self.demand(DomDependency::ClassNormalization);
        }
        if normalize_style {
            self.demand(DomDependency::StyleNormalization);
        }
        if matches!(node.op, Op::Element(_)) {
            if node.block_eligible {
                self.demand(DomDependency::BlockBoundary);
                self.demand(DomDependency::NativeElementBlock);
            } else {
                self.demand(DomDependency::NativeElementValue);
            }
        }
    }
}
