//! Computed enum-key access: `obj[key]` against the enum keys an object
//! declares (index signature keys and computed member names).
use super::super::checker::Checker;
use super::deep::mismatch;
use super::{EnumMismatch, results};
use crate::linter::corsa_session::boolean::checker_node_text;
use corsa::api::{NodeHandle, TypeResponse};

pub(super) async fn computed_receivers(
    checker: &mut Checker<'_>,
    object: &TypeResponse,
    sender: &TypeResponse,
) -> Option<EnumMismatch> {
    let object = checker.apparent(object).await;
    let receivers = computed_keys(checker, &object).await;
    if receivers.is_empty() {
        return None;
    }
    for receiver in &receivers {
        if !mismatch(checker, sender, receiver).await && checker.assignable(sender, receiver).await
        {
            return None;
        }
    }
    results(checker, &receivers).await
}

/// The key types an object's computed access is checked against: its index
/// signature keys and computed member names, read once per object type.
async fn computed_keys(checker: &mut Checker<'_>, object: &TypeResponse) -> Vec<TypeResponse> {
    if let Some(known) = checker.computed_keys.get(&object.id) {
        return known.clone();
    }
    let mut receivers = checker
        .session()
        .client()
        .get_index_infos_of_type(
            checker.session().snapshot().handle.clone(),
            checker.session().project_handle(),
            object.id.clone(),
        )
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|info| info.key_type)
        .collect::<Vec<_>>();
    for property in checker.properties(object).await {
        for declaration in property
            .declarations
            .iter()
            .chain(property.value_declaration.iter())
        {
            if let Some(ty) = computed_declaration_type(checker, declaration).await
                && !receivers.iter().any(|known| known.id == ty.id)
            {
                receivers.push(ty);
            }
        }
    }
    checker
        .computed_keys
        .insert(object.id.clone(), receivers.clone());
    receivers
}

async fn computed_declaration_type(
    checker: &mut Checker<'_>,
    declaration: &NodeHandle,
) -> Option<TypeResponse> {
    let path = declaration.declaring_path()?;
    let position = {
        let encoded = checker.source_file(path.as_str()).await?;
        computed_key_position(encoded.as_bytes(), declaration)?
    };
    checker
        .session()
        .get_type_at_position(path.as_str(), position)
        .await
        .ok()
        .flatten()
}

/// Tokens that may precede a member's name in its declaration.
const MEMBER_MODIFIERS: [&str; 14] = [
    "readonly",
    "static",
    "public",
    "private",
    "protected",
    "abstract",
    "declare",
    "override",
    "accessor",
    "async",
    "get",
    "set",
    "*",
    "async*",
];

/// Whether `prefix` (a declaration's leading trivia and modifiers) holds
/// nothing but comments, whitespace and member modifiers.
fn only_modifiers(prefix: &str) -> bool {
    let mut rest = prefix;
    loop {
        rest = rest.trim_start();
        if let Some(line) = rest.strip_prefix("//") {
            rest = line.split_once('\n').map_or("", |(_, after)| after);
        } else if let Some(block) = rest.strip_prefix("/*") {
            let Some((_, after)) = block.split_once("*/") else {
                return false;
            };
            rest = after;
        } else {
            let (token, after) = rest.split_once(char::is_whitespace).unwrap_or((rest, ""));
            if token.is_empty() {
                return true;
            }
            if !MEMBER_MODIFIERS.contains(&token) {
                return false;
            }
            rest = after;
        }
    }
}

fn computed_key_position(data: &[u8], declaration: &NodeHandle) -> Option<u32> {
    let (source, pos, end) = match declaration.parse() {
        Ok(parsed) => {
            let (source, _, _) = checker_node_text(data, 1).ok()?;
            (source, parsed.pos, parsed.end)
        }
        Err(_) => {
            let index = declaration
                .as_str()
                .split('.')
                .next()?
                .parse::<usize>()
                .ok()?;
            checker_node_text(data, index).ok()?
        }
    };
    let start = utf16_to_byte(source, pos)?;
    let stop = utf16_to_byte(source, end)?;
    let text = source.get(start..stop)?;
    let open = text.find('[')?;
    // A computed name is the declaration's first token after its modifiers;
    // a `[` anywhere later (a parameter list, a method body, an initializer)
    // is not its key.
    if !only_modifiers(text.get(..open)?) {
        return None;
    }
    let close = text.get(open + 1..)?.find(']')?.saturating_add(open + 1);
    let inside = text.get(open + 1..close)?;
    // `[index: number]` is an index signature, not a computed name; a colon
    // inside a comment (`[Fruit.Apple /* key: x */]`) does not make one.
    if without_comments(inside).contains(':') {
        return None;
    }
    let leading = inside.len().saturating_sub(inside.trim_start().len());
    let mapped = inside.find(" in ").map(|offset| offset + 4);
    // `getTypeAtLocation(name.expression)`: the type at a property access's
    // name is the access's own type (`Fruit.Apple`), while the type at its
    // object is the enum object (`typeof Fruit`).
    let key = mapped.unwrap_or_else(|| {
        let expression = inside.trim_end();
        expression
            .rfind('.')
            .map_or(leading, |dot| dot + 1)
            .max(leading)
    });
    let relative = open + 1 + key;
    let prefix = text.get(..relative)?;
    Some(pos.saturating_add(prefix.encode_utf16().count() as u32))
}

/// `text` with its `//` and `/* */` comments removed.
fn without_comments(text: &str) -> vize_l0::String {
    let mut out = vize_l0::String::default();
    let mut rest = text;
    while let Some(index) = rest.find('/') {
        let (Some(code), Some(from)) = (rest.get(..index), rest.get(index..)) else {
            break;
        };
        out.push_str(code);
        if let Some(block) = from.strip_prefix("/*") {
            rest = block.split_once("*/").map_or("", |(_, after)| after);
        } else if let Some(line) = from.strip_prefix("//") {
            rest = line.split_once('\n').map_or("", |(_, after)| after);
        } else {
            out.push('/');
            rest = from.get(1..).unwrap_or_default();
        }
    }
    out.push_str(rest);
    out
}

fn utf16_to_byte(text: &str, offset: u32) -> Option<usize> {
    let mut units = 0u32;
    for (index, character) in text.char_indices() {
        if units >= offset {
            return Some(index);
        }
        units += character.len_utf16() as u32;
    }
    (units >= offset).then_some(text.len())
}
