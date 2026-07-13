//! Owned Vapor planning from the frontend-neutral Rendu HIR.

mod emit;
mod lower;
mod model;
mod syntax;

#[cfg(test)]
mod tests;

pub use emit::{VaporEmitResult, emit_rendu, emit_vapor_plan};
pub use lower::plan_rendu;
pub use lower::plan_rendu as lower_rendu;
pub use model::{
    VaporAttributeValue, VaporBinding, VaporBlock, VaporBlockId, VaporBranch, VaporDirective,
    VaporExpression, VaporExpressionId, VaporName, VaporOperation, VaporPlan, VaporProperty,
};
