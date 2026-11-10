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

#[cfg(test)]
mod tests;
