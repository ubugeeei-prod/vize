use tower_lsp::lsp_types::Hover;

use super::HoverBuilder;
use crate::ide::IdeContext;

pub(super) fn hover_attribute(ctx: &IdeContext<'_>) -> Option<Hover> {
    let (attr_name, component_name) =
        crate::ide::definition::helpers::get_attribute_and_component_at_offset(ctx)?;
    if !crate::ide::is_component_tag(&component_name) {
        return None;
    }

    let import_path = crate::ide::definition::helpers::find_import_path(ctx, &component_name)?;
    let resolved_path =
        crate::ide::definition::helpers::resolve_import_path(ctx.uri, &import_path)?;
    let component_content = std::fs::read_to_string(&resolved_path).ok()?;
    let descriptor = vize_atelier_sfc::parse_sfc(
        &component_content,
        vize_atelier_sfc::SfcParseOptions {
            filename: resolved_path.to_string_lossy().to_string().into(),
            ..Default::default()
        },
    )
    .ok()?;
    let script_setup = descriptor.script_setup.as_ref()?;
    let script = script_setup.content.as_ref();
    let prop_name = crate::ide::definition::helpers::kebab_to_camel(&attr_name);
    let define_props_pos = script.find("defineProps")?;
    let after_define_props = &script[define_props_pos..];
    let prop_pos =
        crate::ide::definition::helpers::find_prop_in_define_props(after_define_props, &prop_name)?;
    let signature = prop_signature_at(script, define_props_pos + prop_pos, &prop_name);

    Some(
        HoverBuilder::new()
            .title(&prop_name)
            .meta("Component prop")
            .code("typescript", &signature)
            .build(),
    )
}

fn prop_signature_at(script: &str, offset: usize, prop_name: &str) -> String {
    let line_start = script[..offset]
        .rfind('\n')
        .map(|index| index + 1)
        .unwrap_or(0);
    let line_end = script[offset..]
        .find('\n')
        .map(|index| offset + index)
        .unwrap_or(script.len());
    let line = script[line_start..line_end]
        .trim()
        .trim_end_matches(',')
        .trim();
    if line.is_empty() {
        prop_name.to_string()
    } else {
        line.to_string()
    }
}
