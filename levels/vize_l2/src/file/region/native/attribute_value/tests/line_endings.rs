use super::*;

#[test]
fn inherited_raw_cr_and_crlf_values_retain_borrowed_original_bytes_and_slots() -> Test {
    let arena = Allocator::default();
    // Verbatim source from the unchanged nine-module original SSR pack.
    let source = "<template><div id=\"r\r\ns\rt\nu\u{2028}v\u{2029}w\"><p title=\"a\r\nb\rc\nd\u{2028}e\u{2029}f\"/><!--a\r\nb\rc\nd\u{2028}e\u{2029}f--></div></template>";
    let output = lower(&arena, source)?;
    let file = output.view().map_err(|_| "complete original view")?.file();
    let file = file.ok_or("complete same File")?;
    check(file.is_complete())?;
    same(file.native_attribute_values().len(), 2)?;
    let root = element(file, 0)?;
    let Some(Op::Element(nested)) = root.children.ops.first() else {
        return Err("actual nested Element");
    };
    for (index, actual, expected) in [
        (0, root, "r\r\ns\rt\nu\u{2028}v\u{2029}w"),
        (1, nested.as_ref(), "a\r\nb\rc\nd\u{2028}e\u{2029}f"),
    ] {
        let joined = file
            .native_attribute_value_for(index, actual, 0)
            .ok_or("exact original allocation/slot")?;
        let value = joined.observation().ok_or("whole original observation")?;
        same(value.source().text(), expected)?;
        check(value.source().decode_map().is_none())?;
        check(core::ptr::eq(value.source().text(), value.raw_value()))?;
        check(core::ptr::eq(value.source().authored_root(), source))?;
        check(core::ptr::eq(joined.file(), file))?;
        check(core::ptr::eq(joined.element(), actual))?;
        same(actual.attributes[0].value, Some(expected))?;
        same(value.value_span().slice(source), expected)?;
    }
    Ok(())
}
