mod aggregate;

use vize_s0::{String, cstr};

use crate::{SourceLocation, TransformContext};

use self::aggregate::{lower_array_pattern, lower_object_pattern};
use super::report_pattern_error;
use super::syntax::{
    is_valid_ident, split_top_level_keyword, split_top_level_or, strip_outer_pair,
};

#[derive(Debug, Default)]
pub(super) struct CompiledPattern {
    pub(super) condition: String,
    pub(super) binding: Option<String>,
    pub(super) as_bindings: std::vec::Vec<AsBinding>,
    pub(super) guard: Option<String>,
}

impl CompiledPattern {
    fn has_bindings(&self) -> bool {
        self.binding.is_some() || !self.as_bindings.is_empty()
    }
}

#[derive(Debug)]
pub(super) struct AsBinding {
    pub(super) pattern: String,
    pub(super) source: String,
}

#[derive(Default)]
pub(super) struct PatternLoweringState {
    dummy_index: usize,
}

pub(super) fn pattern_without_guard(expr: &str) -> &str {
    guard_split(expr).map_or(expr, |(pattern, _)| pattern)
}

pub(super) fn guard_suffix(expr: &str) -> Option<&str> {
    guard_split(expr).map(|(_, guard)| guard)
}

fn guard_split(expr: &str) -> Option<(&str, &str)> {
    let trimmed = expr.trim();
    let (pattern, guard_with_parens) = split_top_level_keyword(trimmed, "if")?;
    let guard_with_parens = guard_with_parens.trim();
    let guard = guard_with_parens
        .strip_prefix('(')
        .and_then(|guard| guard.strip_suffix(')'))?;
    Some((pattern.trim(), guard.trim()))
}

pub(super) fn build_case_pattern(
    ctx: &mut TransformContext<'_>,
    subject_expr: &str,
    case_expr: &str,
    loc: &SourceLocation,
) -> CompiledPattern {
    let guard = guard_suffix(case_expr).map(String::from);
    let case_expr = pattern_without_guard(case_expr).trim();
    let mut state = PatternLoweringState::default();
    let mut pattern = lower_pattern(ctx, case_expr, subject_expr, loc, &mut state);
    pattern.guard = guard;
    pattern
}

pub(super) fn pattern_condition_with_guard(
    pattern: &CompiledPattern,
    subject_expr: &str,
) -> String {
    if pattern.guard.is_some() {
        let guard_condition = build_guard_condition(pattern, subject_expr);
        if pattern.condition == "true" {
            guard_condition
        } else {
            let base = &pattern.condition;
            cstr!("({base}) && ({guard_condition})")
        }
    } else {
        pattern.condition.clone()
    }
}

pub(super) fn build_guard_condition(pattern: &CompiledPattern, subject_expr: &str) -> String {
    let Some(guard) = &pattern.guard else {
        return String::from("true");
    };
    if !pattern.has_bindings() {
        return guard.clone();
    }

    let mut out = String::from("(() => { ");
    if let Some(binding) = &pattern.binding {
        out.push_str("const ");
        out.push_str(binding);
        out.push_str(" = ");
        out.push_str(subject_expr);
        out.push_str("; ");
    }
    for as_binding in &pattern.as_bindings {
        out.push_str("const ");
        out.push_str(&as_binding.pattern);
        out.push_str(" = ");
        out.push_str(&as_binding.source);
        out.push_str("; ");
    }
    out.push_str("return (");
    out.push_str(guard);
    out.push_str("); })()");
    out
}

pub(super) fn build_binding_for_expression(
    pattern: &CompiledPattern,
    subject_expr: &str,
) -> Option<String> {
    if !pattern.has_bindings() {
        return None;
    }

    let mut alias = String::from("{ ");
    let mut source = String::from("[{ ");
    let mut first = true;
    if let Some(binding) = &pattern.binding {
        alias.push_str("__vize_value: ");
        alias.push_str(binding);
        source.push_str("__vize_value: ");
        source.push_str(subject_expr);
        first = false;
    }
    for (index, as_binding) in pattern.as_bindings.iter().enumerate() {
        if !first {
            alias.push_str(", ");
            source.push_str(", ");
        }
        alias.push_str(&cstr!("__vize_as{index}: "));
        alias.push_str(&as_binding.pattern);
        source.push_str(&cstr!("__vize_as{index}: "));
        source.push_str(&as_binding.source);
        first = false;
    }
    alias.push_str(" }");
    source.push_str(" }]");
    alias.push_str(" in ");
    alias.push_str(&source);
    Some(alias)
}

