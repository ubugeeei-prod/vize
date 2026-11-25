//! The N-API result structs carry no `PartialEq`; these spell out every field.

use super::super::BatchFileResultNapi;

pub(super) type Text = vize_l0::CompactString;

pub(super) fn styles(result: &BatchFileResultNapi) -> Vec<Text> {
    result
        .styles
        .iter()
        .map(|block| {
            vize_l0::cstr!(
                "{:?}",
                (
                    &block.content,
                    &block.src,
                    &block.lang,
                    block.scoped,
                    block.module,
                    &block.module_name,
                    block.index
                )
            )
        })
        .collect()
}

pub(super) fn custom_blocks(result: &BatchFileResultNapi) -> Vec<Text> {
    result
        .custom_blocks
        .iter()
        .map(|block| {
            let attrs: Vec<_> = block
                .attrs
                .iter()
                .map(|attr| (&attr.name, &attr.value))
                .collect();
            vize_l0::cstr!(
                "{:?}",
                (
                    &block.block_type,
                    &block.content,
                    &block.src,
                    attrs,
                    block.index
                )
            )
        })
        .collect()
}

pub(super) fn macro_artifacts(result: &BatchFileResultNapi) -> Vec<Text> {
    result
        .macro_artifacts
        .iter()
        .map(|artifact| {
            vize_l0::cstr!(
                "{:?}",
                (
                    &artifact.kind,
                    &artifact.name,
                    &artifact.source,
                    &artifact.content,
                    &artifact.module_code,
                    artifact.start,
                    artifact.end
                )
            )
        })
        .collect()
}

pub(super) fn module_shape(result: &BatchFileResultNapi) -> Option<Text> {
    result.module_shape.as_ref().map(|shape| {
        let imports: Vec<_> = shape
            .imports
            .iter()
            .map(|import| (&import.source, &import.local, import.namespace))
            .collect();
        vize_l0::cstr!(
            "{:?}",
            (
                (
                    shape.has_default_export,
                    shape.has_sfc_main_defined,
                    shape.has_named_render_export,
                    shape.has_named_ssr_render_export,
                ),
                (
                    shape.default_export_start,
                    shape.default_export_keyword_end,
                    shape.default_export_end,
                    shape.default_export_is_sfc_main,
                    &shape.default_export_local,
                ),
                imports,
                (
                    shape.has_side_effect_import,
                    shape.has_reexport,
                    shape.has_dynamic_import
                ),
            )
        )
    })
}

pub(super) fn type_project(result: &BatchFileResultNapi) -> Option<(String, String)> {
    result
        .type_project
        .as_ref()
        .map(|project| (project.root.clone(), project.revision.clone()))
}
