use crate::fast::{CompactString, FastSet};

use super::{
    split::{
        is_wrapped_by, split_comma_refs, split_generic, split_top_level_once, strip_wrapping_parens,
    },
    type_text::{is_bigint_literal, is_number_literal, is_string_literal},
};

/// Returns whether any `any`-typed source can flow unsafely into the target texts.
pub fn has_unsafe_any_flow<S: AsRef<str>, T: AsRef<str>>(
    source_texts: &[S],
    target_texts: &[T],
) -> bool {
    let sources = parse_type_texts(source_texts);
    if sources.is_empty() {
        return false;
    }
    let targets = parse_type_texts(target_texts);
    if targets.is_empty() {
        return sources.iter().any(contains_any_like);
    }
    sources.iter().any(|source| {
        targets
            .iter()
            .filter(|target| !is_permissive_target(target))
            .any(|target| is_unsafe_flow(source, target))
    })
}

/// Alias for unsafe assignment checks.
pub fn is_unsafe_assignment<S: AsRef<str>, T: AsRef<str>>(
    source_texts: &[S],
    target_texts: &[T],
) -> bool {
    has_unsafe_any_flow(source_texts, target_texts)
}

/// Alias for unsafe return checks.
pub fn is_unsafe_return<S: AsRef<str>, T: AsRef<str>>(
    source_texts: &[S],
    target_texts: &[T],
) -> bool {
    has_unsafe_any_flow(source_texts, target_texts)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum SimpleType {
    Any,
    Unknown,
    Never,
    Primitive(CompactString),
    Array(Box<SimpleType>),
    Tuple(Vec<SimpleType>),
    Generic {
        base: CompactString,
        args: Vec<SimpleType>,
    },
    Union(Vec<SimpleType>),
    Intersection(Vec<SimpleType>),
    Other(CompactString),
}

fn parse_type_texts<T: AsRef<str>>(texts: &[T]) -> Vec<SimpleType> {
    let mut seen = FastSet::default();
    let mut parsed = Vec::with_capacity(texts.len());
    for text in texts {
        let trimmed = text.as_ref().trim();
        if !trimmed.is_empty() && seen.insert(trimmed) {
            parsed.push(parse_type_text(trimmed));
        }
    }
    parsed
}

pub(super) fn parse_type_text(text: &str) -> SimpleType {
    let text = strip_wrapping_parens(text);
    if let Some(parts) = split_top_level_once(text, '|') {
        return SimpleType::Union(parts.into_iter().map(parse_type_text).collect());
    }
    if let Some(parts) = split_top_level_once(text, '&') {
        return SimpleType::Intersection(parts.into_iter().map(parse_type_text).collect());
    }
    if let Some(stripped) = text.strip_suffix("[]") {
        return SimpleType::Array(Box::new(parse_type_text(stripped)));
    }
    if text.starts_with('[') && text.ends_with(']') && is_wrapped_by(text, '[', ']') {
        let inner = &text[1..text.len() - 1];
        return SimpleType::Tuple(
            split_comma_refs(inner)
                .into_iter()
                .map(parse_type_text)
                .collect(),
        );
    }
    if let Some((base, args)) = split_generic(text) {
        return SimpleType::Generic {
            base: base.into(),
            args: split_comma_refs(args)
                .into_iter()
                .map(parse_type_text)
                .collect(),
        };
    }
    match text {
        "any" => SimpleType::Any,
        "unknown" => SimpleType::Unknown,
        "never" => SimpleType::Never,
        "string" | "number" | "boolean" | "bigint" | "symbol" | "null" | "undefined" => {
            SimpleType::Primitive(text.into())
        }
        "true" | "false" => SimpleType::Primitive("boolean".into()),
        _ if is_string_literal(text) => SimpleType::Primitive("string".into()),
        _ if is_number_literal(text) => SimpleType::Primitive("number".into()),
        _ if is_bigint_literal(text) => SimpleType::Primitive("bigint".into()),
        _ => SimpleType::Other(text.into()),
    }
}

fn is_unsafe_flow(source: &SimpleType, target: &SimpleType) -> bool {
    if is_permissive_target(target) {
        return false;
    }
    match source {
        SimpleType::Any => true,
        SimpleType::Union(types) | SimpleType::Intersection(types) => {
            types.iter().any(|member| is_unsafe_flow(member, target))
        }
        SimpleType::Array(source_item) => match target {
            SimpleType::Array(target_item) => is_unsafe_flow(source_item, target_item),
            SimpleType::Generic { base, args } if is_array_like_base(base) && args.len() == 1 => {
                is_unsafe_flow(source_item, &args[0])
            }
            _ => false,
        },
        SimpleType::Tuple(source_items) => match target {
            SimpleType::Tuple(target_items) => source_items
                .iter()
                .zip(target_items.iter())
                .any(|(source_item, target_item)| is_unsafe_flow(source_item, target_item)),
            SimpleType::Array(target_item) => source_items
                .iter()
                .any(|source_item| is_unsafe_flow(source_item, target_item)),
            SimpleType::Generic { base, args } if is_array_like_base(base) && args.len() == 1 => {
                source_items
                    .iter()
                    .any(|source_item| is_unsafe_flow(source_item, &args[0]))
            }
            _ => false,
        },
        SimpleType::Generic {
            base: source_base,
            args: source_args,
        } => match target {
            SimpleType::Generic {
                base: target_base,
                args: target_args,
            } if same_container_family(source_base, target_base)
                && source_args.len() == target_args.len() =>
            {
                source_args
                    .iter()
                    .zip(target_args.iter())
                    .any(|(source_arg, target_arg)| is_unsafe_flow(source_arg, target_arg))
            }
            SimpleType::Array(target_item)
                if is_array_like_base(source_base) && source_args.len() == 1 =>
            {
                is_unsafe_flow(&source_args[0], target_item)
            }
            _ => false,
        },
        _ => false,
    }
}

fn contains_any_like(ty: &SimpleType) -> bool {
    match ty {
        SimpleType::Any => true,
        SimpleType::Array(inner) => contains_any_like(inner),
        SimpleType::Tuple(items) | SimpleType::Union(items) | SimpleType::Intersection(items) => {
            items.iter().any(contains_any_like)
        }
        SimpleType::Generic { args, .. } => args.iter().any(contains_any_like),
        _ => false,
    }
}

fn is_permissive_target(ty: &SimpleType) -> bool {
    match ty {
        SimpleType::Any | SimpleType::Unknown | SimpleType::Never => true,
        SimpleType::Union(types) => types.iter().any(is_permissive_target),
        _ => false,
    }
}

fn same_container_family(left: &CompactString, right: &CompactString) -> bool {
    left == right
        || (is_array_like_base(left) && is_array_like_base(right))
        || (is_promise_like_base(left) && is_promise_like_base(right))
}

fn is_array_like_base(base: &str) -> bool {
    matches!(base, "Array" | "ReadonlyArray")
}

fn is_promise_like_base(base: &str) -> bool {
    matches!(base, "Promise" | "PromiseLike")
}
