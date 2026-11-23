//! Verify the byte projection used for every editor decision, label and proof link.
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
    let expected = start <= end
        && end as usize <= length
        && body.is_char_boundary(start as usize)
        && body.is_char_boundary(end as usize);
    let result = projection::authored(body, base, start, end);
    assert_eq!(result.is_some(), expected);
    if let Some((first, last)) = result {
        assert_eq!(u64::from(first), u64::from(base) + u64::from(start));
        assert_eq!(u64::from(last), u64::from(base) + u64::from(end));
    }
}
