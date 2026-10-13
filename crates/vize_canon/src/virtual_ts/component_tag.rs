//! Tag anchors use retained parser coordinates and authored template bytes.
//! Component binding names may instead be generated aliases or `:is` values.

use std::ops::Range;
use vize_croquis::croquis::ComponentUsage;

pub(super) fn source_range(
    template: Option<&str>,
    offset: u32,
    usage: &ComponentUsage,
) -> Option<Range<usize>> {
    let start = usage.start as usize;
    let opening = template?
        .get(start..usage.end as usize)?
        .strip_prefix('<')?;
    let len = opening
        .bytes()
        .position(|byte| byte.is_ascii_whitespace() || matches!(byte, b'/' | b'>'))
        .unwrap_or(opening.len());
    if len == 0 {
        return None;
    }
    let start = (offset as usize).checked_add(start)?.checked_add(1)?;
    Some(start..start.checked_add(len)?)
}
