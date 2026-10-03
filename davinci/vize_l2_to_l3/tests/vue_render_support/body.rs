use core::cell::Cell;
use vize_l0::{Span, id::NodeId};
use vize_l2::{
    artifact::{ComponentBody, ComponentFactory},
    expr::JsExpr,
};

pub struct Properties<'borrow, 'arena> {
    pub values: [(&'arena str, Span, Span, &'arena JsExpr<'arena>); 3],
    pub interpolation: &'arena JsExpr<'arena>,
    pub failed: &'borrow Cell<bool>,
}

impl<'arena> ComponentBody<'arena> for Properties<'_, 'arena> {
    fn run<R: ComponentFactory<'arena>>(self, region: &mut R, _: NodeId) {
        for (name, name_span, span, expression) in self.values {
            if region.bind(name, name_span, expression, span).is_err() {
                self.failed.set(true);
            }
        }
        if region
            .interpolation(self.interpolation, self.interpolation.span)
            .is_err()
        {
            self.failed.set(true);
        }
    }
}
