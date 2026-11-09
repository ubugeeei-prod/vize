//! Values, interpolation and `v-for` iteration of the reference.

use super::expression::{self, integer};
use super::json::{Json, entries, get, get_int, get_str, set};
use super::program::{Op, OpKind, Program};
use super::values::{Operand, for_op, one};

pub const UNSAFE_NAMES: [&str; 3] = ["__proto__", "constructor", "prototype"];

pub fn identifier(text: &str) -> bool {
    let start = |c: char| c.is_ascii_alphabetic() || c == '_' || c == '$';
    let mut chars = text.chars();
    chars.next().is_some_and(start) && chars.all(|c| start(c) || c.is_ascii_digit())
}

pub fn path(text: &str) -> bool {
    text.split('.')
        .all(|part| identifier(part) && !UNSAFE_NAMES.contains(&part))
}

/// `toDisplayString` over the reference value domain.
pub fn display(value: &Json) -> Result<String, String> {
    match value {
        Json::Null => Ok(String::new()),
        Json::String(text) => Ok(text.clone()),
        Json::Bool(b) => Ok(if *b { "true" } else { "false" }.into()),
        Json::Number(_) => {
            let n = get_int(value)?;
            match i32::try_from(n) {
                Ok(n) => Ok(n.to_string()),
                Err(_) => Err("unsupported display number".into()),
            }
        }
        _ => Err("unsupported display value".into()),
    }
}

/// Literal text whose DOM spelling needs no entity decoding or whitespace condensing.
pub fn plain_text(text: &str) -> bool {
    !text.contains(['&', '<', '\n', '\t', '\r']) && !text.contains("  ")
}

/// Compound text as L3 preserves it: literal runs and `{{ expression }}` holes.
pub fn compound_parts(text: &str) -> Result<Vec<(bool, String)>, String> {
    let mut pieces = text.split("{{");
    let first = pieces.next().unwrap_or_default();
    if !plain_text(first) || first.contains('}') {
        return Err("unsupported compound literal".into());
    }
    let mut parts = Vec::new();
    if !first.is_empty() {
        parts.push((false, first.to_owned()));
    }
    for piece in pieces {
        let [expr, literal] = piece.split("}}").collect::<Vec<_>>()[..] else {
            return Err("unbalanced compound interpolation".into());
        };
        if !plain_text(literal) || literal.contains('}') {
            return Err("unsupported compound literal".into());
        }
        parts.push((true, expr.to_owned()));
        if !literal.is_empty() {
            parts.push((false, literal.to_owned()));
        }
    }
    if !parts.iter().any(|(is_expr, _)| *is_expr) {
        return Err("compound text without interpolation".into());
    }
    Ok(parts)
}

pub fn validate_expression(row: &Operand) -> Result<(), String> {
    if row.kind == "literal" {
        return Ok(());
    }
    if row.kind == "opaque" && row.qualifier == "compound" {
        for (is_expr, text) in compound_parts(&row.text)? {
            if is_expr {
                expression::parse(&text)
                    .map_err(|e| format!("unsupported interpolation {text}: {e}"))?;
            }
        }
        return Ok(());
    }
    if row.kind != "js" {
        return Err(format!("unsupported expression {}: {}", row.kind, row.text));
    }
    expression::parse(&row.text)
        .map(|_| ())
        .map_err(|e| format!("unsupported expression {}: {e}", row.text))
}

pub fn lookup(context: &Json, text: &str) -> Result<Json, String> {
    if !path(text) {
        return Err("unsupported member path".into());
    }
    let mut value = context;
    for field in text.split('.') {
        value = get(value, field)?;
    }
    Ok(value.clone())
}

pub fn evaluate(context: &Json, row: &Operand) -> Result<Json, String> {
    validate_expression(row)?;
    if row.kind == "literal" {
        return Ok(Json::String(row.text.clone()));
    }
    if row.kind == "opaque" {
        let mut text = String::new();
        for (is_expr, part) in compound_parts(&row.text)? {
            match is_expr {
                true => text.push_str(&display(&expression::evaluate(context, &part)?)?),
                false => text.push_str(&part),
            }
        }
        return Ok(Json::String(text));
    }
    expression::evaluate(context, &row.text)
}

