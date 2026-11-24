/// Insert after earlier equal ranks. Existing rows shift by one up to the
/// first free slot or the end of the window. The latter drops its last row.
pub(super) fn insert<T: Copy>(slots: &mut [Option<T>], row: T, before: impl Fn(T, T) -> bool) {
    let mut slots = slots.iter_mut();
    if let Some(first) = slots.find(|slot| slot.is_none_or(|old| before(row, old))) {
        let mut displaced = first.replace(row);
        for slot in slots {
            if displaced.is_none() {
                break;
            }
            core::mem::swap(slot, &mut displaced);
        }
    }
}
