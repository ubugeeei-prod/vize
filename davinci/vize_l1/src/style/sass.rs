//! Sass, the indented syntax: a selector opens a block by indenting the
//! lines after it, and `=name`/`+name` spell `@mixin`/`@include`.

use super::structure::{Construct, Interpolation, Lexis};

pub(super) const LEXIS: Lexis = Lexis {
    line_comments: true,
    interpolation: Interpolation::Hash,
    indented: true,
    indented_comments: true,
    colonless_declarations: false,
    construct: sass_construct,
};

/// `=name` and `+name` spell `@mixin` and `@include`.
fn sass_construct(text: &str) -> Construct {
    let bytes = text.as_bytes();
    if matches!(bytes.first(), Some(b'=' | b'+'))
        && bytes
            .get(1)
            .is_some_and(|&byte| byte.is_ascii_alphabetic() || matches!(byte, b'-' | b'_'))
    {
        Construct::Sigil
    } else {
        Construct::Shared
    }
}
