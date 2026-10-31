//! Emit an authored module statement with its default-export rewrite and mapping.

use vize_carton::String;

use super::options_api::OptionsApiScriptFacts;
use super::spans::rewrite_export_default_for_module_scope;
use crate::virtual_ts::VizeMapping;

/// Return whether the statement rewrite declared the default-export alias.
pub(super) fn emit(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    text: &str,
    span: (u32, u32),
    source_offset: &impl Fn(usize) -> usize,
    options_script: &OptionsApiScriptFacts,
) -> bool {
    let (start, end) = span;
    let mut declared_default_alias = false;
    let gen_start = ts.len();
    if text.contains("export default") {
        // Re-base the script-absolute default-export offsets onto this module
        // span; only a target fully owned by the statement can be rewritten.
        let span_start = start as usize;
        let span_end = end as usize;
        let rebase = |(export_start, inner_start, inner_end): (usize, usize, usize)| {
            (export_start >= span_start && inner_end <= span_end).then_some((
                export_start - span_start,
                inner_start - span_start,
                inner_end - span_start,
            ))
        };
        let span_relative_object = options_script.default_export.object.and_then(rebase);
        let span_relative_expr = options_script.default_export.expr.and_then(rebase);
        let rewritten =
            rewrite_export_default_for_module_scope(text, span_relative_object, span_relative_expr);
        // Every rewrite shape declares the alias; a span with no rewriteable
        // default export comes back untouched.
        declared_default_alias |= rewritten.as_str() != text;
        ts.push_str(&rewritten);
        ts.push_str("void __default__;\n");
    } else {
        ts.push_str(text);
    }
    let gen_end = ts.len();
    ts.push('\n');
    mappings.push(VizeMapping {
        gen_range: gen_start..gen_end,
        src_range: source_offset(start as usize)..source_offset(end as usize),
        sub_spans: Vec::new(),
    });
    declared_default_alias
}

#[cfg(test)]
mod tests {
    use super::{String, VizeMapping, emit};
    use crate::virtual_ts::generator::options_api::analyze_options_api_script;

    fn assert_projection(text: &str, expected: &str, expected_alias: bool) {
        let prefix = "// 日本 🧭\n";
        let script = format!("{prefix}{text}");
        let facts = analyze_options_api_script(&script, true, false);
        let mut ts = String::from("header\n");
        let mut mappings = Vec::new();
        let span = (prefix.len() as u32, script.len() as u32);
        assert_eq!(
            emit(
                &mut ts,
                &mut mappings,
                text,
                span,
                &|offset| 400 + offset,
                &facts
            ),
            expected_alias
        );
        assert_eq!(ts.as_str(), format!("header\n{expected}\n"));
        assert_eq!(
            mappings,
            vec![VizeMapping {
                gen_range: "header\n".len().."header\n".len() + expected.len(),
                src_range: 400 + prefix.len()..400 + script.len(),
                sub_spans: Vec::new(),
            }]
        );
    }

    #[test]
    fn ordinary_unicode_crlf_statement_keeps_whole_bytes_and_authored_range() {
        assert_projection("const 名前 = 1;\r\n", "const 名前 = 1;\r\n", false);
    }

    #[test]
    fn options_object_rebases_into_its_statement_without_moving_the_mapping() {
        assert_projection(
            "export default { name: 'A' };\r\n",
            "const __default__ = __vizeDefineComponent({ name: 'A' });\r\nvoid __default__;\n",
            true,
        );
    }

    #[test]
    fn default_call_keeps_its_unicode_expression_and_alias_anchor() {
        assert_projection(
            "export default defineComponent({ name: '雪' });\n",
            "const __default__ = defineComponent({ name: '雪' });\nvoid __default__;\n",
            true,
        );
    }

    #[test]
    fn unrewritable_default_text_preserves_the_existing_anchor_and_false_alias() {
        assert_projection(
            "const text = 'export default';\n",
            "const text = 'export default';\nvoid __default__;\n",
            false,
        );
    }
}
