use super::*;
use vize_l1_to_l2::native_file::lower_selected_scoped_sfc_native;

#[test]
fn genuine_styled_ssr_view_consumes_the_same_original_value_slots() -> Test {
    let arena = Allocator::default();
    let source = "<template><div title='a &amp;lt; b'><p title='😀&amp;amp;'/></div></template><style scoped>.a:empty{color:red}</style>";
    let original = lower_selected_scoped_sfc_native(
        &arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    let selected = original.original().template().ok_or("selected owner")?;
    let file = selected.file().ok_or("complete original File")?;
    let style = original.style_syntax().ok_or("normal original style")?;
    let view = original.admitted().ok_or("genuine styled envelope")?;
    let analysis = ssr::build_native_scoped_ssr_file_decisions(view.into_scoped_template_view())
        .map_err(|_| "scoped original cursor")?;
    check(core::ptr::eq(analysis.owner(), selected))?;
    check(core::ptr::eq(analysis.file(), file))?;
    check(core::ptr::eq(analysis.scoped().style_syntax(), style))?;
    let facts = analysis
        .original_attributes()
        .ok_or("normal all-row finish")?;
    check(core::ptr::eq(facts.file(), file) && facts.len() == 2)?;
    let parent = element(file, 0)?;
    let [Op::Element(child)] = parent.children.ops.as_slice() else {
        return Err("original nested Element");
    };
    for (index, actual, expected) in [(0, parent, "a &lt; b"), (1, child.as_ref(), "😀&amp;")] {
        let observation = facts
            .value(index, actual, 0)
            .ok_or("actual styled slot")?
            .observation()
            .ok_or("whole original preparation")?;
        check(observation.source().text() == expected)?;
        check(core::ptr::eq(observation.source().authored_root(), source))?;
        check(core::ptr::eq(
            observation,
            file.native_attribute_values()[index]
                .observation()
                .ok_or("same normally parked owner")?,
        ))?;
    }
    let generic = ssr::build_ssr_file_decisions(file).map_err(|_| "generic styled File")?;
    check(generic.original_attributes().is_none())?;
    check(matches!(
        ssr::build_native_ssr_file_decisions(selected.view().map_err(|_| "plain view")?),
        Err(ssr::NativeSsrBuildError::StyledSource { .. })
    ))?;
    Ok(())
}
