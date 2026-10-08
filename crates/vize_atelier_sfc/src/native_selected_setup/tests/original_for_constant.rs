//! Complete independent stable-list modules; no native oracle regeneration.
use super::*;
use vize_l2::{file::DeclarationKind, op::Op, resolution::Usage};
use vize_l3::decision::dom::{DomChild, DomChildren, DomDependency};
pub(super) mod fixture;
mod refusal;

#[test]
fn eleven_whole_original_constant_for_components_and_raw_maps_are_captured() -> Result<(), String> {
    let pack = fixture::pack()?;
    let fixtures = &pack.fixtures;
    let faults = &pack.range_controls;
    require!(
        fixtures.len() == 10 && faults.len() == 1,
        "whole independent originals"
    );
    let inherited = fixture::inherited_pack()?;
    require!(
        inherited.fixtures.len() == 1
            && inherited.range_controls.is_empty()
            && inherited.filename == pack.filename,
        "exact inherited whole input"
    );
    let mut inherited_captured = Vec::new();
    let mut captured = Vec::new();
    let mut attempts = Vec::new();
    let mut failures = Vec::new();
    for (index, fixture) in fixtures
        .iter()
        .chain(faults)
        .chain(&inherited.fixtures)
        .enumerate()
    {
        let id = fixture.id.as_str();
        let source = fixture.source.as_str();
        let arena = Allocator::default();
        let mapped = compile_native_selected_setup_sfc_dom(
            &arena,
            source,
            NativeSelectedSfcDomOptions {
                source_map: true,
                filename: &pack.filename,
                ..NativeSelectedSfcDomOptions::default()
            },
        );
        let plain = compile_native_selected_setup_sfc_dom(
            &arena,
            source,
            NativeSelectedSfcDomOptions::default(),
        );
        // Observe every actual result before validating any desired output.
        let observed = |compilation: &NativeSelectedSetupSfcDomCompilation<'_>| match compilation
            .result()
        {
            Ok(output) => serde_json::json!({
                "status":"success", "code":output.code(),
                "nativeMapRaw":output.source_map(),
                "nativeMap":output.source_map().and_then(|raw| serde_json::from_str::<Value>(raw).ok())
            }),
            Err(error) => serde_json::json!({"status":"typed-error", "error":cstr!("{error:?}")}),
        };
        let mapped_result = observed(&mapped);
        let plain_result = observed(&plain);
        let validated = (|| -> Result<Value, String> {
            let output = mapped.result().map_err(|e| cstr!("{id}: {e:?}"))?;
            let unlinked = plain.result().map_err(|e| cstr!("{id}: plain {e:?}"))?;
            require!(
                output.code() == fixture.expected_code,
                "{id}: full desired module"
            );
            require!(
                unlinked.code() == output.code()
                    && unlinked.source_map().is_none()
                    && unlinked.document().links().is_empty(),
                "{id}: actual NoLinks equality"
            );
            let raw = output.source_map().ok_or("raw map")?;
            let map: Value = serde_json::from_str(raw).map_err(|e| cstr!("{e}"))?;
            require!(
                raw == fixture.native_map_raw && map == fixture.native_map,
                "{id}: full independently authored serialized/object maps"
            );
            let view = mapped.observation().admitted().ok_or("normal owner")?;
            let setup = view.setup();
            let file = setup.file();
            let [Op::OriginalFor(original)] = file.artifact().root().ops.as_slice() else {
                return Err(cstr!("{id}: original root"));
            };
            let head = file.for_head_for(original).ok_or("attached For")?;
            let resolution = head.resolution().ok_or("whole resolution")?;
            let analysis =
                build_native_selected_setup_dom_decisions(setup).map_err(|e| cstr!("{e:?}"))?;
            let dom = analysis.dom().ok_or("DOM")?;
            require!(
                dom.unsupported().is_empty(),
                "{id}: complete constant closure"
            );
            let row = dom.file_for_head(original.id().node()).ok_or("For row")?;
            let read = row.collection_read().ok_or("collection Read")?;
            require!(
                row.accepts_original(original)
                    && core::ptr::eq(row.resolution(), resolution)
                    && core::ptr::eq(read.occurrence(), resolution.collection_occurrence())
                    && read.occurrence().usage == Usage::Read
                    && read.kind() == VueReadKind::SetupConst,
                "{id}: authentic constant collection occurrence"
            );
            let binding = setup.binding(read.binding()).map_err(|e| cstr!("{e:?}"))?;
            let declaration = binding.declaration().ok_or("setup declaration")?;
            require!(
                declaration.kind == DeclarationKind::Const
                    && declaration.initializer.is_primitive()
                    && declaration.unit == setup.unit()
                    && declaration.scope == setup.scope()
                    && binding.id() == resolution.collection().binding,
                "{id}: genuine whole setup Const, never numeric inference"
            );
            let root = dom.node(original.id().node()).ok_or("root fact")?;
            let DomChildren::Array(children) = &root.children else {
                return Err(cstr!("carrier"));
            };
            let [DomChild::Node(body)] = children.as_slice() else {
                return Err(cstr!("one carrier"));
            };
            let body = dom.node(*body).ok_or("body")?;
            require!(
                root.block_eligible
                    && !body.block_eligible
                    && matches!(body.op(), Op::Element(element) if element.attributes.is_empty()),
                "{id}: existing stable root block and direct child vnode"
            );
            let dynamic = fixture.dynamic;
            require!(body.changes.text == dynamic, "{id}: exact TEXT contract");
            if dynamic {
                let DomChildren::Text(text) = &body.children else {
                    return Err(cstr!("dynamic body"));
                };
                let [node] = text.nodes.as_slice() else {
                    return Err(cstr!("singleton"));
                };
                let expression = analysis.expression(*node).ok_or("original read")?;
                let [read] = expression.reads() else {
                    return Err(cstr!("sole actual read"));
                };
                require!(
                    read.kind() == VueReadKind::ForValue
                        && read
                            .binding()
                            .same_owner(head.value().ok_or("value declaration")?)
                        && expression.resolution().scope() == head.scope(),
                    "{id}: current scope value"
                );
            }
            let mut demands = vec![
                DomDependency::CollectionIteration,
                DomDependency::FragmentValue,
                DomDependency::BlockBoundary,
                DomDependency::NativeElementBlock,
            ];
            if dynamic {
                demands.push(DomDependency::DisplayValue);
            }
            demands.push(DomDependency::NativeElementValue);
            require!(
                dom.dependencies() == demands,
                "{id}: actual first-use helper demands"
            );
            require!(
                core::ptr::eq(setup.syntax().source().authored_root(), source)
                    && core::ptr::eq(
                        setup.syntax(),
                        setup.owner().retained_setup().ok_or("normal syntax")?
                    ),
                "{id}: full normal Program/source owner"
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
                    "{id}: whole UTF8 links"
                );
            }
            Ok(
                serde_json::json!({"id":id,"source":source,"code":output.code(),"nativeMap":map,"nativeMapRaw":raw}),
            )
        })();
        attempts.push(serde_json::json!({
            "id":id, "source":source, "mapped":mapped_result, "plain":plain_result,
            "validationError":validated.as_ref().err()
        }));
        match validated {
            Ok(row) if index < 11 => captured.push(row),
            Ok(row) => inherited_captured.push(row),
            Err(error) => failures.push(error),
        }
    }
    if let Some(path) = std::env::var_os("VIZE_NATIVE_ORIGINAL_FOR_CONSTANT_SFC_DOM_RAW_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&serde_json::json!({
            "schema":"vize.native-sfc.original-for-constant-unqualified-attempts",
            "qualified":false, "adapter":"vize_atelier_sfc::compile_native_selected_setup_sfc_dom",
            "attempts":attempts
        })).map_err(|error| cstr!("{error}"))?).map_err(|error| cstr!("{error}"))?;
    }
    require!(
        failures.is_empty() && captured.len() == 11 && inherited_captured.len() == 1,
        "all eleven plus inherited actual attempts: {failures:?}"
    );
    if let Some(path) = std::env::var_os("VIZE_NATIVE_ORIGINAL_FOR_CONSTANT_SFC_DOM_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&serde_json::json!({
            "schema":"vize.native-sfc.original-for-constant-capture",
            "adapter":"vize_atelier_sfc::compile_native_selected_setup_sfc_dom", "fixtures":captured, "inheritedControls":inherited_captured
        })).map_err(|e| cstr!("{e}"))?).map_err(|e| cstr!("{e}"))?;
    }
    Ok(())
}
