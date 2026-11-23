/// The caller's `SourceBlock` certifies that the base and the whole body
/// fit in the original u32-sized file.
#[inline]
pub(super) fn authored(body: &str, base: u32, start: u32, end: u32) -> Option<(u32, u32)> {
    body.get(start as usize..end as usize)?;
    Some((base + start, base + end))
}
