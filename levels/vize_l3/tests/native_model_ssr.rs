use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::{container::vue::DescriptorOptions, markup::SurfaceParseOptions};
use vize_l1_to_l2::native_file::lower_selected_setup_sfc_native;
use vize_l2::{
    file::{DeclarationKind, FileResolution, NativeFileModelState},
    resolution::Usage,
};
use vize_l3::decision::ssr::{
    SsrPart, SsrSetupExpression, SsrSetupRead, build_native_selected_setup_ssr_decisions,
};

fn descriptor() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: SurfaceParseOptions::default(),
    }
}

#[test]
fn seven_original_models_retain_write_identity_without_becoming_generic_reads() -> Result<(), String>
{
    let pack: serde_json::Value = serde_json::from_str(include_str!(
        "../../../crates/vize_atelier_sfc/tests/fixtures/native_original_input_model_inputs_vue_3_5_35.json"
    )).map_err(|error| format!("{error}"))?;
    let fixtures = pack["fixtures"].as_array().ok_or("original fixtures")?;
    assert_eq!(fixtures.len(), 7);
    for fixture in fixtures {
        let source = fixture["source"].as_str().ok_or("whole original source")?;
        let name = fixture["binding"].as_str().ok_or("authored binding")?;
        let arena = Allocator::default();
        let observation = lower_selected_setup_sfc_native(&arena, source, descriptor());
        let admitted = observation.admitted().ok_or("genuine setup completion")?;
        let setup = admitted.setup();
        let file = setup.file();
        let provenance = file.artifact().provenance().to_vec();
        let analysis = build_native_selected_setup_ssr_decisions(setup)
            .map_err(|error| format!("{error:?}"))?;
        let facts = analysis.ssr().ok_or("SSR decisions")?;
        assert!(
            facts.unsupported().is_empty(),
            "{source}: {:?}",
            facts.unsupported()
        );
        let [
            SsrPart::Open {
                node: root,
                element,
                void: true,
            },
        ] = facts.parts()
        else {
            return Err(format!("{source}: sole actual void opening"));
        };
        let [original] = file.native_models() else {
            return Err("one actual Model".into());
        };
        let NativeFileModelState::Admitted(node) = original.state() else {
            return Err("normally closed Model".into());
        };
        let expression = analysis
            .expression(node)
            .ok_or("canonical Model value role")?;
        assert!(
            expression.reads().is_empty(),
            "ReadWrite never becomes a normal SSR read"
        );
        let row = analysis
            .model(*root, element)
            .map_err(|error| format!("{error:?}"))?
            .ok_or("separate Model receipt")?;
        assert!(core::ptr::eq(row.original(), original));
        assert!(row.original().accepts(row.model()));
        assert!(row.original().accepts_element(*root, element));
        assert!(core::ptr::eq(row.resolution().file(), file));
        assert_eq!(row.resolution().node(), node);
        assert_eq!(row.resolution().scope(), Some(setup.scope()));
        assert_eq!(row.occurrence().usage, Usage::ReadWrite);
        assert_eq!(row.occurrence().name, name);
        assert_eq!(row.authored_span().slice(source), name);
        assert!(row.resolution().accepts(row.binding()));
        assert!(setup.binding(row.binding()).is_ok());
        let declaration = row.binding().declaration().ok_or("actual declaration")?;
        assert!(matches!(
            declaration.kind,
            DeclarationKind::Let | DeclarationKind::Var
        ));
        assert_eq!(declaration.unit, setup.unit());
        assert_eq!(declaration.scope, setup.scope());
        let [occurrence] = expression
            .resolution()
            .table()
            .ok_or("actual table")?
            .occurrences()
        else {
            return Err("one original semantic occurrence".into());
        };
        assert!(core::ptr::eq(row.occurrence(), occurrence));
        assert_eq!(file.artifact().provenance(), provenance);
    }
    Ok(())
}

