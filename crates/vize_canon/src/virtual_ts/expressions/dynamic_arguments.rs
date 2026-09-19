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
) {
    let start = (template_offset + expression.start) as usize;
    let end = (template_offset + expression.end) as usize;
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
        src_range: start.saturating_sub(1)..end + 1,
        sub_spans: vec![
            // TS2464 belongs to the authored `[`, while errors inside the
            // expression retain their exact source positions.
            VizeSubSpan {
                gen_range: key_start..expression_start,
                src_range: start.saturating_sub(1)..start,
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
