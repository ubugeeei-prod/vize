use super::*;

#[test]
fn authored_and_decoded_whitespace_refusals_keep_the_complete_unmodified_value() -> Test {
    for (raw, expected, policy) in [
        ("a b", "a b", Policy::RawWhitespace),
        (" a", " a", Policy::RawWhitespace),
        ("a ", "a ", Policy::RawWhitespace),
        ("a\r\nb", "a\r\nb", Policy::RawWhitespace),
        ("a&#32;b", "a b", Policy::DecodedWhitespace),
        ("a&#9;b", "a\tb", Policy::DecodedWhitespace),
        ("a&NewLine;b", "a\nb", Policy::DecodedWhitespace),
    ] {
        let arena = Allocator::default();
        let source = alloc::format!("<template>{raw}</template>");
        let mut original = owner(&arena, &source)?;
        {
            let mut walk = original.begin().map_err(|_| "begin")?;
            let selected = walk.selected();
            let child = selected.children().next().ok_or("actual Text")?;
            let error = walk
                .root_text_value(child)
                .err()
                .ok_or("whitespace admitted")?;
            same(error.kind, Kind::TextValuePolicy(policy))?;
        }
        let original = original.finish();
        check(original.view().is_err())?;
        let file = original.file().ok_or("full refused File")?;
        let [record] = file.native_text_values() else {
            return Err("whole preparation parked");
        };
        let value = record.observation().ok_or("original full value")?;
        same(value.raw_text(), raw)?;
        same(value.source().text(), expected)?;
        check(core::ptr::eq(
            value.source().authored_root(),
            source.as_str(),
        ))?;
        same(
            record.state(),
            State::Refused(Kind::TextValuePolicy(policy)),
        )?;
        same(file.artifact().node_count(), 0)?;
        check(!file.is_complete())?;
    }
    Ok(())
}

#[test]
fn nontext_nested_and_sibling_events_keep_original_observations_and_never_mint() -> Test {
    for source in [
        "<template><div>&amp;lt;</div></template>",
        "<template><!--kept-->&amp;lt;</template>",
        "<template>{{1}}</template>",
    ] {
        let arena = Allocator::default();
        let mut original = owner(&arena, source)?;
        {
            let mut walk = original.begin().map_err(|_| "begin")?;
            let selected = walk.selected();
            let child = selected.children().next().ok_or("original child")?;
            check(walk.root_text_value(child).is_err())?;
        }
        let original = original.finish();
        let file = original.file().ok_or("retained File")?;
        let [record] = file.native_text_values() else {
            return Err("full L1 Result retained");
        };
        let failure = record.failure().ok_or("original NotText failure")?;
        same(
            failure.kind(),
            vize_l1::markup::NativeTextValueError::NotText,
        )?;
        check(core::ptr::eq(failure.block().root_source(), source))?;
        same(file.artifact().node_count(), 0)?;
        check(!file.is_complete() && original.view().is_err())?;
    }
    let arena = Allocator::default();
    let source = "<template><div>&amp;lt;</div></template>";
    let mut original = owner(&arena, source)?;
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        let selected = walk.selected();
        let parent = selected
            .children()
            .next()
            .ok_or("parent")?
            .into_element()
            .ok_or("Element")?;
        let child = parent.children().next().ok_or("nested Text")?;
        same(
            walk.root_text_value(child)
                .err()
                .ok_or("nested admitted")?
                .kind,
            Kind::TextValuePolicy(Policy::RootExtent),
        )?;
    }
    let original = original.finish();
    let file = original.file().ok_or("nested refused File")?;
    let value = file.native_text_values()[0]
        .observation()
        .ok_or("whole nested preparation retained")?;
    same(value.raw_text(), "&amp;lt;")?;
    same(value.source().text(), "&lt;")?;
    same(file.artifact().node_count(), 0)?;
    Ok(())
}

