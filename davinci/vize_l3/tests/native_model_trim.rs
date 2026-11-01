//! Actual original modifier custody reaches all three existing Model consumers.

use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::{container::vue::DescriptorOptions, markup::SurfaceParseOptions};
use vize_l1_to_l2::native_file::lower_selected_setup_sfc_native;
use vize_l2::{
    file::{DeclarationKind, NativeFileModelState},
    op::{BindingOp, Op},
    resolution::Usage,
};
use vize_l3::decision::{
    native::build_native_selected_setup_dom_decisions,
    ssr::build_native_selected_setup_ssr_decisions,
    vapor::{VaporPart, build_native_selected_setup_vapor_decisions},
};

fn descriptor() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: SurfaceParseOptions::default(),
    }
}

#[test]
fn original_trim_tokens_and_write_bindings_are_shared_without_generic_read_promotion()
-> Result<(), String> {
    let pack: serde_json::Value = serde_json::from_str(include_str!(
        "../../../crates/vize_atelier_sfc/tests/fixtures/native_original_input_model_trim_inputs.json"
    )).map_err(|error| format!("{error}"))?;
    let fixtures = pack["fixtures"].as_array().ok_or("new input fixtures")?;
    assert_eq!(fixtures.len(), 3);
    for fixture in fixtures {
        let source = fixture["source"].as_str().ok_or("source")?;
        let name = fixture["binding"].as_str().ok_or("binding")?;
        let arena = Allocator::default();
        let observation = lower_selected_setup_sfc_native(&arena, source, descriptor());
        let view = observation
            .admitted()
            .ok_or("normally closed original File")?;
        let setup = view.setup();
        let file = setup.file();
        let before = file.artifact().provenance().to_vec();
        let [original] = file.native_models() else {
            return Err("one original model".into());
        };
        let NativeFileModelState::Admitted(node) = original.state() else {
            return Err("closed model".into());
        };
        let element_node = original.element_node().ok_or("original element")?;
        let [Op::Element(element)] = file.artifact().root().ops.as_slice() else {
            return Err("root input".into());
        };
        let [BindingOp::Model(model)] = element.bindings.as_slice() else {
            return Err("one original contract".into());
        };
        let [modifier] = model.attributes.as_slice() else {
            return Err("one authored trim".into());
        };
        assert_eq!(modifier.name, "trim");
        assert_eq!(modifier.value, None);
        assert_eq!(Some(modifier.span), original.input().operand().trim_span());
        assert!(core::ptr::eq(modifier.name, modifier.span.slice(source)));

        let dom = build_native_selected_setup_dom_decisions(setup)
            .map_err(|error| format!("{error:?}"))?;
        let dom_facts = dom.dom().ok_or("DOM facts")?;
        assert!(
            dom_facts.unsupported().is_empty(),
            "{:?}",
            dom_facts.unsupported()
        );
        let dom_model = dom_facts.file_model(node).ok_or("DOM model")?;
        let ssr = build_native_selected_setup_ssr_decisions(setup)
            .map_err(|error| format!("{error:?}"))?;
        assert!(ssr.ssr().ok_or("SSR facts")?.unsupported().is_empty());
        let ssr_model = ssr
            .model(element_node, element)
            .map_err(|error| format!("{error:?}"))?
            .ok_or("SSR model")?;
        assert!(
            ssr.expression(node)
                .ok_or("SSR model expression")?
                .reads()
                .is_empty()
        );
        let vapor = build_native_selected_setup_vapor_decisions(setup)
            .map_err(|error| format!("{error:?}"))?;
        let vapor_facts = vapor.vapor().ok_or("Vapor facts")?;
        assert!(vapor_facts.unsupported().is_empty());
        let [root] = vapor_facts.roots() else {
            return Err("one Vapor root".into());
        };
        let [
            VaporPart::Open { void: true, .. },
            VaporPart::Model { model: vapor_model },
        ] = vapor_facts.parts(root).ok_or("Vapor parts")?
        else {
            return Err("genuine open/model".into());
        };
        for (resolution, binding, occurrence) in [
            (
                dom_model.resolution(),
                dom_model.binding(),
                dom_model.occurrence(),
            ),
            (
                ssr_model.resolution(),
                ssr_model.binding(),
                ssr_model.occurrence(),
            ),
            (
                vapor_model.resolution(),
                vapor_model.binding().ok_or("Vapor binding")?,
                vapor_model.occurrence().ok_or("Vapor occurrence")?,
            ),
        ] {
            assert!(core::ptr::eq(resolution.file(), file));
            assert_eq!(resolution.node(), node);
            assert_eq!(resolution.scope(), Some(setup.scope()));
            assert!(resolution.accepts(binding) && setup.binding(binding).is_ok());
            assert_eq!(occurrence.usage, Usage::ReadWrite);
            assert_eq!(occurrence.name, name);
            assert_eq!(original.binding(), Some(binding.id()));
            let declaration = binding.declaration().ok_or("actual direct declaration")?;
            assert!(matches!(
                declaration.kind,
                DeclarationKind::Let | DeclarationKind::Var
            ));
            assert_eq!(declaration.unit, setup.unit());
            assert_eq!(declaration.scope, setup.scope());
        }
        assert!(core::ptr::eq(dom_model.original(), original));
        assert!(core::ptr::eq(ssr_model.original(), original));
        assert!(core::ptr::eq(
            vapor_model.original().ok_or("Vapor original")?,
            original
        ));
        assert_eq!(file.artifact().provenance(), before);
    }
    Ok(())
}

#[test]
fn other_model_modifiers_and_arguments_remain_lower_refusals() {
    for head in [
        // Exact former single-number input is the new number fixture's first row.
        "v-model.number.number",
        "v-model.number.trim",
        "v-model.trim.number",
        "v-model.lazy.lazy",
        "v-model.lazy.number",
        "v-model.trim.trim",
        "v-model.trim.lazy",
        "v-model:arg.trim",
    ] {
        let source = format!(
            "<script setup>let draft=''</script><template><input {head}='draft'/></template>"
        );
        let arena = Allocator::default();
        let observation = lower_selected_setup_sfc_native(&arena, &source, descriptor());
        assert!(observation.admitted().is_none(), "{source}");
        let file = observation
            .original()
            .template()
            .and_then(|template| template.file());
        assert!(file.is_none_or(|file| !file.is_complete()));
    }
}
