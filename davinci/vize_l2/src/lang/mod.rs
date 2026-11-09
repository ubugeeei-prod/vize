//! Language-specific L2 expression semantics.

pub(crate) mod dump;
pub mod js;
pub mod tsrx;

pub use crate::expr::foreign::{ForeignExpr, ForeignFact};
