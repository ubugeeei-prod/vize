//! Unknown listener context and callability are checked in separate statements.

use crate::virtual_ts::types::VizeMapping;
use std::ops::Range;
use vize_carton::{append, cstr};

pub(super) struct Listener<'a> {
    pub(super) name: &'a str,
    pub(super) content: &'a str,
    pub(super) annotation: &'a str,
    pub(super) contextless: &'a str,
    pub(super) indent: &'a str,
    pub(super) source: Range<usize>,
    pub(super) event_name: Option<Range<usize>>,
}

pub(super) fn emit(
    ts: &mut crate::virtual_ts::TextBuf,
    diagnostics: &mut Vec<VizeMapping>,
    listener: Listener<'_>,
) -> Range<usize> {
    let Listener {
        name,
        content,
        annotation,
        contextless,
        indent,
        source,
        event_name,
    } = listener;
    // An unknown property context supplies no call signature, while satisfies
    // keeps the initializer's inferred type for the separate callability check.
    append!(*ts, "{indent}const {name} = ({{ ");
    let context_key_start = ts.len();
    ts.push_str("__vize_listener");
    let context_key = context_key_start..ts.len();
    ts.push_str(": (");
    let expression_start = ts.len();
    ts.push_str(content);
    let expression = expression_start..ts.len();
    append!(
        *ts,
        ") }} satisfies {{ __vize_listener: {contextless} extends true ? unknown : ({annotation} | null | undefined) }}).__vize_listener;\n"
    );
    // This constraint cannot contextualize the authored callback. It accepts
    // callable references/arrays and rejects primitives, objects and classes.
    let callable = cstr!("{name}_callable");
    append!(*ts, "{indent}const ");
    let check_start = ts.len();
    ts.push_str(callable.as_str());
    let check = check_start..ts.len();
    append!(
        *ts,
        ": {contextless} extends true ? (((...args: never[]) => unknown) | readonly ((...args: never[]) => unknown)[] | null | undefined) : unknown = {name}; void {callable};\n"
    );
    diagnostics.push(VizeMapping::new(
        context_key,
        event_name.unwrap_or_else(|| source.clone()),
    ));
    diagnostics.push(VizeMapping::new(check, source));
    // The initializer checks authored bodies and declared listener signatures;
    // the assignment checks callability. Replaying the value would add a
    // second synthetic error for an already rejected constructor.
    expression
}

#[cfg(test)]
mod tests;
