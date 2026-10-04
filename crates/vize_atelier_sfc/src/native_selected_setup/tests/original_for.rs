//! Whole runtime qualification keeps original For/collection/alias custody.
use super::*;
use vize_l2::{op::Op, resolution::Usage};
use vize_l3::decision::dom::{DomChild, DomChildren, DomDependency};
mod refusal;

#[test]
fn ten_whole_original_for_components_and_maps_are_captured() -> Result<(), String> {
    let pack: Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/native_original_for_sfc_vue_3_5_35.json"
    ))
    .map_err(|e| cstr!("{e}"))?;
    let fixtures = pack["fixtures"].as_array().ok_or("fixtures")?;
    require!(
        fixtures.len() == 10,
        "ten independent complete original sources"
    );
    let mut captured = Vec::new();
    for fixture in fixtures {
        let id = text(fixture, "id")?;
        let source = text(fixture, "source")?;
        let arena = Allocator::default();
        let mapped = compile_native_selected_setup_sfc_dom(
            &arena,
            source,
            NativeSelectedSfcDomOptions {
                source_map: true,
                filename: text(&pack, "filename")?,
                ..NativeSelectedSfcDomOptions::default()
            },
        );
        let plain = compile_native_selected_setup_sfc_dom(
            &arena,
            source,
            NativeSelectedSfcDomOptions::default(),
        );
        let output = mapped.result().map_err(|e| cstr!("{id}: {e:?}"))?;
        let unlinked = plain.result().map_err(|e| cstr!("{id}: plain {e:?}"))?;
        require!(
            output.code() == text(fixture, "expectedCode")?,
            "{id}: whole desired module"
        );
        require!(
            unlinked.code() == output.code()
                && unlinked.document().links().is_empty()
                && unlinked.source_map().is_none(),
            "{id}: actual NoLinks equality"
        );
        let raw_map = output.source_map().ok_or("map")?;
        require!(
            raw_map == text(fixture, "nativeMapRaw")?,
            "{id}: complete serialized native map"
        );
        let map: Value = serde_json::from_str(raw_map).map_err(|e| cstr!("{e}"))?;
        require!(
            map == fixture["nativeMap"],
            "{id}: complete independent original map"
        );
        let view = mapped.observation().admitted().ok_or("same envelope")?;
        let setup = view.setup();
        let file = setup.file();
        let [Op::OriginalFor(original)] = file.artifact().root().ops.as_slice() else {
            return Err(cstr!("{id}: original root"));
        };
        let head = file.for_head_for(original).ok_or("attached original")?;
        let analysis =
            build_native_selected_setup_dom_decisions(setup).map_err(|e| cstr!("{id}: {e:?}"))?;
        let dom = analysis.dom().ok_or("dom")?;
        require!(
            dom.unsupported().is_empty(),
            "{id}: genuine static carrier closure"
        );
        require!(
            dom.dependencies()
                == [
                    DomDependency::CollectionIteration,
                    DomDependency::FragmentValue,
                    DomDependency::BlockBoundary,
                    DomDependency::NativeElementBlock
                ],
            "{id}: pinned first-use order"
        );
        let row = dom.file_for_head(original.id().node()).ok_or("For row")?;
        let resolution = head.resolution().ok_or("whole resolution")?;
        require!(
            row.accepts_original(original) && core::ptr::eq(row.resolution(), resolution),
            "{id}: actual allocation and retained resolution"
        );
        let read = row.collection_read().ok_or("collection Read")?;
        require!(
            core::ptr::eq(read.occurrence(), resolution.collection_occurrence())
                && read.occurrence().usage == Usage::Read
                && read.kind() == VueReadKind::SetupLet,
            "{id}: immutable actual collection occurrence"
        );
        let binding = setup
            .binding(read.binding())
            .map_err(|e| cstr!("{id}: {e:?}"))?;
        let declaration = binding.declaration().ok_or("real setup declaration")?;
        require!(
            declaration.unit == setup.unit()
                && declaration.scope == setup.scope()
                && binding.id() == resolution.collection().binding,
            "{id}: same selected setup declaration before alias scope"
        );
        let alias = head.value().ok_or("real alias")?;
        require!(
            alias.id() != binding.id()
                && alias.declaration().is_none()
                && setup.binding(alias).is_err(),
            "{id}: template alias is never ScriptUnit"
        );
        let template = alias.template_declaration().ok_or("template declaration")?;
        let original_alias = template.declaration().original().ok_or("original Params")?;
        require!(
            core::ptr::eq(
                original_alias.parameter(),
                resolution.value_declaration().parameter()
            ) && template.declaration().scope() == head.scope().ok_or("alias scope")?
                && head.enclosing_scope() != head.scope(),
            "{id}: original alias namespace/scopes"
        );
        let root = dom.node(original.id().node()).ok_or("root fact")?;
        require!(
            root.block_eligible,
            "{id}: original root block survives finish"
        );
        let DomChildren::Array(children) = &root.children else {
            return Err(cstr!("carrier"));
        };
        let [DomChild::Node(carrier)] = children.as_slice() else {
            return Err(cstr!("one carrier"));
        };
        let body = dom.node(*carrier).ok_or("closed carrier")?;
        require!(
            body.block_eligible && matches!(body.op(), Op::Element(e) if e.attributes.is_empty()),
            "{id}: existing body table and no synthetic node"
        );
        require!(
            core::ptr::eq(setup.syntax().source().authored_root(), source)
                && core::ptr::eq(
                    setup.syntax(),
                    setup.owner().retained_setup().ok_or("normal syntax")?
                ),
            "{id}: whole original setup ownership"
        );
        for link in output.document().links() {
            require!(
                source
                    .get(link.authored.start as usize..link.authored.end as usize)
                    .is_some()
                    && output
                        .code()
                        .get(link.generated.start as usize..link.generated.end as usize)
                        .is_some(),
                "{id}: complete UTF-8 source links"
            );
        }
        captured.push(
            serde_json::json!({"id":id,"source":source,"code":output.code(),"nativeMap":map,"nativeMapRaw":raw_map}),
        );
    }
    if let Some(path) = std::env::var_os("VIZE_NATIVE_ORIGINAL_FOR_SFC_DOM_CAPTURE") {
        std::fs::write(
            path,
            serde_json::to_vec_pretty(&serde_json::json!({
            "schema":"vize.native-sfc.original-for-capture",
            "adapter":"vize_atelier_sfc::compile_native_selected_setup_sfc_dom",
            "fixtures":captured}))
            .map_err(|e| cstr!("{e}"))?,
        )
        .map_err(|e| cstr!("{e}"))?;
    }
    Ok(())
}

