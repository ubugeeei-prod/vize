use super::*;
use serde_json::Value;
use vize_l0::{String, cstr, id::NodeId};
use vize_l3::decision::{dom::vue::VueReadKind, native::build_native_selected_setup_dom_decisions};

mod nested;
mod refusal;

macro_rules! require {
    ($condition:expr, $($message:tt)+) => {
        if !$condition { return Err(cstr!($($message)+)); }
    };
}
mod original_for;
mod original_for_value;

fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| cstr!("missing {key}"))
}

#[test]
fn seven_whole_original_setup_components_and_maps_are_captured() -> Result<(), String> {
    let pack: Value = serde_json::from_str(include_str!(
        "../../tests/fixtures/native_selected_setup_sfc_vue_3_5_35.json"
    ))
    .map_err(|e| cstr!("{e}"))?;
    let fixtures = pack["fixtures"].as_array().ok_or("fixtures")?;
    require!(
        fixtures.len() == 7,
        "six original JS/TS sources plus exact nested counterexample"
    );
    let mut rows = Vec::new();
    for fixture in fixtures {
        let id = text(fixture, "id")?;
        let source = text(fixture, "source")?;
        let arena = Allocator::default();
        let plain = compile_native_selected_setup_sfc_dom(
            &arena,
            source,
            NativeSelectedSfcDomOptions::default(),
        );
        let mapped = compile_native_selected_setup_sfc_dom(
            &arena,
            source,
            NativeSelectedSfcDomOptions {
                source_map: true,
                filename: text(&pack, "filename")?,
                ..NativeSelectedSfcDomOptions::default()
            },
        );
        let output = mapped.result().map_err(|e| cstr!("{id}: {e:?}"))?;
        let unlinked = plain.result().map_err(|e| cstr!("{id}: plain {e:?}"))?;
        require!(
            output.code() == text(fixture, "expectedCode")?,
            "{id}: complete independent desired module"
        );
        require!(
            unlinked.code() == output.code()
                && unlinked.document().links().is_empty()
                && unlinked.source_map().is_none(),
            "{id}: NoLinks byte equality"
        );
        let map: Value =
            serde_json::from_str(output.source_map().ok_or("map")?).map_err(|e| cstr!("{e}"))?;
        require!(
            map == fixture["nativeMap"],
            "{id}: complete independent authored-anchor map"
        );
        let admitted = mapped.observation().admitted().ok_or("own envelope")?;
        let setup = admitted.setup();
        let file = setup.file();
        require!(
            core::ptr::eq(setup.syntax().source().authored_root(), source),
            "{id}: full original source owner"
        );
        require!(
            core::ptr::eq(
                setup.syntax(),
                setup.owner().retained_setup().ok_or("whole syntax")?
            ),
            "{id}: same normally retained whole syntax"
        );
        require!(
            core::ptr::eq(
                setup.syntax().program().ok_or("Program")?.body.as_ptr(),
                setup.program().program().body.as_ptr()
            ),
            "{id}: actual Program body allocation"
        );
        require!(
            setup.syntax().diagnostics().count() == 0
                && file.is_complete()
                && file.units().len() == 1,
            "{id}: genuine completed original unit"
        );
        let names: Vec<_> = setup
            .bindings()
            .map(|b| b.declaration().map(|d| d.name.as_str()))
            .collect();
        let expected: Vec<_> = fixture["bindings"]
            .as_array()
            .ok_or("names")?
            .iter()
            .map(Value::as_str)
            .collect();
        require!(
            names == expected,
            "{id}: complete actual direct binding order"
        );
        let analysis =
            build_native_selected_setup_dom_decisions(setup).map_err(|e| cstr!("{id}: {e:?}"))?;
        require!(
            analysis.dom().ok_or("dom")?.unsupported().is_empty(),
            "{id}: eligible target"
        );
        let mut seen = 0;
        for index in 0..file.artifact().node_count() {
            let node = NodeId::from_index(index).ok_or("bounded actual node index")?;
            let Some(row) = analysis.expression(node) else {
                continue;
            };
            let resolution = row.resolution();
            let record = file
                .native_interpolation(node)
                .ok_or("original interpolation")?;
            let table = resolution.table().ok_or("whole immutable table")?;
            require!(
                core::ptr::eq(resolution.file(), file),
                "{id}: same File expression"
            );
            require!(
                core::ptr::eq(
                    table.expression().ast,
                    record
                        .input()
                        .operand()
                        .syntax()
                        .admitted_expression()
                        .ok_or("original expression")?
                        .expression()
                ),
                "{id}: original AST identity"
            );
            require!(
                row.reads().len() == 1 && table.occurrences().len() == 1,
                "{id}: one complete original occurrence"
            );
            let read = &row.reads()[0];
            require!(
                core::ptr::eq(read.occurrence(), &table.occurrences()[0]),
                "{id}: exact stored occurrence"
            );
            let binding = setup.binding(read.binding()).map_err(|e| cstr!("{e:?}"))?;
            let declaration = binding.declaration().ok_or("original declaration")?;
            require!(
                declaration.unit == setup.unit() && declaration.scope == setup.scope(),
                "{id}: real setup unit/scope"
            );
            let immutable = fixture["immutableBindings"]
                .as_array()
                .ok_or("const")?
                .iter()
                .any(|v| v.as_str() == Some(read.occurrence().name));
            require!(
                read.kind()
                    == if immutable {
                        VueReadKind::SetupConst
                    } else {
                        VueReadKind::SetupLet
                    },
                "{id}: real immutable/mutable classification"
            );
            seen += 1;
        }
        require!(seen == 1, "{id}: original sole expression row");
        for annotation in setup.type_annotations() {
            require!(
                core::ptr::eq(annotation.file(), file),
                "{id}: same-File annotation"
            );
            require!(
                !output
                    .document()
                    .links()
                    .iter()
                    .any(|l| l.authored.start < annotation.span().end
                        && annotation.span().start < l.authored.end),
                "{id}: only genuine TS annotation omitted"
            );
        }
        for link in output.document().links() {
            require!(
                source
                    .get(link.authored.start as usize..link.authored.end as usize)
                    .is_some()
                    && output
                        .code()
                        .get(link.generated.start as usize..link.generated.end as usize)
                        .is_some(),
                "{id}: full original valid UTF-8 links"
            );
        }
        rows.push(
            serde_json::json!({"id":id,"source":source,"code":output.code(),"nativeMap":map}),
        );
    }
    if let Some(path) = std::env::var_os("VIZE_NATIVE_SELECTED_SETUP_SFC_DOM_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&serde_json::json!({"schema":"vize.native-sfc.selected-setup-capture","adapter":"vize_atelier_sfc::compile_native_selected_setup_sfc_dom","fixtures":rows})).map_err(|e| cstr!("{e}"))?).map_err(|e| cstr!("{e}"))?;
    }
    Ok(())
}