pub fn record_argument(text: &str) -> Option<&str> {
    let argument = text.strip_prefix("record(")?.strip_suffix(')')?;
    path(argument).then_some(argument)
}

pub fn event(context: &Json, row: &Operand) -> Result<Json, String> {
    if row.kind != "js" {
        return Err("unsupported event value".into());
    }
    if row.text == "save" {
        return Ok(Json::String("save".into()));
    }
    let Some(argument) = record_argument(&row.text) else {
        return Err("unsupported event expression".into());
    };
    Ok(Json::String(get_str(&lookup(context, argument)?)?.into()))
}

pub fn alias(row: &Operand) -> Result<&str, String> {
    let kind_ok = row.kind == "js" || (row.kind == "opaque" && row.qualifier == "for-value");
    let reserved = ["save", "record", "$event", "$slots"];
    if !kind_ok
        || !identifier(&row.text)
        || reserved.contains(&row.text.as_str())
        || UNSAFE_NAMES.contains(&row.text.as_str())
    {
        return Err("unsupported loop alias".into());
    }
    Ok(&row.text)
}

fn owned_regions(program: &Program, op: u32) -> Vec<u32> {
    program
        .regions
        .iter()
        .filter(|r| r.owner == Some(op))
        .map(|r| r.id)
        .collect()
}

pub fn validate(program: &Program, rows: &[Operand], op: &Op) -> Result<(), String> {
    if for_op(rows, op.id).len() != 4 {
        return Err("expected four loop operands".into());
    }
    let source = one(rows, op.id, "for-source")?;
    if source.kind != "js" {
        return Err("unsupported loop source".into());
    }
    validate_expression(source)?;
    let mut aliases = vec![alias(one(rows, op.id, "for-value")?)?];
    for role in ["for-key", "for-index"] {
        let row = one(rows, op.id, role)?;
        if row.kind != "absent" {
            let name = alias(row)?;
            if aliases.contains(&name) {
                return Err("duplicate loop alias".into());
            }
            aliases.push(name);
        }
    }
    if one(rows, op.id, "for-key")?.kind == "absent"
        && one(rows, op.id, "for-index")?.kind != "absent"
    {
        return Err("third loop alias without a second alias".into());
    }
    let [region] = owned_regions(program, op.id)[..] else {
        return Err("expected one loop body".into());
    };
    let body = || {
        program
            .ops
            .iter()
            .filter(move |child| child.region == region)
    };
    let roots: Vec<&Op> = body().filter(|c| c.kind == OpKind::InsertNode).collect();
    let [root] = roots[..] else {
        return Err("expected one native loop root".into());
    };
    let allowed = [
        OpKind::InsertNode,
        OpKind::SetProp,
        OpKind::SetEvent,
        OpKind::SetText,
    ];
    if body().any(|child| !allowed.contains(&child.kind)) {
        return Err("unsupported loop root shape".into());
    }
    if body().any(|child| {
        child.kind == OpKind::SetText
            && !for_op(rows, child.id)
                .iter()
                .any(|row| row.target == Some(root.id))
    }) {
        return Err("loop root text must belong to its element".into());
    }
    Ok(())
}

pub fn key_value(value: &Json) -> Result<Json, String> {
    match value {
        Json::String(_) => Ok(value.clone()),
        Json::Number(_) => match i32::try_from(get_int(value)?) {
            Ok(_) => Ok(value.clone()),
            Err(_) => Err("unsupported loop key integer".into()),
        },
        _ => Err("loop keys must be strings or signed 32-bit integers".into()),
    }
}

/// JavaScript enumerates canonical array-index keys before other keys, so
/// objects with such keys have no order shared with the JSON reference.
pub fn array_index_key(key: &str) -> bool {
    key.bytes().all(|b| b.is_ascii_digit())
        && !key.is_empty()
        && key
            .parse::<u64>()
            .is_ok_and(|n| n.to_string() == key && n < 4_294_967_295)
}