#[test]
fn equal_foreign_ids_and_source_bytes_cannot_supply_an_opening_owner() -> Result<(), String> {
    let source_a = String::from(
        "<script setup>let draft='x'</script><template><input v-model='draft'/></template>",
    );
    let source_b = source_a.clone();
    let arena_a = Allocator::default();
    let arena_b = Allocator::default();
    let observation_a = lower_selected_setup_sfc_native(&arena_a, &source_a, descriptor());
    let observation_b = lower_selected_setup_sfc_native(&arena_b, &source_b, descriptor());
    let admitted_a = observation_a.admitted().ok_or("first setup")?;
    let admitted_b = observation_b.admitted().ok_or("second setup")?;
    let analysis_a = build_native_selected_setup_ssr_decisions(admitted_a.setup())
        .map_err(|error| format!("{error:?}"))?;
    let analysis_b = build_native_selected_setup_ssr_decisions(admitted_b.setup())
        .map_err(|error| format!("{error:?}"))?;
    let [
        SsrPart::Open {
            node: a,
            element: element_a,
            ..
        },
    ] = analysis_a.ssr().ok_or("first SSR")?.parts()
    else {
        return Err("first opening".into());
    };
    let [
        SsrPart::Open {
            node: b,
            element: element_b,
            ..
        },
    ] = analysis_b.ssr().ok_or("second SSR")?.parts()
    else {
        return Err("second opening".into());
    };
    assert_eq!(a, b);
    assert_eq!(source_a, source_b);
    assert!(!core::ptr::eq(*element_a, *element_b));
    assert!(analysis_a.model(*a, element_b).is_err());
    assert!(analysis_b.model(*b, element_a).is_err());
    assert!(
        analysis_a
            .model(*a, element_a)
            .map_err(|error| format!("{error:?}"))?
            .is_some()
    );
    assert!(
        analysis_b
            .model(*b, element_b)
            .map_err(|error| format!("{error:?}"))?
            .is_some()
    );
    Ok(())
}

#[test]
fn ordinary_read_receipts_and_model_absence_keep_their_existing_policy() -> Result<(), String> {
    let source = "<script setup>let draft='x'</script><template><p>{{draft}}</p></template>";
    let arena = Allocator::default();
    let observation = lower_selected_setup_sfc_native(&arena, source, descriptor());
    let admitted = observation.admitted().ok_or("actual setup")?;
    let analysis = build_native_selected_setup_ssr_decisions(admitted.setup())
        .map_err(|error| format!("{error:?}"))?;
    let facts = analysis.ssr().ok_or("SSR decisions")?;
    assert!(facts.unsupported().is_empty());
    let [
        SsrPart::Open { node, element, .. },
        SsrPart::Interpolation { node: read, .. },
        SsrPart::Close { .. },
    ] = facts.parts()
    else {
        return Err("original direct-child shape".into());
    };
    assert!(
        analysis
            .model(*node, element)
            .map_err(|error| format!("{error:?}"))?
            .is_none()
    );
    let [occurrence] = analysis
        .expression(*read)
        .ok_or("normal interpolation")?
        .reads()
    else {
        return Err("one normal read".into());
    };
    assert_eq!(occurrence.occurrence().usage, Usage::Read);
    Ok(())
}

#[test]
fn internal_access_tag_has_a_bounded_explicit_layout_cost() {
    type LegacyCarrier = (
        FileResolution<'static, 'static>,
        Option<SsrSetupRead<'static, 'static>>,
    );
    let before = core::mem::size_of::<LegacyCarrier>();
    let current = core::mem::size_of::<SsrSetupExpression<'static, 'static>>();
    eprintln!("SSR expression carrier before={before} current={current}");
    assert!(current <= before + core::mem::size_of::<usize>());
    assert_eq!(
        core::mem::align_of::<LegacyCarrier>(),
        core::mem::align_of::<SsrSetupExpression<'static, 'static>>()
    );
}
