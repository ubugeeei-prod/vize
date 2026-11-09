//! Emit an authored module statement with its default-export rewrite and mapping.

use super::options_api::OptionsApiScriptFacts;
use super::spans::rewrite_export_default_for_module_scope;
use crate::virtual_ts::VizeMapping;

/// Return whether the statement rewrite declared the default-export alias.
pub(super) fn emit(
    ts: &mut crate::virtual_ts::TextBuf,
    mappings: &mut Vec<VizeMapping>,
    text: &str,
    span: (u32, u32),
    source_offset: &impl Fn(usize) -> usize,
    options_script: &OptionsApiScriptFacts,
) -> bool {
    let (start, end) = span;
    let gen_start = ts.len();
    let map = |mappings: &mut Vec<VizeMapping>,
               generated: std::ops::Range<usize>,
               authored: std::ops::Range<usize>| {
        mappings.push(VizeMapping {
            gen_range: generated,
            src_range: source_offset(start as usize + authored.start)
                ..source_offset(start as usize + authored.end),
            sub_spans: Vec::new(),
        });
    };
    if !text.contains("export default") {
        ts.push_str(text);
        map(mappings, gen_start..ts.len(), 0..text.len());
        ts.push('\n');
        return false;
    }
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
    let declared_default_alias = rewritten.as_str() != text;
    ts.push_str(&rewritten);
    // Identical bytes around the rewrite, and the exported value it copies,
    // keep 1:1 mappings: diagnostics there must not collapse onto one byte.
    let (prefix, suffix) = unchanged_edges(text, &rewritten);
    let (text_end, rewritten_end) = (text.len() - suffix, rewritten.len() - suffix);
    let core = span_relative_object
        .or(span_relative_expr)
        .map(|(_, inner_start, inner_end)| inner_start..inner_end)
        .filter(|core| prefix <= core.start && core.end <= text_end)
        .and_then(|core| {
            let generated = rewritten
                .get(prefix..rewritten_end)?
                .find(text.get(core.clone())?)?;
            Some((prefix + generated, core))
        });
    let mut segments = vec![(0..prefix, 0..prefix)];
    match core {
        Some((generated, core)) => {
            let generated_end = generated + core.len();
            segments.push((prefix..generated, prefix..core.start));
            segments.push((generated..generated_end, core.clone()));
            segments.push((generated_end..rewritten_end, core.end..text_end));
        }
        None => segments.push((prefix..rewritten_end, prefix..text_end)),
    }
    segments.push((rewritten_end..rewritten.len(), text_end..text.len()));
    for (generated, authored) in segments {
        if !generated.is_empty() {
            map(
                mappings,
                gen_start + generated.start..gen_start + generated.end,
                authored,
            );
        }
    }
    // Without a declared alias, an anchor would invent TS2304.
    if declared_default_alias {
        ts.push_str("void __default__;\n");
    }
    ts.push('\n');
    declared_default_alias
}

/// Byte lengths of the shared prefix and the disjoint shared suffix, on char
/// boundaries of both texts.
fn unchanged_edges(text: &str, rewritten: &str) -> (usize, usize) {
    let prefix = text
        .char_indices()
        .zip(rewritten.chars())
        .find(|((_, left), right)| left != right)
        .map_or(text.len().min(rewritten.len()), |((index, _), _)| index);
    let limit = text.len().min(rewritten.len()) - prefix;
    let suffix = text
        .char_indices()
        .rev()
        .zip(rewritten.chars().rev())
        .take_while(|((index, left), right)| left == right && text.len() - index <= limit)
        .last()
        .map_or(0, |((index, _), _)| text.len() - index);
    (prefix, suffix)
}

#[cfg(test)]
mod tests {
    use super::{VizeMapping, emit};
    use crate::virtual_ts::generator::options_api::analyze_options_api_script;

    /// `segments` are `(generated, authored)` ranges relative to the statement.
    fn assert_projection(
        text: &str,
        expected: &str,
        expected_alias: bool,
        segments: &[(std::ops::Range<usize>, std::ops::Range<usize>)],
    ) {
        let prefix = "// 日本 🧭\n";
        let script = format!("{prefix}{text}");
        let facts = analyze_options_api_script(&script, true, false);
        let mut ts = crate::virtual_ts::TextBuf::from_text("header\n");
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
        let generated = "header\n".len();
        let authored = 400 + prefix.len();
        assert_eq!(
            mappings,
            segments
                .iter()
                .map(|(gen_range, src_range)| VizeMapping {
                    gen_range: generated + gen_range.start..generated + gen_range.end,
                    src_range: authored + src_range.start..authored + src_range.end,
                    sub_spans: Vec::new(),
                })
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn ordinary_unicode_crlf_statement_keeps_whole_bytes_and_authored_range() {
        let text = "const 名前 = 1;\r\n";
        assert_projection(text, text, false, &[(0..text.len(), 0..text.len())]);
    }

    #[test]
    fn options_object_rebases_into_its_statement_without_moving_the_mapping() {
        assert_projection(
            "export default { name: 'A' };\r\n",
            "const __default__ = __vizeDefineComponent({ name: 'A' });\r\nvoid __default__;\n",
            true,
            // Keyword head, verbatim object, inserted `)`, shared `;\r\n`.
            &[
                (0..42, 0..15),
                (42..55, 15..28),
                (55..56, 28..28),
                (56..59, 28..31),
            ],
        );
    }

    #[test]
    fn default_call_keeps_its_unicode_expression_and_alias_anchor() {
        assert_projection(
            "export default defineComponent({ name: '雪' });\n",
            "const __default__ = defineComponent({ name: '雪' });\nvoid __default__;\n",
            true,
            &[(0..19, 0..14), (19..54, 14..49)],
        );
    }

    #[test]
    fn unrewritable_default_text_declares_no_alias_and_reads_none() {
        assert_projection(
            "const text = 'export default';\n",
            "const text = 'export default';\n",
            false,
            &[(0..31, 0..31)],
        );
    }
}