#[test]
fn root_extent_and_empty_template_cannot_be_completed_as_prepared_profile() -> Test {
    let arena = Allocator::default();
    let source = "<template>&amp;lt;<!--tail--></template>";
    let mut original = owner(&arena, source)?;
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        let selected = walk.selected();
        let child = selected.children().next().ok_or("first original Text")?;
        same(
            walk.root_text_value(child)
                .err()
                .ok_or("partial root admitted")?
                .kind,
            Kind::TextValuePolicy(Policy::RootExtent),
        )?;
    }
    let original = original.finish();
    let file = original.file().ok_or("partial source owner")?;
    same(file.native_text_values().len(), 1)?;
    same(file.artifact().node_count(), 0)?;
    // The ordinary empty cursor completes, but mints no prepared row/profile.
    let mut empty = owner(&arena, "<template></template>")?;
    empty
        .begin()
        .map_err(|_| "empty begin")?
        .complete()
        .map_err(|_| "ordinary empty complete")?;
    let empty = empty.finish();
    let file = empty.file().ok_or("ordinary empty File")?;
    check(file.is_complete() && file.native_text_values().is_empty())?;
    Ok(())
}

#[test]
fn original_setup_program_and_styles_cannot_be_promoted_by_root_text_source_custody() -> Test {
    let arena = Allocator::default();
    let source = "<script setup>const value=1</script><template>&amp;lt;</template>";
    let mut original = owner(&arena, source)?;
    original
        .parse_setup_program()
        .map_err(|_| "actual setup Program")?;
    {
        let mut walk = original.begin_setup().map_err(|_| "genuine setup begin")?;
        let selected = walk.selected();
        let child = selected.children().next().ok_or("root Text")?;
        same(
            walk.root_text_value(child)
                .err()
                .ok_or("setup source admitted")?
                .kind,
            Kind::TextValuePolicy(Policy::ScriptOrStyle),
        )?;
    }
    let original = original.finish();
    check(original.setup().is_err())?;
    let file = original.file().ok_or("whole refused setup owner")?;
    same(file.units().len(), 1)?;
    same(file.native_text_values().len(), 1)?;
    same(file.artifact().node_count(), 0)?;
    let source = "<template>&amp;lt;</template><style></style>";
    let mut original = owner(&arena, source)?;
    {
        let mut walk = original.begin().map_err(|_| "style original begin")?;
        let selected = walk.selected();
        same(
            walk.root_text_value(selected.children().next().ok_or("styled Text")?)
                .err()
                .ok_or("style admitted")?
                .kind,
            Kind::TextValuePolicy(Policy::ScriptOrStyle),
        )?;
    }
    let original = original.finish();
    check(original.view().is_err())?;
    same(
        original
            .file()
            .ok_or("styled owner")?
            .native_text_values()
            .len(),
        1,
    )?;
    Ok(())
}

#[test]
fn original_raw_nul_and_recovered_carrier_are_retained_without_guessed_decoder_errors() -> Test {
    for source in [
        "<template>a\0b</template>",
        "<template><p>&amp;lt;</template>",
    ] {
        let arena = Allocator::default();
        let mut original = owner(&arena, source)?;
        let original_errors = original.selected().component().carrier().errors.len();
        match original.begin() {
            Ok(mut walk) => {
                let selected = walk.selected();
                let child = selected
                    .children()
                    .next()
                    .ok_or("actual recovered/raw child")?;
                check(walk.root_text_value(child).is_err())?;
            }
            Err(issue) => same(issue.kind, Kind::UnsupportedChild)?,
        }
        let original = original.finish();
        check(original.view().is_err())?;
        same(
            original.selected().component().carrier().errors.len(),
            original_errors,
        )?;
        vize_l1::check_fidelity(&original.selected().component().carrier().tree)
            .map_err(|_| "full original malformed source")?;
        let file = original.file().ok_or("normally retained refusal File")?;
        same(file.artifact().node_count(), 0)?;
        check(!file.is_complete())?;
    }
    // HTML numeric zero genuinely replaces to U+FFFD; it is never a fake
    // decoded-NUL refusal or an invented character-origin inference.
    let arena = Allocator::default();
    let original = completed(&arena, "<template>&#0;</template>")?;
    same(
        text(original.file().ok_or("numeric replacement File")?)?.content,
        "�",
    )?;
    Ok(())
}
