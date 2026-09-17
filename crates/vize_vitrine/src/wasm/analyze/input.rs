use super::to_sfc_utf16_range;
use vize_atelier_core::{ParserOptions, TemplateSyntaxMode, parser::parse_with_options};
use vize_atelier_sfc::{
    SfcDescriptor, SfcParseOptions,
    croquis::{SfcCroquisAnalysis, SfcCroquisOptions, analyze_sfc_descriptor_with_context},
    parse_sfc, prepare_root_patterned_template,
};
use vize_s0::Allocator;
#[cfg(test)]
mod tests;

pub(super) fn analyze<'a>(
    source: &'a str,
    filename: &str,
    in_tag_comments: bool,
    patterned_template: bool,
) -> Result<(SfcDescriptor<'a>, u32, SfcCroquisAnalysis), String> {
    let descriptor = parse_sfc(
        source,
        SfcParseOptions {
            filename: filename.into(),
            ..Default::default()
        },
    )
    .map_err(|error| error.message.to_string())?;
    let view = prepare_root_patterned_template(
        &descriptor,
        patterned_template,
        in_tag_comments,
        TemplateSyntaxMode::default(),
    )
    .map_err(|error| error.message.to_string())?;
    let offset = view
        .template
        .as_ref()
        .map_or(0, |template| template.loc.start as u32);
    let mut options = SfcCroquisOptions::full();
    options.analyzer_options.experimental_patterned_template = patterned_template;
    let analysis = if let Some(template) = &view.template {
        let allocator = Allocator::new();
        let (root, errors) = parse_with_options(
            &allocator,
            &template.content,
            ParserOptions {
                experimental_in_tag_comments: in_tag_comments,
                ..Default::default()
            },
        );
        if let Some(error) = errors.iter().find(|error| !error.is_recoverable()) {
            return Err(format!("Template parse error: {}", error.message));
        }
        analyze_sfc_descriptor_with_context(&view, Some(&root), options)
    } else {
        analyze_sfc_descriptor_with_context(&view, None, options)
    };
    drop(view);
    Ok((descriptor, offset, analysis))
}

pub(super) fn diagnostics(
    source: &str,
    offset: u32,
    summary: &vize_croquis::Croquis,
) -> Vec<serde_json::Value> {
    summary
        .pattern_diagnostics
        .iter()
        .map(|diagnostic| {
            let (start, end) = to_sfc_utf16_range(source, offset, diagnostic.start, diagnostic.end);
            serde_json::json!({
                "severity": if diagnostic.warning { "warning" } else { "error" },
                "message": diagnostic.message.as_str(),
                "start": start,
                "end": end,
                "code": "patterned-template",
                "related": [],
            })
        })
        .collect()
}
