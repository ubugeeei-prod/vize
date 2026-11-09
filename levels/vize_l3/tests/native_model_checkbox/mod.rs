//! Real static type ownership reaches all three existing mutable Model consumers.

use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::{container::vue::DescriptorOptions, markup::SurfaceParseOptions};
use vize_l1_to_l2::native_file::lower_selected_setup_sfc_native;
use vize_l2::{
    file::{DeclarationKind, InitializerKind, NativeFileModelKind, NativeFileModelState},
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
fn original_checkbox_type_and_real_mutable_binding_reach_all_three_consumers() -> Result<(), String>
{
    let pack: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../crates/vize_atelier_sfc/tests/fixtures/native_original_input_model_checkbox_inputs.json"
    )).map_err(|error| format!("{error}"))?;
    let fixtures = pack["fixtures"]
        .as_array()
        .ok_or("three new physical originals")?;
    assert_eq!(fixtures.len(), 3);
    let first = fixtures.first().ok_or("exact inline successor")?;
    assert_eq!(
        first["source"],
        "<script setup>let draft=''</script><template><input v-model='draft' type='checkbox'/></template>"
    );
    assert_eq!(
        first["successorOf"]["historicalCompleteNativePacket"],
        false
    );
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
            return Err("one original Model".into());
        };
        let NativeFileModelState::Admitted(node) = original.state() else {
            return Err("closed Model".into());
        };
        assert_eq!(original.kind(), Some(NativeFileModelKind::Checkbox));
        let element_node = original.element_node().ok_or("Element")?;
        let [Op::Element(element)] = file.artifact().root().ops.as_slice() else {
            return Err("root input".into());
        };
        let [BindingOp::Model(model)] = element.bindings.as_slice() else {
            return Err("actual Model".into());
        };
        assert!(model.argument.is_none() && model.attributes.is_empty());
        let [attribute] = element.attributes.as_slice() else {
            return Err("one original type".into());
        };
        let value = file
            .native_attribute_value_for(0, element, 0)
            .ok_or("closed static type owner")?;
        let original_value = value.observation().ok_or("original source")?;
        assert_eq!(attribute.name, "type");
        assert_eq!(attribute.value, Some("checkbox"));
        assert_eq!(original_value.name_span().slice(source), "type");
        assert_eq!(original_value.source().span().slice(source), "checkbox");
        assert!(original_value.source().decode_map().is_none());
        assert!(core::ptr::eq(
            original_value.source().authored_root(),
            source
        ));
        assert!(core::ptr::eq(
            value.attribute().ok_or("actual slot")?,
            attribute
        ));
        let dom = build_native_selected_setup_dom_decisions(setup)
            .map_err(|error| format!("{error:?}"))?;
        let dom_facts = dom.dom().ok_or("DOM facts")?;
        assert!(
            dom_facts.unsupported().is_empty(),
            "{:?}",
            dom_facts.unsupported()
        );
        let dom_model = dom_facts.file_model(node).ok_or("DOM Model")?;
        let ssr = build_native_selected_setup_ssr_decisions(setup)
            .map_err(|error| format!("{error:?}"))?;
        assert!(ssr.ssr().ok_or("SSR facts")?.unsupported().is_empty());
        let ssr_model = ssr
            .model(element_node, element)
            .map_err(|error| format!("{error:?}"))?
            .ok_or("SSR Model")?;
        assert!(
            ssr.expression(node)
                .ok_or("dedicated Model value")?
                .reads()
                .is_empty()
        );
        let vapor = build_native_selected_setup_vapor_decisions(setup)
            .map_err(|error| format!("{error:?}"))?;
        let facts = vapor.vapor().ok_or("Vapor facts")?;
        assert!(facts.unsupported().is_empty(), "{:?}", facts.unsupported());
        let [root] = facts.roots() else {
            return Err("one root".into());
        };
        let [
            VaporPart::Open { void: true, .. },
            VaporPart::Model { model: vapor_model },
        ] = facts.parts(root).ok_or("Vapor parts")?
        else {
            return Err("genuine void Model".into());
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
                vapor_model.binding().ok_or("binding")?,
                vapor_model.occurrence().ok_or("occurrence")?,
            ),
        ] {
            assert!(core::ptr::eq(resolution.file(), file));
            assert_eq!(resolution.node(), node);
            assert_eq!(resolution.scope(), Some(setup.scope()));
            assert!(resolution.accepts(binding) && setup.binding(binding).is_ok());
            assert_eq!(occurrence.usage, Usage::ReadWrite);
            assert_eq!(occurrence.name, name);
            assert_eq!(original.binding(), Some(binding.id()));
            let declaration = binding.declaration().ok_or("actual declaration")?;
            assert!(declaration.is_direct_program());
            assert!(matches!(
                declaration.kind,
                DeclarationKind::Let | DeclarationKind::Var
            ));
            assert_eq!(declaration.initializer, InitializerKind::PrimitiveLiteral);
            assert_eq!(declaration.unit, setup.unit());
            assert_eq!(declaration.scope, setup.scope());
        }
        assert!(core::ptr::eq(dom_model.original(), original));
        assert!(core::ptr::eq(ssr_model.original(), original));
        assert!(core::ptr::eq(
            vapor_model.original().ok_or("Vapor owner")?,
            original
        ));
        assert_eq!(file.artifact().provenance(), before);
    }
    Ok(())
}

#[test]
fn all_derived_checkbox_outside_inputs_remain_lower_refusals() -> Result<(), String> {
    let pack: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../crates/vize_atelier_sfc/tests/fixtures/native_original_input_model_checkbox_refusal_inputs.json"
    )).map_err(|error| format!("{error}"))?;
    let cases = pack["cases"].as_array().ok_or("derived outside inputs")?;
    assert_eq!(cases.len(), 16);
    for case in cases {
        let source = case["source"].as_str().ok_or("source")?;
        let arena = Allocator::default();
        let observation = lower_selected_setup_sfc_native(&arena, source, descriptor());
        assert!(observation.admitted().is_none(), "{source}");
        let file = observation
            .original()
            .template()
            .and_then(|template| template.file());
        assert!(file.is_none_or(|file| !file.is_complete()));
        if let Some(file) = file {
            for original in file.native_models() {
                assert_eq!(original.kind(), None);
            }
        }
    }
    Ok(())
}
