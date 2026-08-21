//! The Vue 2 pipe-filter splitter, mirrored byte-for-byte from the
//! shipped lane (`crates/vize_atelier_core/src/transforms/
//! legacy_filters.rs`, itself a mirror of `@vue/compiler-core`'s
//! `parseFilter`).
//!
//! # Why a mirror and not an import (the installment-2 precedent)
//!
//! Importing the shipped splitter would point the strangler's new lane
//! at the legacy crate — the dependency inversion the v-for installment
//! rejected for the same ~120 duplicated grammar lines. The two copies
//! stay confined to their two homes, the differential lane is the
//! standing agreement proof (the legacy battery pins every split against
//! the shipped `parse_filters` directly), and at the exit gate the
//! legacy copy is deleted, not unified.
//!
//! The mirrored rules, in the shipped lane's own words: top-level `|`
//! only — pipes inside strings, template literals, regex literals, or
//! `()[]{}` nesting are not filter separators, and `||` is logical-OR,
//! never a filter. A chain whose every segment names a valid filter
//! splits; **any** malformed segment name abandons the whole split (the
//! `rewrite_filters_in_place` bail, mirrored), leaving the expression to
//! the ordinary admission rule.

use alloc::vec::Vec as StdVec;

use vize_carton::String;

/// One split filter segment: the trimmed authored text and the bare
/// filter name it resolves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilterSegment {
    /// The segment exactly as authored after its top-level `|`, trimmed
    /// (`"capitalize"`, `"f(b)"`).
    pub text: String,
    /// The bare filter name (`"f"` for `"f(b)"`), the asset the
    /// realization resolves.
    pub name: String,
}

/// The recorded split of one filter site, keyed by its op's page-order
/// id ([`crate::lower::Lowered::filters`]); validated and consumed by
/// `pass::legacy`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilterParts {
    /// The base expression text the chain applies to, trimmed.
    pub base: String,
    /// The segments, outermost last.
    pub segments: StdVec<FilterSegment>,
}

const _: () = {
    const fn assert_owned<T: 'static>() {}
    assert_owned::<FilterSegment>();
    assert_owned::<FilterParts>();
};

/// 64-bit footprints, guarded like every fact-size assert.
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<FilterSegment>() == 48);
    assert!(core::mem::size_of::<FilterParts>() == 48);
};

/// Mirrors the shipped `validDivisionCharRE` (`/[\w).+\-_$\]]/`): a `/`
/// after one of these is division, otherwise it opens a regex literal.
fn is_valid_division_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || matches!(c, ')' | '.' | '+' | '-' | '$' | ']')
}

/// Mirrors the shipped `is_simple_identifier`
/// (`transforms/transform_expression/prefix.rs:84`), which the shipped
/// `filter_name` validates candidate names through.
fn is_simple_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_alphabetic() && first != '_' && first != '$' {
        return false;
    }
    chars.all(|c| c.is_alphanumeric() || c == '_' || c == '$')
}

/// Mirrors the shipped `filter_name`: the bare name of a (possibly
/// call-style) segment, `None` for an empty or non-identifier name
/// (`-` allowed — the shipped `toValidAssetId` maps it at realization).
fn segment_name(filter: &str) -> Option<&str> {
    let name = match filter.find('(') {
        Some(idx) => &filter[..idx],
        None => filter,
    };
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    let mut dashless = String::default();
    for c in name.chars() {
        dashless.push(if c == '-' { '_' } else { c });
    }
    if is_simple_identifier(dashless.as_str()) {
        Some(name)
    } else {
        None
    }
}

/// Mirrors the shipped `parse_filters`, then validates every segment
/// name up front exactly as `rewrite_filters_in_place` does: any
/// malformed name abandons the whole split. Returns `None` when the
/// expression carries no top-level filter pipe or the split is
/// abandoned — the caller then admits the text through the ordinary
/// rule, byte-identical to a non-filter dialect.
pub fn filter_split(exp: &str) -> Option<FilterParts> {
    let bytes = exp.as_bytes();
    let len = bytes.len();

    let mut in_single = false;
    let mut in_double = false;
    let mut in_template = false;
    let mut in_regex = false;
    let mut curly: u32 = 0;
    let mut square: u32 = 0;
    let mut paren: u32 = 0;

    let mut last_filter_index = 0usize;
    let mut expression: Option<String> = None;
    let mut filters: StdVec<String> = StdVec::new();
    let mut prev: u8 = 0;

    let mut i = 0usize;
    while i < len {
        let c = bytes[i];

        if in_single {
            if c == b'\'' && prev != b'\\' {
                in_single = false;
            }
        } else if in_double {
            if c == b'"' && prev != b'\\' {
                in_double = false;
            }
        } else if in_template {
            if c == b'`' && prev != b'\\' {
                in_template = false;
            }
        } else if in_regex {
            if c == b'/' && prev != b'\\' {
                in_regex = false;
            }
        } else if c == b'|'
            && bytes.get(i + 1).copied() != Some(b'|')
            && (i == 0 || bytes[i - 1] != b'|')
            && curly == 0
            && square == 0
            && paren == 0
        {
            if expression.is_none() {
                last_filter_index = i + 1;
                expression = Some(String::from(exp[..i].trim()));
            } else {
                filters.push(String::from(exp[last_filter_index..i].trim()));
                last_filter_index = i + 1;
            }
        } else {
            match c {
                b'"' => in_double = true,
                b'\'' => in_single = true,
                b'`' => in_template = true,
                b'(' => paren += 1,
                b')' => paren = paren.saturating_sub(1),
                b'[' => square += 1,
                b']' => square = square.saturating_sub(1),
                b'{' => curly += 1,
                b'}' => curly = curly.saturating_sub(1),
                _ => {}
            }
            if c == b'/' {
                // Look back past spaces for the previous non-space char
                // to decide division vs. regex.
                let mut j = i as isize - 1;
                let mut p: Option<char> = None;
                while j >= 0 {
                    let pc = exp[j as usize..].chars().next().unwrap_or(' ');
                    if pc != ' ' {
                        p = Some(pc);
                        break;
                    }
                    j -= 1;
                }
                if p.is_none_or(|pc| !is_valid_division_char(pc)) {
                    in_regex = true;
                }
            }
        }

        prev = c;
        i += 1;
    }

    match &mut expression {
        None => return None,
        Some(_) if last_filter_index != 0 => {
            filters.push(String::from(exp[last_filter_index..].trim()));
        }
        Some(_) => {}
    }

    let base = expression?;
    if filters.is_empty() {
        return None;
    }
    // The whole-split bail: every segment must name a valid filter, or
    // the shipped rewrite leaves the expression untouched.
    let mut segments = StdVec::with_capacity(filters.len());
    for text in filters {
        let name = segment_name(text.as_str())?;
        let name = String::from(name);
        segments.push(FilterSegment { text, name });
    }
    Some(FilterParts { base, segments })
}