/// One iteration entry: value, second alias and optional third alias.
pub type Entry = (Json, Json, Json);

/// Iteration entries as `renderList` produces them. Objects iterate in key
/// order, and ranges yield `1..n`.
pub fn entries_of(source: &Json, third: bool) -> Result<Vec<Entry>, String> {
    match source {
        Json::Array(items) => {
            if third {
                return Err("third loop alias requires an object source".into());
            }
            Ok((items.iter().enumerate())
                .map(|(index, item)| (item.clone(), Json::from(index), Json::Null))
                .collect())
        }
        Json::Number(_) => {
            if third {
                return Err("third loop alias requires an object source".into());
            }
            let n = integer(source)?;
            if !(0..=1000).contains(&n) {
                return Err("unsupported range source".into());
            }
            Ok((0..n)
                .map(|index| {
                    (
                        Json::from(index as i64 + 1),
                        Json::from(index as i64),
                        Json::Null,
                    )
                })
                .collect())
        }
        Json::Object(_) => {
            let pairs = entries(source)?;
            if pairs.iter().any(|(key, _)| array_index_key(key)) {
                return Err("unsupported integer-like object key".into());
            }
            Ok((pairs.into_iter().enumerate())
                .map(|(index, (key, value))| {
                    (value.clone(), Json::String(key.clone()), Json::from(index))
                })
                .collect())
        }
        _ => Err("unsupported loop source value".into()),
    }
}

/// Scope of every iteration with its loop identity. `for-key` is the second
/// alias (index or object key); Vue's `:key` belongs to the repeated root.
pub fn contexts(
    program: &Program,
    rows: &[Operand],
    op: &Op,
    context: &Json,
) -> Result<Vec<(Json, Json)>, String> {
    validate(program, rows, op)?;
    let [region] = owned_regions(program, op.id)[..] else {
        return Err("missing loop body".into());
    };
    let Some(root) = (program.ops.iter())
        .find(|child| child.region == region && child.kind == OpKind::InsertNode)
    else {
        return Err("missing loop root".into());
    };
    let keys: Vec<&Operand> = (rows.iter())
        .filter(|row| {
            row.role == "name"
                && row.target == Some(root.id)
                && row.kind == "literal"
                && row.text == "key"
        })
        .collect();
    if keys.len() > 1 {
        return Err("duplicate loop key binding".into());
    }
    let value_alias = alias(one(rows, op.id, "for-value")?)?;
    let optional = |role| -> Result<Option<&str>, String> {
        let row = one(rows, op.id, role)?;
        match row.kind == "absent" {
            true => Ok(None),
            false => alias(row).map(Some),
        }
    };
    let second = optional("for-key")?;
    let third = optional("for-index")?;
    let source = evaluate(context, one(rows, op.id, "for-source")?)?;
    let mut result = Vec::new();
    let mut seen: Vec<Json> = Vec::new();
    for (index, (item, second_value, third_value)) in entries_of(&source, third.is_some())?
        .into_iter()
        .enumerate()
    {
        let mut scope = set(context, value_alias, item)?;
        if let Some(name) = second {
            scope = set(&scope, name, second_value)?;
        }
        if let Some(name) = third {
            scope = set(&scope, name, third_value)?;
        }
        let identity = match keys[..] {
            [] => Json::Array(vec!["position".into(), Json::from(index)]),
            [key] => {
                let value = key_value(&evaluate(&scope, one(rows, key.op, "value")?)?)?;
                if seen.contains(&value) {
                    return Err("duplicate loop key".into());
                }
                seen.push(value.clone());
                Json::Array(vec!["key".into(), value])
            }
            _ => return Err("ambiguous loop key".into()),
        };
        result.push((Json::Array(vec![Json::from(op.id), identity]), scope));
    }
    Ok(result)
}
