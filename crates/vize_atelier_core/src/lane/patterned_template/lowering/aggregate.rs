use vize_s0::{String, cstr};

use crate::{SourceLocation, TransformContext};

use super::super::report_pattern_error;
use super::super::syntax::{
    is_valid_ident, split_top_level_colon, split_top_level_commas, strip_outer_pair,
};
use super::{
    CompiledPattern, PatternLoweringState, binding_pattern, is_valid_binding_pattern,
    join_conditions, lower_pattern, next_dummy, property_access, starts_with_binding_keyword,
};

pub(super) fn lower_object_pattern(
    ctx: &mut TransformContext<'_>,
    inner: &str,
    subject_expr: &str,
    loc: &SourceLocation,
    state: &mut PatternLoweringState,
) -> CompiledPattern {
    let parts = split_top_level_commas(inner);
    let has_rest_binding = parts.iter().any(|part| {
        part.trim()
            .strip_prefix("...")
            .is_some_and(|rest| !rest.trim().is_empty())
    });
    let mut conditions = std::vec::Vec::new();
    let mut binding_props = std::vec::Vec::new();
    let mut as_bindings = std::vec::Vec::new();
    let mut rest_binding = None;

    conditions.push(cstr!("({subject_expr}) != null"));
    for part in parts {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        if let Some(rest) = part.strip_prefix("...") {
            if !rest.trim().is_empty() {
                rest_binding = parse_rest_binding(ctx, rest, loc);
            }
            continue;
        }

        let shorthand_key = object_shorthand_binding_key(part);
        let (key, value) = split_top_level_colon(part)
            .map(|(key, value)| (key.trim(), value.trim()))
            .or_else(|| shorthand_key.map(|key| (key, part)))
            .unwrap_or((part, part));
        let key = key.trim();
        let value = value.trim();
        let presence_key = if is_valid_ident(key) {
            cstr!("\"{key}\"")
        } else {
            String::from(strip_outer_pair(key, '[', ']').unwrap_or(key))
        };
        conditions.push(cstr!("({presence_key}) in Object({subject_expr})"));
        let property_expr = property_access(subject_expr, key);
        let compiled = lower_pattern(ctx, value, &property_expr, loc, state);
        if compiled.condition != "true" {
            conditions.push(compiled.condition);
        }
        if let Some(binding) = compiled.binding {
            binding_props.push(cstr!("{key}: {binding}"));
        } else if has_rest_binding {
            binding_props.push(cstr!("{key}: {}", next_dummy(state)));
        }
        as_bindings.extend(compiled.as_bindings);
    }

    if let Some(rest_binding) = rest_binding {
        binding_props.push(cstr!("...{rest_binding}"));
    }

    let binding = (!binding_props.is_empty()).then(|| {
        let mut out = String::from("{ ");
        for (index, prop) in binding_props.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            out.push_str(prop);
        }
        out.push_str(" }");
        out
    });

    CompiledPattern {
        condition: join_conditions(&conditions, " && "),
        binding,
        as_bindings,
        guard: None,
    }
}

pub(super) fn lower_array_pattern(
    ctx: &mut TransformContext<'_>,
    inner: &str,
    subject_expr: &str,
    loc: &SourceLocation,
    state: &mut PatternLoweringState,
) -> CompiledPattern {
    let mut parts = split_top_level_commas(inner);
    // A trailing comma is punctuation, not an extra array element. This also
    // makes the empty pattern have length zero.
    if parts.last().is_some_and(|part| part.trim().is_empty()) {
        parts.pop();
    }
    let has_rest = parts.iter().any(|part| part.trim().starts_with("..."));
    let has_rest_binding = parts.iter().any(|part| {
        part.trim()
            .strip_prefix("...")
            .is_some_and(|rest| !rest.trim().is_empty())
    });
    let mut conditions = std::vec::Vec::new();
    let mut binding_items = std::vec::Vec::new();
    let mut as_bindings = std::vec::Vec::new();
    let mut rest_binding = None;
    let element_count = parts
        .iter()
        .filter(|part| !part.trim().starts_with("..."))
        .count();
    let mut element_index = 0usize;

    conditions.push(cstr!("Array.isArray({subject_expr})"));
    conditions.push(if has_rest {
        cstr!("({subject_expr}).length >= {element_count}")
    } else {
        cstr!("({subject_expr}).length === {element_count}")
    });
    for part in parts {
        let part = part.trim();
        if let Some(rest) = part.strip_prefix("...") {
            if !rest.trim().is_empty() {
                rest_binding = parse_rest_binding(ctx, rest, loc);
            }
            continue;
        }

        let property_expr = cstr!("({subject_expr})[{element_index}]");
        if part.is_empty() {
            binding_items.push(String::from(""));
            element_index += 1;
            continue;
        }

        let compiled = lower_pattern(ctx, part, &property_expr, loc, state);
        if compiled.condition != "true" {
            conditions.push(compiled.condition);
        }
        if let Some(binding) = compiled.binding {
            binding_items.push(binding);
        } else if has_rest_binding {
            binding_items.push(next_dummy(state));
        } else {
            binding_items.push(String::from(""));
        }
        as_bindings.extend(compiled.as_bindings);
        element_index += 1;
    }

    let has_binding_items =
        binding_items.iter().any(|item| !item.is_empty()) || rest_binding.is_some();
    let binding = has_binding_items.then(|| {
        while binding_items.last().is_some_and(|item| item.is_empty()) && rest_binding.is_none() {
            binding_items.pop();
        }
        let mut out = String::from("[");
        for (index, item) in binding_items.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            out.push_str(item);
        }
        if let Some(rest_binding) = rest_binding {
            if !binding_items.is_empty() {
                out.push_str(", ");
            }
            out.push_str("...");
            out.push_str(&rest_binding);
        }
        out.push(']');
        out
    });

    CompiledPattern {
        condition: join_conditions(&conditions, " && "),
        binding,
        as_bindings,
        guard: None,
    }
}

fn parse_rest_binding(
    ctx: &mut TransformContext<'_>,
    rest: &str,
    loc: &SourceLocation,
) -> Option<String> {
    let rest = rest.trim();
    if let Some(binding) = binding_pattern(ctx, rest, loc) {
        return Some(binding);
    }
    if starts_with_binding_keyword(rest, "let") || starts_with_binding_keyword(rest, "var") {
        report_pattern_error(
            ctx,
            loc,
            "`v-when` rest bindings must use `const`; `let` and `var` are not supported.",
        );
        return None;
    }
    if is_valid_binding_pattern(rest) {
        return Some(String::from(rest));
    }
    report_pattern_error(ctx, loc, "`v-when` rest pattern is missing a binding name.");
    None
}

fn object_shorthand_binding_key(pattern: &str) -> Option<&str> {
    let binding = pattern.trim().strip_prefix("const ")?.trim();
    super::super::syntax::is_valid_ident(binding).then_some(binding)
}
