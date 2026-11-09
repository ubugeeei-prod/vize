//! Authored local class names in one resolved selector prelude.
//!
//! The selector grammar and CSS Modules scope (`:global`, `:local`) are the
//! same for every style language once its dialect module has found the
//! prelude and resolved nesting. Unsupported selector syntax refuses instead
//! of guessing names.

use alloc::collections::BTreeSet;
use alloc::vec::Vec;
use vize_l0::{String, ToCompactString};

/// The local class names one style block's rules declare.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoredClasses {
    names: BTreeSet<String>,
    complete: bool,
}

impl AuthoredClasses {
    pub(super) const fn new(names: BTreeSet<String>, complete: bool) -> Self {
        Self { names, complete }
    }

    #[must_use]
    pub const fn names(&self) -> &BTreeSet<String> {
        &self.names
    }

    #[must_use]
    pub fn into_names(self) -> BTreeSet<String> {
        self.names
    }

    /// Whether the names are all the block can declare: every rule's selector
    /// is known, and no preprocessor construct (include, import, mixin call,
    /// control flow, interpolated selector) can add rules this scan cannot see.
    #[must_use]
    pub const fn is_complete(&self) -> bool {
        self.complete
    }
}

pub(super) fn collect_classes_from_selector_prelude(
    prelude: &[u8],
    classes: &mut BTreeSet<String>,
) -> Option<()> {
    let trimmed = prelude.trim_ascii_start();
    if trimmed.starts_with(b"@") {
        return Some(());
    }
    if trimmed.contains(&b'\\') || !trimmed.is_ascii() {
        return None;
    }

    let mut index = 0usize;
    let mut local = true;
    // Functions restore the surrounding scope when they close. A bare global
    // scope survives commas inside a selector function; only a top-level comma
    // starts a fresh local selector.
    let mut functions = Vec::new();
    let mut attribute_depth = 0usize;
    while let Some(&byte) = trimmed.get(index) {
        if byte == b'[' {
            attribute_depth += 1;
        } else if byte == b']' {
            attribute_depth = attribute_depth.checked_sub(1)?;
        } else if attribute_depth == 0 {
            let scope = [(b":global".as_slice(), false), (b":local".as_slice(), true)]
                .into_iter()
                .find(|(keyword, _)| {
                    trimmed
                        .get(index..)
                        .is_some_and(|rest| rest.starts_with(keyword))
                        && trimmed
                            .get(index + keyword.len())
                            .is_none_or(|&next| next == b'(' || next.is_ascii_whitespace())
                });
            if let Some((keyword, mode)) = scope {
                index += keyword.len();
                if trimmed.get(index) == Some(&b'(') {
                    functions.push(local);
                    index += 1;
                }
                local = mode;
                continue;
            }
            match byte {
                b'(' => functions.push(local),
                b')' => local = functions.pop()?,
                b',' if functions.is_empty() => local = true,
                b'.' if local
                    && trimmed
                        .get(index + 1)
                        .is_some_and(|&b| is_css_class_start(b)) =>
                {
                    let start = index + 1;
                    let mut end = start + 1;
                    while trimmed.get(end).is_some_and(|&b| is_css_class_continue(b)) {
                        end += 1;
                    }
                    let class_name = core::str::from_utf8(trimmed.get(start..end)?).ok()?;
                    classes.insert(class_name.to_compact_string());
                    index = end;
                    continue;
                }
                _ => {}
            }
        }
        index += 1;
    }
    if attribute_depth != 0 || !functions.is_empty() {
        return None;
    }
    Some(())
}

fn is_css_class_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'-')
}

fn is_css_class_continue(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-')
}
