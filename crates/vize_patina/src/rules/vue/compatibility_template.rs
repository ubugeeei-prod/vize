//! Compatibility rules for JavaScript retained inside Vue templates.

mod core_checks;
mod core_rules;
mod origin;
mod restricted;
mod template_rules;

pub use core_rules::{
    Camelcase, DotNotation, Eqeqeq, NoConsole, NoConstantCondition, NoEmptyPattern,
    NoImplicitCoercion, NoIrregularWhitespace, NoLossOfPrecision, NoNegatedCondition,
    NoSparseArrays, NoUselessConcat, ObjectShorthand, PreferTemplate, QuoteProps,
};
pub use restricted::NoRestrictedSyntax;
pub use template_rules::{NoLiteralsInTemplate, NoRestrictedVBind, PreferVModel, VIfElseKey};

use crate::rule::Rule;

pub(crate) fn rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(Camelcase),
        Box::new(DotNotation),
        Box::new(Eqeqeq),
        Box::new(NoConsole),
        Box::new(NoConstantCondition),
        Box::new(NoEmptyPattern),
        Box::new(NoImplicitCoercion),
        Box::new(NoIrregularWhitespace),
        Box::new(NoLossOfPrecision),
        Box::new(NoNegatedCondition),
        Box::new(NoSparseArrays),
        Box::new(NoUselessConcat),
        Box::new(ObjectShorthand),
        Box::new(PreferTemplate),
        Box::new(QuoteProps),
        Box::new(NoRestrictedSyntax::default()),
        Box::new(NoLiteralsInTemplate),
        Box::new(NoRestrictedVBind),
        Box::new(PreferVModel),
        Box::new(VIfElseKey),
    ]
}

#[cfg(test)]
mod tests;
