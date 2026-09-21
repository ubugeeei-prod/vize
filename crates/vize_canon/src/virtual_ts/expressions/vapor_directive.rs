//! A custom directive in a Vapor component is a plain function:
//! `(node, value?: () => T, argument?, modifiers?) => void`. The template calls
//! it directly, so the value is checked through its getter and the argument
//! and modifiers as the positional parameters they are.

use std::ops::Range;

use super::directive_values::DirectiveValueBinding;
use crate::virtual_ts::helpers::push_ts_string_literal;
use crate::virtual_ts::types::VizeMapping;
use vize_carton::{String, append};

pub(super) fn generate(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    (name, generated_expression, source): (&str, &str, Range<usize>),
    binding: &DirectiveValueBinding,
    template_offset: u32,
    indent: &str,
) {
    let variable = binding.variable.as_str();
    if binding.in_setup {
        append!(
            *ts,
            "{indent}const {name} = __vizeVaporDirective({variable});\n"
        );
    } else {
        let registered = if binding.has_default_alias {
            "typeof __default__"
        } else {
            "unknown"
        };
        append!(
            *ts,
            "{indent}const {name} = __vizeVaporDirective(__vizeRegisteredDirective<{registered}, \"{variable}\">());\n",
        );
    }
    let mut push_mapped = |ts: &mut String, text: &str, source: Range<usize>| {
        let gen_start = ts.len();
        ts.push_str(text);
        mappings.push(VizeMapping {
            gen_range: gen_start..ts.len(),
            src_range: source,
            sub_spans: Vec::new(),
        });
    };
    let in_template = |(start, end): (u32, u32)| {
        (template_offset + start) as usize..(template_offset + end) as usize
    };

    append!(*ts, "{indent}{name}(null!, () => (");
    push_mapped(ts, generated_expression, source);
    ts.push_str("), ");
    match &binding.arg {
        Some((arg, range)) => {
            let mut literal = String::default();
            push_ts_string_literal(&mut literal, arg.as_str());
            push_mapped(ts, literal.as_str(), in_template(*range));
        }
        None => ts.push_str("undefined"),
    }
    ts.push_str(", ");
    if binding.modifiers.is_empty() {
        ts.push_str("undefined");
    } else {
        ts.push_str("{ ");
        for (modifier, range) in &binding.modifiers {
            let mut key = String::default();
            push_ts_string_literal(&mut key, modifier.as_str());
            push_mapped(ts, key.as_str(), in_template(*range));
            ts.push_str(": true, ");
        }
        ts.push('}');
    }
    ts.push_str("); // CustomDirective\n");
}
