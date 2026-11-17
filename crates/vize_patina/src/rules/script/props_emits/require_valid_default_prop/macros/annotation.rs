//! Runtime types of an imported prop's authored annotation.
//!
//! The type world returns each member's type text and its lexical module.
//! Every union part maps to the constructor Vue checks. A reference resolves
//! through the world, and an interface is an object. A part that resolves to
//! nothing known (a DOM interface such as `ScrollIntoViewOptions`, `any`, a
//! type parameter) may be an object, so the prop is left unchecked.

use vize_croquis::types::world::TypeDeclarationKind;
use vize_croquis::types::{ResolvedTypeWorld, TypeLookup};

/// Alias hops followed before a type is treated as unknown.
const MAX_ALIAS_DEPTH: usize = 4;

/// The constructors `annotation` allows, or `None` when one part is unknown.
pub(super) fn runtime_types(
    world: &ResolvedTypeWorld,
    module: &str,
    annotation: &str,
) -> Option<Vec<&'static str>> {
    let mut types = Vec::new();
    collect(world, module, annotation, 0, &mut types)?;
    Some(types)
}

fn collect(
    world: &ResolvedTypeWorld,
    module: &str,
    annotation: &str,
    depth: usize,
    types: &mut Vec<&'static str>,
) -> Option<()> {
    for part in union_parts(annotation) {
        let part = without_parentheses(part);
        if matches!(part, "" | "null" | "undefined" | "void" | "never") {
            continue;
        }
        if let Some(runtime) = shape(part) {
            push(types, runtime);
            continue;
        }
        let name = reference_name(part)?;
        let runtime = match name {
            "Array" | "ReadonlyArray" => "Array",
            "Function" => "Function",
            "Object" | "Record" | "Partial" | "Readonly" | "Pick" | "Omit" | "Required" => "Object",
            _ => {
                let TypeLookup::Found(id) = world.resolve(module, name) else {
                    return None;
                };
                let declaration = world.declaration(&id)?;
                if declaration.kind == TypeDeclarationKind::Alias {
                    if depth >= MAX_ALIAS_DEPTH || declaration.type_parameters.is_some() {
                        return None;
                    }
                    collect(world, &id.module, &declaration.body, depth + 1, types)?;
                    continue;
                }
                "Object"
            }
        };
        push(types, runtime);
    }
    Some(())
}

/// A keyword, literal, object, array or function type. A function or array
/// suffix wins over a leading `{` or literal: `{ label: string }[]` is an array.
fn shape(part: &str) -> Option<&'static str> {
    if has_top_level(part, "=>") {
        return Some("Function");
    }
    if part.starts_with('[') || part.ends_with("[]") {
        return Some("Array");
    }
    Some(match part {
        "string" | "String" => "String",
        "number" | "Number" => "Number",
        "boolean" | "Boolean" | "true" | "false" => "Boolean",
        "bigint" | "BigInt" => "BigInt",
        "symbol" | "Symbol" => "Symbol",
        "object" => "Object",
        _ if part.starts_with(['\'', '"', '`']) => "String",
        _ if part.starts_with(|character: char| character.is_ascii_digit() || character == '-') => {
            "Number"
        }
        _ if part.starts_with('{') => "Object",
        _ => return None,
    })
}

/// `Name` or `Name<...>`; a qualified or computed type has no plain name.
fn reference_name(part: &str) -> Option<&str> {
    let end = part
        .find(|character: char| {
            !(character.is_ascii_alphanumeric() || matches!(character, '_' | '$'))
        })
        .unwrap_or(part.len());
    let (name, rest) = part.split_at(end);
    let starts = name.starts_with(|character: char| {
        character.is_ascii_alphabetic() || matches!(character, '_' | '$')
    });
    (starts && (rest.is_empty() || (rest.starts_with('<') && rest.ends_with('>')))).then_some(name)
}

fn without_parentheses(mut part: &str) -> &str {
    part = part.trim();
    while let Some(inner) = part
        .strip_prefix('(')
        .and_then(|inner| inner.strip_suffix(')'))
    {
        if !balanced(inner) {
            break;
        }
        part = inner.trim();
    }
    part
}

/// The top-level `|` operands of a type, outside brackets and strings.
fn union_parts(annotation: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    scan(annotation, |index, character, depth| {
        if character == '|' && depth == 0 {
            parts.push(annotation.get(start..index).unwrap_or_default());
            start = index + 1;
        }
    });
    parts.push(annotation.get(start..).unwrap_or_default());
    parts
}

fn has_top_level(part: &str, token: &str) -> bool {
    let mut found = false;
    scan(part, |index, _, depth| {
        found |= depth == 0
            && part
                .get(index..)
                .is_some_and(|rest| rest.starts_with(token));
    });
    found
}

fn balanced(text: &str) -> bool {
    let mut lowest = 0i32;
    let mut last = 0i32;
    scan(text, |_, _, depth| {
        lowest = lowest.min(depth);
        last = depth;
    });
    lowest >= 0 && last == 0
}

/// Calls `visit(index, character, depth)` for each code character, with the
/// bracket depth before it; quoted text is skipped.
fn scan(text: &str, mut visit: impl FnMut(usize, char, i32)) {
    let mut depth = 0i32;
    let mut quote = None;
    let mut escaped = false;
    for (index, character) in text.char_indices() {
        if let Some(delimiter) = quote {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == delimiter {
                quote = None;
            }
            continue;
        }
        match character {
            '\'' | '"' | '`' => quote = Some(character),
            '(' | '[' | '{' | '<' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            '>' if !text
                .get(..index)
                .is_some_and(|before| before.ends_with('=')) =>
            {
                depth -= 1
            }
            _ => {}
        }
        visit(index, character, depth);
    }
}

fn push(types: &mut Vec<&'static str>, runtime: &'static str) {
    if !types.contains(&runtime) {
        types.push(runtime);
    }
}
