//! CSS (and PostCSS): braced rules, `/* */` comments, no interpolation.

use alloc::collections::BTreeSet;

use super::super::AuthoredClasses;
use super::super::classes::collect_classes_from_selector_prelude;
use super::super::structure::{Interpolation, Lexis, StyleSheet, shared};

pub(super) const LEXIS: Lexis = Lexis {
    line_comments: false,
    interpolation: Interpolation::None,
    indented: false,
    indented_comments: false,
    colonless_declarations: false,
    construct: shared,
};

/// Simple authored class selectors from rule preludes. Declarations are never
/// scanned, so values such as `url(./asset.png)` cannot become fake module
/// keys. Interpolation syntax left in a CSS block names classes no scan can see.
pub(super) fn authored_classes(sheet: &StyleSheet<'_>) -> Option<AuthoredClasses> {
    if !sheet.is_terminated() {
        return None;
    }
    let mut classes = BTreeSet::new();
    for node in sheet.nodes() {
        if node.block().is_some() {
            collect_classes_from_selector_prelude(
                sheet.code(node.prelude()).as_bytes(),
                &mut classes,
            )?;
        }
    }
    Some(AuthoredClasses::new(
        classes,
        !sheet.source().contains("#{"),
    ))
}
