//! Recorded binding spans map to the acknowledged canonical checker document.

use crate::ide::{IdeContext, corsa_support, position_to_offset};
use std::collections::BTreeMap;
use vize_atelier_sfc::croquis::{
    SfcCroquisAnalysis, SfcCroquisOptions, script_content_for_descriptor,
};
use vize_canon::CorsaBridge;
use vize_croquis::{Croquis, ScopeData};
use vize_l0::String;
use vize_relief::BindingType;

pub(super) fn script_analysis(
    ctx: &IdeContext<'_>,
    croquis: Croquis,
) -> Option<SfcCroquisAnalysis> {
    let (script_content, script_offset) =
        script_content_for_descriptor(ctx.descriptor()?, SfcCroquisOptions::full());
    Some(SfcCroquisAnalysis {
        croquis,
        script_content,
        script_offset,
    })
}

pub(super) async fn classify(
    ctx: &IdeContext<'_>,
    analysis: &SfcCroquisAnalysis,
    bridge: &CorsaBridge,
) -> BTreeMap<String, bool> {
    classify_inner(ctx, analysis, bridge)
        .await
        .unwrap_or_default()
}

async fn classify_inner(
    ctx: &IdeContext<'_>,
    analysis: &SfcCroquisAnalysis,
    bridge: &CorsaBridge,
) -> Option<BTreeMap<String, bool>> {
    if !analysis.croquis.bindings.is_script_setup || ctx.state.legacy_vue2_enabled() {
        return None;
    }
    let descriptor = ctx.descriptor()?;
    let script = analysis.script_content_ref()?;
    let module = analysis.croquis.scopes.iter().find_map(|scope| {
        let ScopeData::ExternalModule(import) = scope.data() else {
            return None;
        };
        if import.is_type_only || import.source != "vue" {
            return None;
        }
        let statement = script.get(scope.span.start as usize..scope.span.end as usize)?;
        let offset = statement
            .rfind("\"vue\"")
            .or_else(|| statement.rfind("'vue'"))?;
        Some(
            analysis.script_source_offset(descriptor, scope.span.start + offset as u32 + 1)
                as usize,
        )
    })?;
    let candidates = analysis
        .croquis
        .bindings
        .bindings
        .iter()
        .filter_map(|(name, kind)| {
            if matches!(kind, BindingType::Props | BindingType::LiteralConst)
                || !(name.chars().next().is_some_and(char::is_uppercase)
                    || crate::ide::definition::component_import::resolve_component_file(ctx, name)
                        .is_some())
            {
                return None;
            }
            let &(start, end) = analysis.croquis.binding_spans.get(name)?;
            let start = analysis.script_source_offset(descriptor, start) as usize;
            let end = analysis.script_source_offset(descriptor, end) as usize;
            (ctx.content.get(start..end) == Some(name.as_str())).then_some((name.clone(), start))
        })
        .collect::<Vec<_>>();
    if candidates.is_empty() {
        return None;
    }
    let document = corsa_support::open_canonical_virtual_document(ctx, bridge).await?;
    let position = |offset| {
        let (line, character) =
            corsa_support::canonical_source_offset_to_position(&document, offset)?;
        let byte = position_to_offset(&document.virtual_result.code, line, character)?;
        Some((
            u32::try_from(
                document
                    .virtual_result
                    .code
                    .get(..byte)?
                    .encode_utf16()
                    .count(),
            )
            .ok()?,
            byte,
        ))
    };
    let (module, _) = position(module)?;
    let requests = candidates
        .into_iter()
        .filter_map(|(name, offset)| {
            let (position, byte) = position(offset)?;
            let end = byte.checked_add(name.len())?;
            (document.virtual_result.code.get(byte..end) == Some(name.as_str()))
                .then_some((name, position))
        })
        .collect::<Vec<_>>();
    let positions = requests
        .iter()
        .map(|(_, position)| *position)
        .collect::<Vec<_>>();
    let values = bridge
        .component_types(
            &document.request_uri,
            &document.virtual_result.code,
            module,
            &positions,
        )
        .await
        .ok()
        .flatten()?;
    if values.len() != requests.len() {
        return None;
    }
    Some(
        requests
            .into_iter()
            .zip(values)
            .filter_map(|((name, _), value)| value.map(|value| (name, value)))
            .collect(),
    )
}
