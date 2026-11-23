// Shared with the standalone model checker so it exercises this exact policy.
fn same_header(before: StyleView<'_, '_>, after: StyleView<'_, '_>) -> bool {
    let a = before.attrs();
    let b = after.attrs();
    if a.len() != b.len() {
        return false;
    }
    if a.iter()
        .map(|v| (v.name, v.value))
        .eq(b.iter().map(|v| (v.name, v.value)))
    {
        return true;
    }
    // The stock formatter can order plain SFC header flags. Duplicates and
    // dynamic or unfamiliar attributes retain their original ordering contract.
    a.iter().enumerate().all(|(i, attr)| {
        matches!(attr.name, "lang" | "scoped" | "module")
            && !a.iter().take(i).any(|prior| prior.name == attr.name)
            && b.iter()
                .any(|other| (other.name, other.value) == (attr.name, attr.value))
    })
}
