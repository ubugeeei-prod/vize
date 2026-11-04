//! The actual typed instance check owns its diagnostic occurrence.

use crate::virtual_ts::VizeMapping;
use crate::virtual_ts::diagnostic_ownership::DiagnosticOwnershipCollector;
use vize_carton::{String, append, cstr};

pub(super) fn emit_strict_template_context_ref(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    (receiver, name): (&str, &str),
    src_start: usize,
    src_end: usize,
    already_declared: bool,
    ownership: Option<&DiagnosticOwnershipCollector>,
) {
    let expr_code = if already_declared {
        cstr!("  void ({receiver}.{name});\n")
    } else {
        cstr!("  var {name}: any = undefined; void ({name});\n  void ({receiver}.{name});\n")
    };
    let gen_start = ts.len();
    let name_offset = expr_code.rfind(name).unwrap_or(0);
    let gen_name_start = gen_start + name_offset;
    let gen_name_end = gen_name_start + name.len();
    if let Some(collector) = ownership {
        collector
            .borrow_mut()
            .record_diagnostic_primary(gen_name_start..gen_name_end, src_start..src_end);
    }
    ts.push_str(&expr_code);
    mappings.push(VizeMapping {
        gen_range: gen_name_start..gen_name_end,
        src_range: src_start..src_end,
        sub_spans: Vec::new(),
    });
    append!(
        *ts,
        "  // @vize-map: {gen_name_start}:{gen_name_end} -> {src_start}:{src_end}\n",
    );
}
