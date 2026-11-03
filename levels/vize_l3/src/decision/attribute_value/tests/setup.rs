use super::*;
use vize_l1_to_l2::native_file::lower_selected_setup_sfc_native;

#[test]
fn every_actual_setup_target_preserves_its_own_context_and_original_value_cursor() -> Test {
    for lang in ["", " lang='ts'"] {
        for declaration in ["const unused=1", "let unused=1", "var unused=1"] {
            let arena = Allocator::default();
            let source = alloc::format!(
                "<script setup{lang}>{declaration}</script><template><div title='&amp;lt;'>x</div></template>"
            );
            let original = lower_selected_setup_sfc_native(
                &arena,
                &source,
                DescriptorOptions {
                    version: VueVersion::V3,
                    dialect: VueDialect::Vue,
                    template: SurfaceParseOptions::default(),
                },
            );
            let selected = original.admitted().ok_or("actual whole setup envelope")?;
            let setup = selected.setup();
            let file = setup.file();
            let actual = element(file, 0)?;
            let dom = native::build_native_selected_setup_dom_decisions(setup)
                .map_err(|_| "actual DOM setup context")?;
            let ssr = ssr::build_native_selected_setup_ssr_decisions(setup)
                .map_err(|_| "actual SSR setup context")?;
            let vapor = vapor::build_native_selected_setup_vapor_decisions(setup)
                .map_err(|_| "actual Vapor setup context")?;
            check(core::ptr::eq(dom.setup(), setup))?;
            check(core::ptr::eq(ssr.setup(), setup))?;
            check(core::ptr::eq(vapor.setup(), setup))?;
            for facts in [
                dom.original_attributes(),
                ssr.original_attributes(),
                vapor.original_attributes(),
            ] {
                let facts = facts.ok_or("complete selected value facts")?;
                check(core::ptr::eq(facts.file(), file) && facts.len() == 1)?;
                let value = facts
                    .value(0, actual, 0)
                    .ok_or("actual canonical slot")?
                    .observation()
                    .ok_or("normal whole value owner")?;
                check(value.source().text() == "&lt;")?;
                check(core::ptr::eq(
                    value.source().authored_root(),
                    source.as_str(),
                ))?;
            }
            check(dom.dom().ok_or("DOM facts")?.unsupported().is_empty())?;
            check(ssr.ssr().ok_or("SSR facts")?.unsupported().is_empty())?;
            check(vapor.vapor().ok_or("Vapor facts")?.unsupported().is_empty())?;
        }
    }
    Ok(())
}
