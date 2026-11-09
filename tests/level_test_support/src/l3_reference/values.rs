//! Parser for the committed `[s3-values-dump]` operand pages.

use super::dump::field;
use super::json::{Json, get_arr, get_nat, get_str, parse as parse_json};
use super::program::Span;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Operand {
    pub op: u32,
    pub role: String,
    pub target: Option<u32>,
    pub region: Option<u32>,
    pub name: Option<String>,
    pub kind: String,
    pub text: String,
    pub qualifier: String,
    pub span: Span,
}

pub const ROLES: [&str; 19] = [
    "tag",
    "namespace",
    "attribute",
    "text",
    "comment",
    "binding-kind",
    "name",
    "value",
    "modifier",
    "condition",
    "for-source",
    "for-value",
    "for-key",
    "for-index",
    "model-read",
    "model-write",
    "model-attribute",
    "params",
    "child-value",
];

pub const KINDS: [&str; 6] = ["absent", "literal", "js", "opaque", "foreign", "vue.filter"];

const TARGETED: [&str; 7] = [
    "binding-kind",
    "value",
    "modifier",
    "model-read",
    "model-write",
    "model-attribute",
    "params",
];

fn u32_of(value: &Json) -> Result<u32, String> {
    u32::try_from(get_nat(value)?).map_err(|_| String::from("L3 integer exceeds u32"))
}

fn optional<T>(
    parse: impl Fn(&Json) -> Result<T, String>,
    value: &Json,
) -> Result<Option<T>, String> {
    match value {
        Json::Null => Ok(None),
        _ => parse(value).map(Some),
    }
}

/// One `operand=[...]` record.
pub fn parse_operand(text: &str) -> Result<Operand, String> {
    let row = parse_json(field(0, "operand", text)?)?;
    let fields = get_arr(&row)?;
    let [
        op,
        role,
        target,
        region,
        name,
        kind,
        value,
        qualifier,
        start,
        stop,
    ] = fields.as_slice()
    else {
        return Err("expected 10 L3 operand fields".into());
    };
    let op = u32_of(op)?;
    let role = get_str(role)?;
    let target = optional(u32_of, target)?;
    let region = optional(u32_of, region)?;
    let name = optional(|v| get_str(v).map(String::from), name)?;
    let kind = get_str(kind)?;
    let text = get_str(value)?;
    let qualifier = get_str(qualifier)?;
    let (start, stop) = (u32_of(start)?, u32_of(stop)?);
    if !ROLES.contains(&role) {
        return Err("unknown L3 operand role".into());
    }
    if !KINDS.contains(&kind) {
        return Err("unknown L3 value kind".into());
    }
    if start > stop
        || (kind == "absent" && !text.is_empty())
        || (matches!(kind, "opaque" | "foreign") == qualifier.is_empty())
    {
        return Err("malformed L3 operand value".into());
    }
    let requires_target = TARGETED.contains(&role);
    if (target.is_none() && requires_target)
        || (target.is_some() && !requires_target && !matches!(role, "tag" | "name"))
        || (name.is_some() != matches!(role, "attribute" | "model-attribute"))
        || (region.is_some() != (role == "condition"))
    {
        return Err("malformed L3 operand references".into());
    }
    Ok(Operand {
        op,
        role: role.into(),
        target,
        region,
        name,
        kind: kind.into(),
        text: text.into(),
        qualifier: qualifier.into(),
        span: Span { start, stop },
    })
}

/// One value page, rows in page order.
pub fn parse(text: &str) -> Result<Vec<Operand>, String> {
    let mut current = 0;
    let mut rows = Vec::new();
    for (index, raw) in text.split('\n').enumerate() {
        let line = raw.trim_ascii();
        if line.is_empty() {
            continue;
        }
        if line == "[s3-values-dump]" && current == 0 {
            current = 1;
        } else if line == "[s3-values-dump.operands]" && current == 1 {
            current = 2;
        } else if current == 2 {
            rows.push(parse_operand(line).map_err(|e| format!("line {}: {e}", index + 1))?);
        } else {
            return Err(format!("line {}: expected L3 value section", index + 1));
        }
    }
    if current != 2 {
        return Err("missing L3 value sections".into());
    }
    Ok(rows)
}

pub fn for_op(rows: &[Operand], id: u32) -> Vec<&Operand> {
    rows.iter().filter(|row| row.op == id).collect()
}

pub fn one<'r>(rows: &'r [Operand], id: u32, role: &str) -> Result<&'r Operand, String> {
    match for_op(rows, id)
        .into_iter()
        .filter(|row| row.role == role)
        .collect::<Vec<_>>()
        .as_slice()
    {
        [row] => Ok(*row),
        _ => Err(format!("op#{id}: expected exactly one {role} operand")),
    }
}

pub fn literal(row: &Operand) -> Result<&str, String> {
    match row.kind.as_str() {
        "literal" => Ok(&row.text),
        _ => Err(format!("expected literal {}", row.role)),
    }
}
