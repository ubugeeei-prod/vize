//! Replace original Vue CSS binding operations, retaining authored mapping edits.
use core::ops::Range;
use vize_l0::{Span, String};
use vize_l1_to_l2::lower::OriginalStyleBindings;

/// One CSS replacement, in file-absolute input and block-relative output bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyleBindingEdit {
    pub source: Span,
    pub generated: Range<usize>,
}

/// Emit an inline style using the host's CSS variable naming policy. Each edit
/// covers the entire generated reference. Verbatim gaps keep their original bytes.
pub fn emit_style_bindings(
    style: &OriginalStyleBindings<'_>,
    mut variable_name: impl FnMut(&str) -> String,
    mut record: impl FnMut(StyleBindingEdit),
) -> String {
    let block = style.block();
    let source = block.source();
    if !style.has_calls() {
        return String::from(source);
    }
    let mut output = String::with_capacity(source.len());
    let mut previous = 0;
    let mut remaining = source;
    for call in style.calls() {
        let start = (call.span.start - block.start()) as usize;
        let (gap, rest) = remaining.split_at(start - previous);
        output.push_str(gap);
        let generated_start = output.len();
        output.push_str("var(--");
        output.push_str(&variable_name(call.value.source()));
        output.push(')');
        record(StyleBindingEdit {
            source: call.span,
            generated: generated_start..output.len(),
        });
        previous = (call.span.end - block.start()) as usize;
        remaining = rest.split_at(previous - start).1;
    }
    output.push_str(remaining);
    output
}
