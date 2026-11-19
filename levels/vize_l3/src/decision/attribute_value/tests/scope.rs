use super::*;
use vize_l2::op::BindingOp;

#[test]
fn genuine_js_ts_setup_for_alias_and_original_handler_scopes_survive_value_consumption() -> Test {
    for lang in ["", " lang='ts'"] {
        let arena = Allocator::default();
        let source = alloc::format!(
            "<script setup{lang}>let items=2</script><template><div title='&amp;lt;' v-for='item in items' @click='$event.count++'>x</div></template>"
        );
        let mut original = owner(&arena, &source)?;
        original
            .parse_setup_program()
            .map_err(|_| "whole original Program")?;
        {
            let mut walk = original
                .begin_setup()
                .map_err(|_| "same-owner setup cursor")?;
            for child in walk.selected().children() {
                walk.child(child).map_err(|_| "genuine original child")?;
            }
            walk.complete().map_err(|_| "normal completion")?;
        }
        let original = original.finish();
        let setup = original.setup().map_err(|_| "same normal setup")?;
        let file = setup.file();
        let [Op::OriginalFor(actual_for)] = file.artifact().root().ops.as_slice() else {
            return Err("original For allocation");
        };
        let head = file
            .for_head_for(actual_for)
            .ok_or("genuine original head")?;
        let [Op::Element(actual)] = actual_for.region.ops.as_slice() else {
            return Err("original For Element");
        };
        let [BindingOp::On(on)] = actual.bindings.as_slice() else {
            return Err("original On");
        };
        let handler = file
            .handler_for(on)
            .ok_or("same original handler allocation")?;
        let resolution = handler.resolution().ok_or("whole HandlerBody resolution")?;
        let dom = native::build_native_selected_setup_dom_decisions(&setup)
            .map_err(|_| "sole genuine DOM cursor")?;
        let ssr = ssr::build_native_selected_setup_ssr_decisions(&setup)
            .map_err(|_| "sole genuine SSR cursor")?;
        let vapor = vapor::build_native_selected_setup_vapor_decisions(&setup)
            .map_err(|_| "sole genuine Vapor cursor")?;
        for facts in [
            dom.original_attributes(),
            ssr.original_attributes(),
            vapor.original_attributes(),
        ] {
            let facts = facts.ok_or("complete whole row gate")?;
            let value = facts.value(0, actual, 0).ok_or("same actual allocation")?;
            check(facts.len() == 1 && core::ptr::eq(value.file(), file))?;
            check(value.observation().ok_or("original value")?.source().text() == "&lt;")?;
        }
        check(handler.scope() == head.scope() && head.scope() != head.enclosing_scope())?;
        check(core::ptr::eq(
            file.handler_for(on)
                .ok_or("retained handler")?
                .resolution()
                .ok_or("retained resolution")?,
            resolution,
        ))?;
        check(resolution.input().operand().raw_value() == "$event.count++")?;
        check(core::ptr::eq(
            setup.syntax(),
            original.retained_setup().ok_or("retained whole Program")?,
        ))?;
        // Original custody never lifts the existing attribute-bearing For target
        // refusal. Its eventual bounded runtime family remains attribute-free.
        check(!dom.dom().ok_or("DOM facts")?.unsupported().is_empty())?;
        check(!ssr.ssr().ok_or("SSR facts")?.unsupported().is_empty())?;
        check(!vapor.vapor().ok_or("Vapor facts")?.unsupported().is_empty())?;
    }
    Ok(())
}