#[test]
fn foreign_original_for_and_bindings_never_join_equal_source_or_numeric_ids() {
    let arena = Allocator::default();
    let source = "<script setup>let count=2</script><template><i v-for='count in count'>fixed</i></template>";
    let a = compile_native_selected_setup_sfc_dom(
        &arena,
        source,
        NativeSelectedSfcDomOptions::default(),
    );
    let b = compile_native_selected_setup_sfc_dom(
        &arena,
        source,
        NativeSelectedSfcDomOptions::default(),
    );
    let av = a.observation().admitted().unwrap();
    let bv = b.observation().admitted().unwrap();
    let [Op::OriginalFor(ao)] = av.setup().file().artifact().root().ops.as_slice() else {
        panic!("For")
    };
    let [Op::OriginalFor(bo)] = bv.setup().file().artifact().root().ops.as_slice() else {
        panic!("For")
    };
    let analysis = build_native_selected_setup_dom_decisions(av.setup()).unwrap();
    let row = analysis
        .dom()
        .unwrap()
        .file_for_head(ao.id().node())
        .unwrap();
    assert_eq!(ao.id(), bo.id());
    assert!(row.accepts_original(ao));
    assert!(!row.accepts_original(bo));
    let foreign = bv.setup().bindings().next().unwrap();
    assert_eq!(row.collection().id(), foreign.id());
    assert!(!row.collection().same_owner(foreign));
    assert!(av.setup().binding(foreign).is_err());
    assert!(av.setup().file().for_head_for(bo).is_none());
}

#[test]
fn moving_complete_for_output_keeps_whole_program_params_collection_and_maps() {
    let arena = Allocator::default();
    let source =
        "<script setup>let count=2</script><template><i v-for='item in count'>fixed</i></template>";
    let compiled = compile_native_selected_setup_sfc_dom(
        &arena,
        source,
        NativeSelectedSfcDomOptions {
            source_map: true,
            filename: "OriginalFor雪🌸.vue",
            ..NativeSelectedSfcDomOptions::default()
        },
    );
    let (program, collection, parameter) = {
        let view = compiled.observation().admitted().unwrap();
        let [Op::OriginalFor(op)] = view.setup().file().artifact().root().ops.as_slice() else {
            panic!("For")
        };
        let resolution = view
            .setup()
            .file()
            .for_head_for(op)
            .unwrap()
            .resolution()
            .unwrap();
        (
            view.setup().program().program().body.as_ptr(),
            resolution.input().collection() as *const _,
            resolution.value_declaration().parameter() as *const _,
        )
    };
    let moved = Box::new(compiled);
    let view = moved.observation().admitted().unwrap();
    let [Op::OriginalFor(op)] = view.setup().file().artifact().root().ops.as_slice() else {
        panic!("For")
    };
    let resolution = view
        .setup()
        .file()
        .for_head_for(op)
        .unwrap()
        .resolution()
        .unwrap();
    assert_eq!(program, view.setup().program().program().body.as_ptr());
    assert_eq!(collection, resolution.input().collection() as *const _);
    assert_eq!(
        parameter,
        resolution.value_declaration().parameter() as *const _
    );
    let pack: Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/native_original_for_sfc_vue_3_5_35.json"
    ))
    .unwrap();
    assert_eq!(
        moved.result().unwrap().code(),
        pack["fixtures"][0]["expectedCode"].as_str().unwrap()
    );
    assert_eq!(
        moved.result().unwrap().source_map().unwrap(),
        pack["fixtures"][0]["nativeMapRaw"].as_str().unwrap()
    );
    let map: Value = serde_json::from_str(moved.result().unwrap().source_map().unwrap()).unwrap();
    assert_eq!(map, pack["fixtures"][0]["nativeMap"]);
}
