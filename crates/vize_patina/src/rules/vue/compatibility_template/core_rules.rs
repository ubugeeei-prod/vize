//! Core JavaScript rules applied to retained template expressions.

mod camelcase;
mod dot_notation;
mod eqeqeq;
mod no_console;
mod no_constant_condition;
mod no_empty_pattern;
mod no_implicit_coercion;
mod no_irregular_whitespace;
mod no_loss_of_precision;
mod no_negated_condition;
mod no_sparse_arrays;
mod no_useless_concat;
mod object_shorthand;
mod prefer_template;
mod quote_props;

pub use camelcase::Camelcase;
pub use dot_notation::DotNotation;
pub use eqeqeq::Eqeqeq;
pub use no_console::NoConsole;
pub use no_constant_condition::NoConstantCondition;
pub use no_empty_pattern::NoEmptyPattern;
pub use no_implicit_coercion::NoImplicitCoercion;
pub use no_irregular_whitespace::NoIrregularWhitespace;
pub use no_loss_of_precision::NoLossOfPrecision;
pub use no_negated_condition::NoNegatedCondition;
pub use no_sparse_arrays::NoSparseArrays;
pub use no_useless_concat::NoUselessConcat;
pub use object_shorthand::ObjectShorthand;
pub use prefer_template::PreferTemplate;
pub use quote_props::QuoteProps;