#[test]
fn equal_local_ids_and_source_bytes_do_not_authorize_foreign_setup_bindings() {
    let arena = Allocator::default();
    let source = "<script setup>let count=1</script><template>{{count}}</template>";
    let first = compile_native_selected_setup_sfc_dom(
        &arena,
        source,
        NativeSelectedSfcDomOptions::default(),
    );
    let second = compile_native_selected_setup_sfc_dom(
        &arena,
        source,
        NativeSelectedSfcDomOptions::default(),
    );
    let a = first.observation().admitted().unwrap();
    let b = second.observation().admitted().unwrap();
    let own = a.setup().bindings().next().unwrap();
    let foreign = b.setup().bindings().next().unwrap();
    assert_eq!(own.id(), foreign.id());
    assert!(!own.same_owner(foreign));
    assert_eq!(
        a.setup().binding(foreign).err().unwrap().kind,
        vize_l2::file::vue::ExposureIssueKind::ForeignBinding
    );
    assert!(a.setup().binding(own).is_ok());
}

#[test]
fn moving_whole_compilation_keeps_normal_program_allocation_and_complete_output() {
    let arena = Allocator::default();
    let source = "<script setup lang=ts>let count:number=1</script><template>{{count}}</template>";
    let compiled = compile_native_selected_setup_sfc_dom(
        &arena,
        source,
        NativeSelectedSfcDomOptions {
            source_map: true,
            ..NativeSelectedSfcDomOptions::default()
        },
    );
    let before = compiled
        .observation()
        .admitted()
        .unwrap()
        .setup()
        .syntax()
        .program()
        .unwrap()
        .body
        .as_ptr();
    let moved = Box::new(compiled);
    assert_eq!(
        moved
            .observation()
            .admitted()
            .unwrap()
            .setup()
            .syntax()
            .program()
            .unwrap()
            .body
            .as_ptr(),
        before
    );
    assert!(moved.result().unwrap().code().contains("let count=1"));
    assert_eq!(
        serde_json::from_str::<Value>(moved.result().unwrap().source_map().unwrap()).unwrap()["sourcesContent"],
        serde_json::json!([source])
    );
}
