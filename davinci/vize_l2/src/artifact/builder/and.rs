//! `left && <region>` whose `left` is not known to be a boolean: a `ui.and`
//! whose region is what a truthy `left` renders.

use super::{ArtifactError, Builder, Owner, RegionBuilder, check};
use crate::artifact::OpFamily;
use crate::expr::ExprRef;
use vize_l0::{Span, id::NodeId};

impl<'a> Builder<'a> {
    pub(super) fn and(
        &mut self,
        left: ExprRef<'a>,
        span: Span,
        children: impl FnOnce(&mut RegionBuilder<'_, 'a>, NodeId),
    ) -> Result<NodeId, ArtifactError> {
        let id = self.prepare(span)?;
        check::expression(self.parts.source, id, left, span)?;
        // A falsy `left` is a raw child value, as `ui.jsx-child` is.
        self.parts.features = self.parts.features.observing(OpFamily::JsxChild);
        self.children(Owner::And { left }, span, id, children)?;
        Ok(id)
    }
}
