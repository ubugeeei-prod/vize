//! Maximal prefix of a group, with the intersection of its analysis masks.
//! The verifier instantiates this same scan with arbitrary masks and barriers.
#[inline]
pub(super) fn next_group<T>(
    remaining: &[T],
    classify: impl Fn(&T) -> (bool, u64),
) -> Option<(usize, bool, u64)> {
    let (first, tail) = remaining.split_first()?;
    let (fusable, mut mask) = classify(first);
    let mut len = 1;
    if fusable {
        for pass in tail {
            let (joins, preserved) = classify(pass);
            if !joins {
                break;
            }
            mask &= preserved;
            len += 1;
        }
    }
    Some((len, !fusable, mask))
}
