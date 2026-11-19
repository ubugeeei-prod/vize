//! Less: braced rules that nest, `//` comments, `@{}` interpolation, and
//! mixins and detached rulesets that emit rules where they are called.

use super::structure::{Interpolation, Lexis};

pub(super) const LEXIS: Lexis = Lexis {
    interpolation: Interpolation::At,
    ..super::scss::LEXIS
};
