//! CSS (and PostCSS): braced rules, `/* */` comments, no interpolation.

use alloc::collections::BTreeSet;
use alloc::vec::Vec;

use super::super::AuthoredClasses;
use super::super::classes::collect_classes_from_selector_prelude;
use super::super::structure::{Interpolation, Lexis, shared};

pub(super) const LEXIS: Lexis = Lexis {
    line_comments: false,
    interpolation: Interpolation::None,
    indented: false,
    indented_comments: false,
    colonless_declarations: false,
    construct: shared,
};

/// Extract simple authored class selectors from selector preludes. Declarations
/// are never scanned, so values such as `url(./asset.png)` cannot become fake
/// module keys. Comments and string contents are blanked before inspection.
/// Interpolation syntax left in a CSS block names classes no scan can see.
pub(super) fn authored_classes(css: &str) -> Option<AuthoredClasses> {
    let bytes = css.as_bytes();
    let mut classes = BTreeSet::new();
    let mut prelude = Vec::new();
    let mut index = 0usize;
    let mut quote = None;

    while let Some(&byte) = bytes.get(index) {
        if let Some(open_quote) = quote {
            if byte == b'\\' {
                index = (index + 2).min(bytes.len());
                continue;
            }
            if byte == open_quote {
                quote = None;
            }
            index += 1;
            continue;
        }
        if byte == b'/' && bytes.get(index + 1) == Some(&b'*') {
            let relative_end = css.get(index + 2..)?.find("*/")?;
            index += relative_end + 4;
            continue;
        }
        if matches!(byte, b'\'' | b'"') {
            quote = Some(byte);
            index += 1;
            continue;
        }
        match byte {
            b'{' => {
                collect_classes_from_selector_prelude(&prelude, &mut classes)?;
                prelude.clear();
            }
            b';' | b'}' => prelude.clear(),
            _ => prelude.push(byte),
        }
        index += 1;
    }
    quote
        .is_none()
        .then(|| AuthoredClasses::new(classes, !css.contains("#{")))
}
