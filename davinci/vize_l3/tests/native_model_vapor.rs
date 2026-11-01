//! A third consumer of the same completed original Model, not writable reads.

use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
    id::NodeId,
};
use vize_l1::{container::vue::DescriptorOptions, markup::SurfaceParseOptions};
use vize_l1_to_l2::native_file::lower_selected_setup_sfc_native;
use vize_l2::{
    file::{DeclarationKind, NativeFileModelState},
    op::InterpolationOp,
    resolution::Usage,
};
use vize_l3::decision::{
    build_decisions,
    policy::TargetPolicy,
    vapor::{
        VaporExpression, VaporPart, VaporUnsupported, build_native_selected_setup_vapor_decisions,
    },
};

fn descriptor() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: SurfaceParseOptions::default(),
    }
}

#[test]
fn original_models_retain_their_write_owner_and_neutral_analysis_cannot_promote_them()
-> Result<(), String> {
    let pack: serde_json::Value = serde_json::from_str(include_str!(
        "../../../crates/vize_atelier_sfc/tests/fixtures/native_original_input_model_inputs_vue_3_5_35.json"
    )).map_err(|error| format!("{error}"))?;
    let fixtures = pack["fixtures"].as_array().ok_or("original inputs")?;
    assert_eq!(fixtures.len(), 7);
    for fixture in fixtures {
        let source = fixture["source"].as_str().ok_or("whole source")?;
        let name = fixture["binding"].as_str().ok_or("authored target")?;
        let arena = Allocator::default();
        let observation = lower_selected_setup_sfc_native(&arena, source, descriptor());
        let selected = observation.admitted().ok_or("actual selected completion")?;
        let setup = selected.setup();
        let file = setup.file();
        let before = file.artifact().provenance().to_vec();
        let analysis = build_native_selected_setup_vapor_decisions(setup)
            .map_err(|error| format!("{error:?}"))?;
        let facts = analysis.vapor().ok_or("Vapor facts")?;
        assert!(
            facts.unsupported().is_empty(),
            "{source}: {:?}",
            facts.unsupported()
        );
        let [root] = facts.roots() else {
            return Err("one original root".into());
        };
        let [
            VaporPart::Open {
                node,
                element,
                void: true,
            },
            VaporPart::Model { model },
        ] = facts.parts(root).ok_or("actual contiguous root")?
        else {
            return Err("one original void input and canonical Model role".into());
        };
        assert_eq!(facts.inherit_attrs(), Some(*node));
        assert_eq!(root.node(), *node);
        let original = model.original().ok_or("original Model")?;
        assert!(original.accepts_element(*node, element));
        assert_eq!(
            original.state(),
            NativeFileModelState::Admitted(model.resolution().node())
        );
        let occurrence = model.occurrence().ok_or("original occurrence")?;
        let binding = model.binding().ok_or("actual binding")?;
        let declaration = binding.declaration().ok_or("actual declaration")?;
        assert_eq!(occurrence.usage, Usage::ReadWrite);
        assert_eq!(occurrence.name, name);
        assert_eq!(
            model.authored_span().ok_or("authored span")?.slice(source),
            name
        );
        assert!(matches!(
            declaration.kind,
            DeclarationKind::Let | DeclarationKind::Var
        ));
        assert_eq!(declaration.scope, setup.scope());
        assert_eq!(declaration.unit, setup.unit());
        assert!(model.resolution().accepts(binding));
        assert!(setup.binding(binding).is_ok());
        assert!(core::ptr::eq(model.resolution().file(), file));
        assert!(
            analysis
                .tables()
                .nodes
                .get(*node)
                .is_some_and(|row| row.dynamic_bindings.as_slice() == [model.resolution().node()])
        );
        assert_eq!(file.artifact().provenance(), before);
        let neutral = build_decisions(file.artifact(), TargetPolicy::Vapor)
            .map_err(|error| format!("{error:?}"))?;
        let neutral = neutral.vapor().ok_or("neutral facts")?;
        assert!(
            neutral
                .unsupported()
                .iter()
                .any(|row| row.reason == VaporUnsupported::ElementSemantics)
        );
        assert!(
            neutral
                .roots()
                .iter()
                .all(|root| neutral.parts(root).is_some_and(|parts| parts
                    .iter()
                    .all(|part| !matches!(part, VaporPart::Model { .. }))))
        );
    }
    Ok(())
}

