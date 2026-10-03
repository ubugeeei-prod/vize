use super::*;

#[test]
fn setup_eligibility_refuses_before_any_original_template_node_and_keeps_stock_owner()
-> Result<(), &'static str> {
    for content in [
        "/*whole*/ const =",
        "/*whole*/ const rows=[1]",
        "/*whole*/ const rows={x:1}",
        "/*whole*/ const rows=run()",
        "/*whole*/ export const rows=1",
        "/*whole*/ const __expose=1",
        "/*whole*/ const rows=010",
        "/*whole*/ const rows:number[]=[]",
        "/*whole*/",
        "",
    ] {
        let arena = Allocator::default();
        let source = std::format!(
            "<template><i title='must not mint'/></template><script setup lang=ts>{content}</script>"
        );
        let owner = lower_selected_setup_sfc_native(&arena, &source, options());
        check(owner.admitted().is_none())?;
        check(!owner.original().issues().is_empty())?;
        let original = owner
            .original()
            .template()
            .ok_or("same parked File owner")?;
        let syntax = original
            .retained_setup()
            .ok_or("full original setup owner")?;
        check(syntax.source().text() == content)?;
        check(core::ptr::eq(
            syntax.source().authored_root(),
            source.as_str(),
        ))?;
        if content.contains("/*whole*/") {
            check(syntax.comments().count() == 1)?;
        }
        if content == "/*whole*/ const =" {
            check(syntax.diagnostics().count() != 0)?;
        }
        check(original.view().is_err())?;
        if let Some(file) = original.file() {
            check(file.artifact().node_count() == 0)?;
        }
        if let Some(file) = original.rejected_file() {
            check(file.artifact().parts.root.ops.is_empty())?;
        }
    }
    Ok(())
}

#[test]
fn omitted_sibling_external_and_profile_refusals_preserve_descriptor_before_program_parse()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    for source in [
        "<template></template>",
        "<template></template><script>const value=1</script><script setup>const rows=1</script>",
        "<template></template><script setup>const rows=1</script><style scoped>.x{}</style>",
        "<template></template><custom>whole</custom><script setup>const rows=1</script>",
        "<template></template><script setup src='external.js'></script>",
        "<template></template><script setup lang=tsx>const rows=1</script>",
        "<template></template><script setup='true'>const rows=1</script>",
        "<template></template><script setup>const rows=1</script><script setup>const other=2</script>",
    ] {
        let owner = lower_selected_setup_sfc_native(&arena, source, options());
        check(owner.admitted().is_none())?;
        check(owner.original().template().is_none())?;
        check(!owner.original().issues().is_empty())?;
        check(core::ptr::eq(
            owner.original().descriptor().source(),
            source,
        ))?;
        check(!owner.original().descriptor().container().blocks.is_empty())?;
    }
    Ok(())
}

#[test]
fn late_template_refusal_keeps_original_descriptor_program_and_actual_completed_unit()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<!--whole--><script setup>/*stock*/ const rows='ab'</script><template><i/><div v-bind='unknown'/></template>";
    let owner = lower_selected_setup_sfc_native(&arena, source, options());
    check(owner.admitted().is_none())?;
    check(!owner.original().issues().is_empty())?;
    let original = owner
        .original()
        .template()
        .ok_or("actual native prefix File")?;
    let syntax = original
        .retained_setup()
        .ok_or("whole Program retained after late failure")?;
    check(syntax.comments().count() == 1)?;
    check(syntax.program().ok_or("stock Program")?.body.len() == 1)?;
    check(core::ptr::eq(
        syntax.source().authored_root(),
        owner.original().descriptor().source(),
    ))?;
    check(original.view().is_err())?;
    let file = original.file().ok_or("normal incomplete prefix")?;
    check(file.units().len() == 1)?;
    check(file.artifact().node_count() != 0)?;
    Ok(())
}

#[test]
fn old_scriptless_entry_stays_closed_for_every_setup_spelling() -> Result<(), &'static str> {
    let arena = Allocator::default();
    for source in [
        "<template></template><script setup>const rows=1</script>",
        "<template></template><script setup lang=ts>const rows:number=1</script>",
        "<template></template><script setup>/*only*/</script>",
        "<template></template><script setup></script>",
    ] {
        let owner = lower_selected_sfc_native(&arena, source, options());
        check(owner.admitted().is_none())?;
        check(owner.template().is_none())?;
        check(
            owner
                .issues()
                .iter()
                .any(|issue| issue.kind == Kind::Script(ScriptRole::Setup)),
        )?;
    }
    Ok(())
}
