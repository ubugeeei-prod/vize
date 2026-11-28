//! Verify the shipped byte projection against the standard string slice.
mod projection;

#[kani::proof]
#[kani::unwind(8)]
fn all_utf8_ranges_match_the_reference_and_fit_the_source_file() {
    let bytes: [u8; 5] = kani::any();
    let length: usize = kani::any();
    kani::assume(length <= bytes.len());
    let Ok(body) = core::str::from_utf8(&bytes[..length]) else {
        return;
    };
    let base: u32 = kani::any();
    kani::assume(u64::from(base) + length as u64 <= u64::from(u32::MAX));
    let start: u32 = kani::any();
    let end: u32 = kani::any();
    let expected = body.get(start as usize..end as usize).is_some();
    let result = projection::authored(body.as_bytes(), base, start, end);
    assert_eq!(result.is_some(), expected);
    if let Some((first, last)) = result {
        assert_eq!(u64::from(first), u64::from(base) + u64::from(start));
        assert_eq!(u64::from(last), u64::from(base) + u64::from(end));
    }
    kani::cover!(
        result.is_some() && start == 0 && end as usize == length,
        "the whole block projects"
    );
    kani::cover!(
        result.is_some() && start == end,
        "an empty range at a boundary projects"
    );
    kani::cover!(
        start > end && result.is_none(),
        "a reversed range is refused"
    );
    kani::cover!(
        end as usize > length && result.is_none(),
        "an out-of-bounds range is refused"
    );
    kani::cover!(
        start <= end && end as usize <= length && !expected,
        "a range cutting a multibyte scalar is refused"
    );
    kani::cover!(
        result.is_some() && u64::from(base) + u64::from(end) == u64::from(u32::MAX),
        "a range reaching the largest source endpoint projects"
    );
}
