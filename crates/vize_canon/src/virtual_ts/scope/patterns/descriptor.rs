use super::source::{PatternEmitter, map_range};
use crate::virtual_ts::types::VizeMapping;
use vize_armature::patterns::{MatchPattern, PatternBinding, PatternKind};
use vize_carton::{String, append, cstr};

pub(super) fn emit_descriptor(
    emitter: &mut PatternEmitter<'_, '_>,
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    pattern: &MatchPattern,
    offset: u32,
) -> String {
    match &pattern.kind {
        PatternKind::Wildcard | PatternKind::Binding(_) => "['any']".into(),
        PatternKind::As { pattern, .. } => emit_descriptor(emitter, ts, mappings, pattern, offset),
        PatternKind::Literal(value) => cstr!(
            "['literal', {}]",
            value.text.strip_prefix('+').unwrap_or(&value.text)
        ),
        PatternKind::Value(value) => {
            let name = emitter.name();
            append!(*ts, "{}const {name} = (", emitter.indent);
            emitter.expression(
                ts,
                mappings,
                &value.text,
                offset + value.span.start,
                offset + value.span.end,
            );
            ts.push_str(");\n");
            cstr!("['value', typeof {name}]")
        }
        PatternKind::Or(elements) | PatternKind::Array { elements, .. } => {
            let mut output = String::from(if matches!(pattern.kind, PatternKind::Or(_)) {
                "['or', ["
            } else {
                "['array', ["
            });
            for element in elements {
                output.push_str(&emit_descriptor(emitter, ts, mappings, element, offset));
                output.push(',');
            }
            match &pattern.kind {
                PatternKind::Array { rest, .. } => append!(output, "], {}]", rest.is_some()),
                _ => output.push_str("]]"),
            }
            output
        }
        PatternKind::Object { properties, .. } => {
            let mut output = String::from("['object', [");
            for property in properties {
                let descriptor = emit_descriptor(emitter, ts, mappings, &property.pattern, offset);
                append!(output, "[{}, {descriptor}],", property.key.text);
            }
            output.push_str("]]");
            output
        }
    }
}

pub(super) fn emit_bindings(
    emitter: &mut PatternEmitter<'_, '_>,
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    pattern: &MatchPattern,
    value: &str,
    offset: u32,
    indent: &str,
) {
    match &pattern.kind {
        PatternKind::Binding(binding) => {
            declare(emitter, ts, mappings, binding, value, offset, indent)
        }
        PatternKind::As { pattern, binding } => {
            emit_bindings(emitter, ts, mappings, pattern, value, offset, indent);
            declare(emitter, ts, mappings, binding, value, offset, indent);
        }
        PatternKind::Object { properties, rest } => {
            for property in properties {
                emit_bindings(
                    emitter,
                    ts,
                    mappings,
                    &property.pattern,
                    &cstr!("{value}[{}]", property.key.text),
                    offset,
                    indent,
                );
            }
            if let Some(binding) = rest.as_ref().and_then(|rest| rest.binding.as_ref()) {
                // An indexed Omit preserves types without unused destructuring
                // temporaries or runtime property reads in the virtual module.
                let keys = if properties.is_empty() {
                    String::from("never")
                } else {
                    properties
                        .iter()
                        .map(|property| property.key.text.as_str())
                        .collect::<Vec<_>>()
                        .join(" | ")
                        .into()
                };
                declare(
                    emitter,
                    ts,
                    mappings,
                    binding,
                    &cstr!("{value} as Omit<typeof {value}, {keys}>"),
                    offset,
                    indent,
                );
            }
        }
        PatternKind::Array { elements, rest } => {
            for (index, element) in elements.iter().enumerate() {
                emit_bindings(
                    emitter,
                    ts,
                    mappings,
                    element,
                    &cstr!("{value}[{index}]"),
                    offset,
                    indent,
                );
            }
            if let Some(binding) = rest.as_ref().and_then(|rest| rest.binding.as_ref()) {
                append!(*ts, "{indent}const [{}...", ",".repeat(elements.len()));
                binding_name(emitter, ts, mappings, binding, offset);
                append!(*ts, "] = {value};\n{indent}void {};\n", binding.name);
            }
        }
        _ => {}
    }
}

fn declare(
    emitter: &PatternEmitter<'_, '_>,
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    binding: &PatternBinding,
    value: &str,
    offset: u32,
    indent: &str,
) {
    append!(*ts, "{indent}const ");
    binding_name(emitter, ts, mappings, binding, offset);
    append!(*ts, " = {value};\n{indent}void {};\n", binding.name);
}

fn binding_name(
    emitter: &PatternEmitter<'_, '_>,
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    binding: &PatternBinding,
    offset: u32,
) {
    let start = ts.len();
    ts.push_str(&binding.name);
    if emitter.ctx.verification {
        let source = emitter.ctx.template_offset + offset;
        map_range(
            mappings,
            start..ts.len(),
            (source + binding.span.start) as usize..(source + binding.span.end) as usize,
        );
    }
}
