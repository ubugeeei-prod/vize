//! Module-scope facts collected from normal Vue `<script>` blocks.

use vize_carton::{CompactString, String as VizeString, append};

pub(super) fn collect_normal_script_named_value_exports(
    facts: Option<&vize_atelier_sfc::SfcScriptGeneratorFacts>,
    has_script_setup: bool,
    has_plain_script_scope: bool,
) -> Vec<CompactString> {
    if has_script_setup || !has_plain_script_scope {
        return Vec::new();
    }
    facts
        .map(|facts| {
            facts
                .named_value_exports()
                .iter()
                .map(|name| CompactString::new(name.as_str()))
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn collect_projected_module_statement_spans(
    script: &str,
    facts: &vize_atelier_sfc::SfcScriptGeneratorFacts,
) -> Option<Vec<(u32, u32)>> {
    (facts.synthetic_source_len() == script.len()).then(|| {
        include_leading_ts_directive_comments(script, facts.module_statement_spans().to_vec())
    })
}

/// Rebase import/re-export spans from the Atlas module snapshot into Canon's
/// synthetic `script + "\\n" + script-setup` view.
///
/// `None` means the supplied module document does not describe these exact
/// bytes; callers then use the owned SFC generator projection.
pub(super) fn collect_cached_module_statement_spans(
    script: &str,
    document: &vize_module::ModuleDocument,
) -> Option<Vec<(u32, u32)>> {
    let mut joined = VizeString::default();
    for (index, module) in document.modules.iter().enumerate() {
        if index > 0 {
            joined.push('\n');
        }
        joined.push_str(module.source.as_ref());
    }
    if joined.as_str() != script {
        return None;
    }

    let mut spans = Vec::new();
    let mut synthetic_base = 0u32;
    for module in &document.modules {
        let rebase = |span: vize_module::ModuleSpan| {
            let start = span.start.checked_sub(module.base_offset)?;
            let end = span.end.checked_sub(module.base_offset)?;
            (end <= module.source.len() as u32).then_some((
                synthetic_base.saturating_add(start),
                synthetic_base.saturating_add(end),
            ))
        };
        spans.extend(
            module
                .imports
                .iter()
                .filter(|import| !import.dynamic)
                .filter_map(|import| rebase(import.span)),
        );
        spans.extend(
            module
                .exports
                .iter()
                .filter(|export| export.source.is_some())
                .filter_map(|export| rebase(export.span)),
        );
        synthetic_base = synthetic_base
            .saturating_add(module.source.len() as u32)
            .saturating_add(1);
    }
    spans.sort_unstable();
    spans.dedup();
    Some(include_leading_ts_directive_comments(script, spans))
}

pub(super) fn push_setup_return_fields(names: &[CompactString], fields: &mut Vec<CompactString>) {
    fields.extend(names.iter().cloned());
}

pub(super) fn emit_setup_invocation_and_exports(ts: &mut VizeString, names: &[CompactString]) {
    if names.is_empty() {
        ts.push_str("__setup();\n\n");
        return;
    }

    ts.push_str("const __vize_plain_script_exports = __setup();\n");
    for name in names {
        append!(
            *ts,
            "export const {name} = __vize_plain_script_exports.{name};\n"
        );
    }
    ts.push('\n');
}

fn include_leading_ts_directive_comments(script: &str, spans: Vec<(u32, u32)>) -> Vec<(u32, u32)> {
    spans
        .into_iter()
        .map(|(start, end)| {
            let start = leading_ts_directive_comment_start(script, start as usize)
                .unwrap_or(start as usize);
            (start as u32, end)
        })
        .collect()
}

fn leading_ts_directive_comment_start(script: &str, statement_start: usize) -> Option<usize> {
    let mut cursor = line_start_at(script, statement_start);
    let mut comment_group_start = None;
    let mut has_ts_directive = false;
    while cursor > 0 {
        let previous_line_end = cursor.saturating_sub(1);
        let previous_line_start = script[..previous_line_end]
            .rfind('\n')
            .map_or(0, |index| index + 1);
        let line = &script[previous_line_start..previous_line_end];
        let line = line.strip_suffix('\r').unwrap_or(line);
        let trimmed = line.trim_start();
        if !trimmed.starts_with("//") {
            break;
        }
        comment_group_start = Some(previous_line_start);
        if contains_ts_suppression_directive(trimmed) {
            has_ts_directive = true;
        }
        cursor = previous_line_start;
    }
    has_ts_directive.then_some(comment_group_start).flatten()
}

fn line_start_at(script: &str, offset: usize) -> usize {
    script[..offset.min(script.len())]
        .rfind('\n')
        .map_or(0, |index| index + 1)
}

fn contains_ts_suppression_directive(comment: &str) -> bool {
    comment.contains("@ts-ignore") || comment.contains("@ts-expect-error")
}

#[cfg(test)]
mod tests {
    use super::{CompactString, collect_projected_module_statement_spans};

    fn projected_spans(script: &str) -> Vec<(u32, u32)> {
        let facts = vize_atelier_sfc::SfcScriptGeneratorFacts::from_source(script);
        collect_projected_module_statement_spans(script, &facts).unwrap()
    }

    #[test]
    fn collect_import_span_includes_adjacent_ts_ignore_comment_group() {
        let script = "const before = 1;\n// FIXME: types\n// @ts-ignore\nimport Chart from \"chart.js/auto/auto\";\nconst after = 2;\n";
        let spans = projected_spans(script);

        assert_eq!(spans.len(), 1);
        assert_eq!(
            &script[spans[0].0 as usize..spans[0].1 as usize],
            "// FIXME: types\n// @ts-ignore\nimport Chart from \"chart.js/auto/auto\";"
        );
    }

    #[test]
    fn collect_import_span_leaves_regular_comments_in_script_body() {
        let script = "// import note\nimport Chart from \"chart.js/auto/auto\";\n";
        let spans = projected_spans(script);

        assert_eq!(spans.len(), 1);
        assert_eq!(
            &script[spans[0].0 as usize..spans[0].1 as usize],
            "import Chart from \"chart.js/auto/auto\";"
        );
    }

    #[test]
    fn collect_named_value_exports_includes_ts_enums() {
        let facts = vize_atelier_sfc::SfcScriptGeneratorFacts::from_source(
            "export enum DiffDisplayMode { Hidden = 'hidden' }\nexport type Props = {}\n",
        );
        let names = facts
            .named_value_exports()
            .iter()
            .map(|name| CompactString::new(name.as_str()))
            .collect::<Vec<_>>();

        assert_eq!(names, vec![CompactString::new("DiffDisplayMode")]);
    }
}
