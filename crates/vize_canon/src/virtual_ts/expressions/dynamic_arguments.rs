//! Dynamic directive names use JavaScript's computed property-key contract.

use crate::virtual_ts::{VizeMapping, VizeSubSpan};
use vize_carton::{String, append};
use vize_croquis::TemplateExpression;

pub(super) fn generate(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    expression: &TemplateExpression,
    generated: &str,
    template_offset: u32,
    indent: &str,
    template_source: Option<&str>,
) {
    let start = (template_offset + expression.start) as usize;
    let end = (template_offset + expression.end) as usize;
    // The computed key's own error (TS2464, a non-key argument type) belongs
    // to the whole directive, as `vue-tsc` anchors it: `:[foo]` reports at
    // the `:`, `v-bind:[foo]` at the `v`. The `[` is the last byte before the
    // expression; the directive name and its shorthand prefix sit before it.
    let key_anchor = template_source
        .and_then(|source| directive_start(source, expression.start.saturating_sub(1) as usize))
        .map_or(start.saturating_sub(1), |offset| {
            template_offset as usize + offset
        });
    append!(*ts, "{indent}void ({{ ");
    let key_start = ts.len();
    ts.push('[');
    let expression_start = ts.len();
    ts.push_str(generated);
    let expression_end = ts.len();
    ts.push(']');
    let key_end = ts.len();
    ts.push_str(": undefined }); // DynamicDirectiveArgument\n");
    mappings.push(VizeMapping {
        gen_range: key_start..key_end,
        src_range: key_anchor..end + 1,
        sub_spans: vec![
            // TS2464 belongs to the authored directive, while errors inside
            // the expression retain their exact source positions.
            VizeSubSpan {
                gen_range: key_start..expression_start,
                src_range: key_anchor..key_anchor + 1,
            },
            VizeSubSpan {
                gen_range: expression_start..expression_end,
                src_range: start..end,
            },
            VizeSubSpan {
                gen_range: expression_end..key_end,
                src_range: end..end + 1,
            },
        ],
    });
}

/// Template-relative start of the directive whose dynamic argument opens at
/// `bracket` (the `[`): the name and its `v-bind:`/`v-on:`/`v-slot:` prefix,
/// or the `:` / `@` / `#` shorthand.
fn directive_start(source: &str, bracket: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    if bytes.get(bracket) != Some(&b'[') {
        return None;
    }
    let mut start = bracket;
    while start
        .checked_sub(1)
        .and_then(|prev| bytes.get(prev))
        .is_some_and(|byte| {
            matches!(
                byte,
                b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b':'
            )
        })
    {
        start -= 1;
    }
    if start
        .checked_sub(1)
        .and_then(|prev| bytes.get(prev))
        .is_some_and(|byte| matches!(byte, b'@' | b'#'))
    {
        start -= 1;
    }
    (start < bracket).then_some(start)
}

#[cfg(test)]
mod tests {
    use super::directive_start;

    #[test]
    fn dynamic_argument_directives_start_at_their_prefix() {
        for (source, bracket, expected) in [
            ("<div :[foo]=\"1\">", 6, Some(5)),
            ("<div v-bind:[foo]=\"1\">", 12, Some(5)),
            ("<div @[evt]=\"h\">", 6, Some(5)),
            ("<div v-on:[evt]=\"h\">", 10, Some(5)),
            ("<div #[slot]>", 6, Some(5)),
            ("<div v-slot:[slot]>", 12, Some(5)),
            ("<div [foo]>", 5, None),
            ("<div foo=\"1\">", 5, None),
        ] {
            assert_eq!(directive_start(source, bracket), expected, "{source}");
        }
    }
}