fn lower_pattern(
    ctx: &mut TransformContext<'_>,
    pattern: &str,
    subject_expr: &str,
    loc: &SourceLocation,
    state: &mut PatternLoweringState,
) -> CompiledPattern {
    let pattern = pattern.trim();
    if pattern.is_empty() {
        return CompiledPattern {
            condition: String::from("false"),
            ..Default::default()
        };
    }

    if let Some((inner, alias)) = split_top_level_keyword(pattern, "as") {
        let mut compiled = lower_pattern(ctx, inner, subject_expr, loc, state);
        if let Some(binding) = binding_pattern(ctx, alias, loc) {
            compiled.as_bindings.push(AsBinding {
                pattern: binding,
                source: String::from(subject_expr),
            });
        }
        return compiled;
    }

    if let Some(parts) = split_top_level_or(pattern) {
        let mut conditions = std::vec::Vec::new();
        let mut any_bindings = false;
        for part in parts {
            let compiled = lower_pattern(ctx, part, subject_expr, loc, state);
            any_bindings |= compiled.has_bindings();
            conditions.push(compiled.condition);
        }
        if any_bindings {
            report_pattern_error(
                ctx,
                loc,
                "`v-when` alternatives separated by `|` cannot introduce bindings yet.",
            );
        }
        return CompiledPattern {
            condition: join_conditions(&conditions, " || "),
            ..Default::default()
        };
    }

    if let Some(inner) = strip_outer_pair(pattern, '(', ')') {
        return lower_pattern(ctx, inner, subject_expr, loc, state);
    }

    if pattern == "NaN" {
        return CompiledPattern {
            condition: cstr!("Number.isNaN({subject_expr})"),
            ..Default::default()
        };
    }
    if pattern == "_" {
        return CompiledPattern {
            condition: String::from("true"),
            ..Default::default()
        };
    }
    if let Some(binding) = binding_pattern(ctx, pattern, loc) {
        return CompiledPattern {
            condition: String::from("true"),
            binding: Some(binding),
            ..Default::default()
        };
    }

    if starts_with_binding_keyword(pattern, "let") || starts_with_binding_keyword(pattern, "var") {
        report_pattern_error(
            ctx,
            loc,
            "`v-when` pattern bindings must use `const`; `let` and `var` are not supported.",
        );
        return CompiledPattern {
            condition: String::from("false"),
            ..Default::default()
        };
    }

    if let Some(inner) = strip_outer_pair(pattern, '{', '}') {
        return lower_object_pattern(ctx, inner, subject_expr, loc, state);
    }
    if let Some(inner) = strip_outer_pair(pattern, '[', ']') {
        return lower_array_pattern(ctx, inner, subject_expr, loc, state);
    }

    CompiledPattern {
        condition: cstr!("({subject_expr}) === ({pattern})"),
        ..Default::default()
    }
}

fn binding_pattern(
    ctx: &mut TransformContext<'_>,
    pattern: &str,
    loc: &SourceLocation,
) -> Option<String> {
    let pattern = pattern.trim();
    if let Some(rest) = pattern.strip_prefix("const ") {
        let rest = rest.trim();
        if is_valid_binding_pattern(rest) {
            Some(String::from(rest))
        } else {
            report_pattern_error(
                ctx,
                loc,
                "`v-when` const binding is missing a binding pattern.",
            );
            None
        }
    } else {
        None
    }
}

fn starts_with_binding_keyword(input: &str, keyword: &str) -> bool {
    let Some(rest) = input.trim().strip_prefix(keyword) else {
        return false;
    };
    rest.chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_whitespace())
}

fn is_valid_binding_pattern(pattern: &str) -> bool {
    let pattern = pattern.trim();
    is_valid_ident(pattern)
        || (pattern.starts_with('{') && pattern.ends_with('}'))
        || (pattern.starts_with('[') && pattern.ends_with(']'))
}

pub(super) fn is_wildcard_pattern(pattern: &str) -> bool {
    let mut pattern = pattern.trim();
    while let Some(inner) = strip_outer_pair(pattern, '(', ')') {
        pattern = inner.trim();
    }
    pattern == "_"
}

fn property_access(subject_expr: &str, key: &str) -> String {
    let key = key.trim();
    if is_valid_ident(key) {
        cstr!("({subject_expr}).{key}")
    } else if let Some(inner) = strip_outer_pair(key, '[', ']') {
        cstr!("({subject_expr})[{inner}]")
    } else {
        cstr!("({subject_expr})[{key}]")
    }
}

fn next_dummy(state: &mut PatternLoweringState) -> String {
    let index = state.dummy_index;
    state.dummy_index += 1;
    cstr!("__vize_match_unused_{index}")
}

fn join_conditions(parts: &[String], op: &str) -> String {
    if parts.is_empty() {
        return String::from("true");
    }
    if parts.len() == 1 {
        return parts[0].clone();
    }

    let mut out = String::from("(");
    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            out.push_str(op);
        }
        out.push('(');
        out.push_str(part);
        out.push(')');
    }
    out.push(')');
    out
}