#[test]
fn foreign_equal_roots_do_not_replace_original_model_ownership() -> Result<(), String> {
    let source =
        "<script setup>let draft='x'</script><template><input v-model='draft'/></template>";
    let arena = Allocator::default();
    let first = lower_selected_setup_sfc_native(&arena, source, descriptor());
    let second = lower_selected_setup_sfc_native(&arena, source, descriptor());
    let first_view = first.admitted().ok_or("first setup")?;
    let second_view = second.admitted().ok_or("second setup")?;
    let first_setup = first_view.setup();
    let second_setup = second_view.setup();
    let first = build_native_selected_setup_vapor_decisions(first_setup)
        .map_err(|error| format!("{error:?}"))?;
    let second = build_native_selected_setup_vapor_decisions(second_setup)
        .map_err(|error| format!("{error:?}"))?;
    let first_facts = first.vapor().ok_or("first facts")?;
    let second_facts = second.vapor().ok_or("second facts")?;
    let [first_root] = first_facts.roots() else {
        return Err("first root".into());
    };
    let [second_root] = second_facts.roots() else {
        return Err("second root".into());
    };
    assert_eq!(first_root.node(), second_root.node());
    assert!(first_facts.parts(second_root).is_none());
    assert!(second_facts.parts(first_root).is_none());
    let [
        VaporPart::Open { node, element, .. },
        VaporPart::Model { model },
    ] = second_facts.parts(second_root).ok_or("second parts")?
    else {
        return Err("second canonical parts".into());
    };
    let [first_original] = first.file().native_models() else {
        return Err("first original".into());
    };
    assert!(!first_original.accepts_element(*node, element));
    assert!(!core::ptr::eq(model.resolution().file(), first.file()));
    assert!(
        first_setup
            .binding(model.binding().ok_or("second binding")?)
            .is_err()
    );
    Ok(())
}

#[test]
fn ordinary_input_keeps_its_old_nonvoid_refusal_and_lexical_reads_stay_reads() -> Result<(), String>
{
    for template in ["<input/>", "{{draft}}"] {
        let source = format!("<script setup>let draft='x'</script><template>{template}</template>");
        let arena = Allocator::default();
        let observation = lower_selected_setup_sfc_native(&arena, &source, descriptor());
        let selected = observation.admitted().ok_or("actual setup")?;
        let setup = selected.setup();
        let analysis = build_native_selected_setup_vapor_decisions(setup)
            .map_err(|error| format!("{error:?}"))?;
        let facts = analysis.vapor().ok_or("facts")?;
        let [root] = facts.roots() else {
            return Err("one root".into());
        };
        let parts = facts.parts(root).ok_or("parts")?;
        if template == "<input/>" {
            assert_eq!(facts.unsupported().len(), 1);
            assert_eq!(
                facts.unsupported()[0].reason,
                VaporUnsupported::ElementSemantics
            );
            assert!(matches!(
                parts,
                [VaporPart::Open { void: false, .. }, VaporPart::Close { .. }]
            ));
        } else {
            assert!(facts.unsupported().is_empty());
            let [VaporPart::Interpolation { expression, .. }] = parts else {
                return Err("original normal read".into());
            };
            let [read] = expression
                .resolution()
                .table()
                .ok_or("table")?
                .occurrences()
            else {
                return Err("one read".into());
            };
            assert_eq!(read.usage, Usage::Read);
        }
    }
    Ok(())
}

#[test]
fn model_role_does_not_grow_the_existing_largest_part_payload() {
    type LargestOldPayload = (
        NodeId,
        &'static InterpolationOp<'static>,
        VaporExpression<'static, 'static>,
    );
    let previous_bound = core::mem::size_of::<LargestOldPayload>() + core::mem::size_of::<usize>();
    let current = core::mem::size_of::<VaporPart<'static, 'static>>();
    eprintln!("Vapor ordered part previous_bound={previous_bound} current={current}");
    assert!(current <= previous_bound);
}
