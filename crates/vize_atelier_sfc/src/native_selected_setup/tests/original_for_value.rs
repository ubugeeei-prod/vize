//! Whole original callback value custody and independent complete module maps.
use super::*;
use vize_l2::{op::Op, resolution::Usage};
use vize_l3::decision::dom::{DomChild, DomChildren, DomDependency};
mod custody;
mod refusal;

#[test]
fn six_whole_original_for_value_components_and_raw_maps_are_captured() -> Result<(), String> {
    let pack: Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/native_original_for_value_sfc_vue_3_5_35.json"
    ))
    .map_err(|e| cstr!("{e}"))?;
    let fixtures = pack["fixtures"].as_array().ok_or("fixtures")?;
    require!(
        fixtures.len() == 6,
        "six independent complete original sources"
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
            "{id}: genuine value carrier closure"
        );
        require!(
            dom.dependencies()
                == [
                    DomDependency::CollectionIteration,
                    DomDependency::FragmentValue,
                    DomDependency::BlockBoundary,
                    DomDependency::NativeElementBlock,
                    DomDependency::DisplayValue
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
        let DomChildren::Text(text) = &body.children else {
            return Err(cstr!("{id}: original dynamic singleton"));
        };
        let [body_node] = text.nodes.as_slice() else {
            return Err(cstr!("{id}: one current alias interpolation"));
        };
        require!(text.dynamic && body.changes.text, "{id}: actual TEXT1 fact");
        let body_row = analysis.expression(*body_node).ok_or("alias read row")?;
        let body_resolution = body_row.resolution();
        let table = body_resolution.table().ok_or("immutable original table")?;
        let [body_read] = body_row.reads() else {
            return Err(cstr!("{id}: one body read"));
        };
        require!(
            table.occurrences().len() == 1
                && core::ptr::eq(body_read.occurrence(), &table.occurrences()[0])
                && body_read.kind() == VueReadKind::ForValue
                && body_read.occurrence().usage == Usage::Read
                && body_read.binding().id() == alias.id()
                && body_read.binding().same_owner(alias)
                && body_resolution.scope() == head.scope()
                && core::ptr::eq(body_resolution.file(), file),
            "{id}: current original callback value and scope, never ScriptUnit"
        );
        let original_body = file
            .native_interpolation(*body_node)
            .ok_or("original operand")?;
        require!(
            core::ptr::eq(
                table.expression().ast,
                original_body
                    .input()
                    .operand()
                    .syntax()
                    .expression()
                    .ok_or("stock body root")?
            ) && core::ptr::eq(
                original_body
                    .input()
                    .operand()
                    .syntax()
                    .source()
                    .authored_root(),
                source
            ),
            "{id}: complete authentic original interpolation AST/source"
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
    if let Some(path) = std::env::var_os("VIZE_NATIVE_ORIGINAL_FOR_VALUE_SFC_DOM_CAPTURE") {
        std::fs::write(
            path,
            serde_json::to_vec_pretty(&serde_json::json!({
            "schema":"vize.native-sfc.original-for-value-capture",
            "adapter":"vize_atelier_sfc::compile_native_selected_setup_sfc_dom",
            "fixtures":captured}))
            .map_err(|e| cstr!("{e}"))?,
        )
        .map_err(|e| cstr!("{e}"))?;
    }
    Ok(())
}
