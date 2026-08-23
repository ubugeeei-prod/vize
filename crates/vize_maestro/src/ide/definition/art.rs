//! Shared definition helpers for `.art.vue` component metadata.

use crate::ide::{IdeContext, kebab_to_pascal};

pub(super) fn component_path(ctx: &IdeContext<'_>, component_name: &str) -> Option<String> {
    if !ctx.uri.path().ends_with(".art.vue") {
        return None;
    }

    let allocator = vize_carton::Allocator::new();
    let art_desc = vize_musea::parse_art(
        &allocator,
        &ctx.content,
        vize_musea::ArtParseOptions::default(),
    )
    .ok()?;
    let component_path = art_desc.metadata.component?;
    let descriptor = vize_atelier_sfc::parse_sfc(
        &ctx.content,
        vize_atelier_sfc::SfcParseOptions {
            filename: ctx.uri.path().to_string().into(),
            ..Default::default()
        },
    )
    .ok()?;
    if let Some(script_setup) = descriptor.script_setup.as_ref()
        && let Some(defined_component) =
            crate::virtual_code::find_define_art_component_name(script_setup.content.as_ref())
    {
        let pascal_component = kebab_to_pascal(component_name);
        if component_name == defined_component || pascal_component == defined_component {
            return Some(component_path.to_string());
        }
    }

    let stem = std::path::Path::new(component_path)
        .file_stem()
        .and_then(|stem| stem.to_str())?;

    let pascal_component = kebab_to_pascal(component_name);
    let pascal_stem = kebab_to_pascal(stem);
    (component_name == stem || pascal_component == pascal_stem).then(|| component_path.to_string())
}
