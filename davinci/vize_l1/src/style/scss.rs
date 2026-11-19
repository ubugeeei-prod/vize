//! SCSS: braced rules that nest, `//` comments and `#{}` interpolation.

use super::structure::{Interpolation, Lexis, shared};

pub(super) const LEXIS: Lexis = Lexis {
    line_comments: true,
    interpolation: Interpolation::Hash,
    indented: false,
    indented_comments: false,
    colonless_declarations: false,
    construct: shared,
};
