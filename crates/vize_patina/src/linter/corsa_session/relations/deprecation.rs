//! typescript-eslint `no-deprecated`: the `@deprecated` JSDoc of the symbol a
//! use refers to, walked through import/export aliases, and, for calls, of
//! the one overload the call resolves to rather than the tags every
//! declaration of the callee's symbol contributes.
use super::checker::Checker;
use super::{Deprecation, DeprecationAsk, DeprecationKind, Located, Span, flags};
use crate::linter::corsa_session::boolean::checker_node_text;
use corsa::api::{NodeHandle, SymbolResponse, TypeResponse};
use vize_l0::String as CompactString;

/// TypeScript `SymbolFlags` the rule reads.
const SYMBOL_FUNCTION: u32 = 1 << 4;
const SYMBOL_METHOD: u32 = 1 << 13;
const SYMBOL_ALIAS: u32 = 1 << 21;

/// Whether one use is deprecated, and why.
pub(super) async fn check(
    checker: &mut Checker<'_>,
    located: &Located,
    ask: &DeprecationAsk,
    symbol: Option<SymbolResponse>,
    translate: &dyn Fn(Span) -> Span,
) -> Option<Deprecation> {
    let name = match &ask.kind {
        DeprecationKind::ContextualProperty {
            name: None,
            key: Some(key),
            ..
        } => located.ty(translate(*key)).and_then(literal_name),
        DeprecationKind::Member { property, .. } => {
            located.ty(translate(*property)).and_then(literal_name)
        }
        _ => None,
    };
    let reason = reason(checker, located, ask, symbol, translate).await?;
    Some(Deprecation { reason, name })
}

/// The deprecation reason of one use (`Some("")` without a reason).
async fn reason(
    checker: &mut Checker<'_>,
    located: &Located,
    ask: &DeprecationAsk,
    symbol: Option<SymbolResponse>,
    translate: &dyn Fn(Span) -> Span,
) -> Option<CompactString> {
    match &ask.kind {
        DeprecationKind::Symbol => alias_chain(checker, symbol, true).await,
        DeprecationKind::ExportAlias => {
            let symbol = symbol?;
            if tags(checker, &symbol).await.is_some() {
                return None;
            }
            alias_chain(checker, Some(symbol), true).await
        }
        DeprecationKind::CallLike { call } => {
            let symbol = symbol?;
            let aliased = if symbol.flags & SYMBOL_ALIAS != 0 {
                checker.aliased_symbol(&symbol).await
            } else {
                Some(symbol.clone())
            };
            let function_like = aliased
                .as_ref()
                .is_some_and(|aliased| aliased.flags & (SYMBOL_FUNCTION | SYMBOL_METHOD) != 0);
            if function_like {
                // Overloads each carry their own JSDoc; the symbol merges them.
                if let Some(found) = alias_chain(checker, Some(symbol), false).await {
                    return Some(found);
                }
                let merged = match &aliased {
                    Some(aliased) => tags(checker, aliased).await,
                    None => None,
                };
                merged?;
                let handle = located.handle(translate(*call))?;
                return signature(checker, handle).await;
            }
            if let Some(found) = alias_chain(checker, Some(symbol), true).await {
                return Some(found);
            }
            if let Some(handle) = located.handle(translate(*call))
                && let Some(found) = signature(checker, handle).await
            {
                return Some(found);
            }
            match &aliased {
                Some(aliased) => tags(checker, aliased).await,
                None => None,
            }
        }
        DeprecationKind::ObjectProperty { object, name } => {
            if let Some(found) = alias_chain(checker, symbol.clone(), true).await {
                return Some(found);
            }
            if let Some(object) = located.ty(translate(*object)).cloned()
                && let Some(property) = checker.property(&object, name).await
                && let Some(found) = tags(checker, &property).await
            {
                return Some(found);
            }
            if let Some(symbol) = &symbol
                && let Some(found) = tags(checker, symbol).await
            {
                return Some(found);
            }
            let value = match symbol.and_then(|symbol| symbol.value_declaration) {
                Some(declaration) => checker.shorthand_value_symbol(&declaration).await,
                None => None,
            };
            alias_chain(checker, value, true).await
        }
        DeprecationKind::ContextualProperty { object, name, key } => {
            let name = match (name, key) {
                (Some(name), _) => name.clone(),
                (None, Some(key)) => literal_name(located.ty(translate(*key))?)?,
                (None, None) => return None,
            };
            let handle = located.handle(translate(*object))?;
            let contextual = checker.contextual(handle).await?;
            let property = checker.property(&contextual, &name).await?;
            tags(checker, &property).await
        }
        DeprecationKind::Member { object, property } => {
            let name = literal_name(located.ty(translate(*property))?)?;
            let object = located.ty(translate(*object))?.clone();
            let property = checker.property(&object, &name).await?;
            tags(checker, &property).await
        }
    }
}

/// The literal value of a string or number literal type, as a property name.
pub(super) fn literal_name(ty: &TypeResponse) -> Option<CompactString> {
    if ty.flags & (flags::STRING_LITERAL | flags::NUMBER_LITERAL) == 0 {
        return None;
    }
    match ty.value.as_ref()? {
        serde_json::Value::String(value) => Some(CompactString::from(value.as_str())),
        serde_json::Value::Number(value) => Some(vize_l0::cstr!("{value}")),
        _ => None,
    }
}

/// `getJsDocDeprecation(symbol)`.
async fn tags(checker: &mut Checker<'_>, symbol: &SymbolResponse) -> Option<CompactString> {
    checker.deprecation_tag(symbol).await
}

/// `searchForDeprecationInAliasesChain(symbol, checkDeprecationsOfAliasedSymbol)`.
/// Longest alias chain followed; real re-export chains are far shorter.
const MAX_ALIAS_CHAIN: usize = 64;

async fn alias_chain(
    checker: &mut Checker<'_>,
    symbol: Option<SymbolResponse>,
    check_target: bool,
) -> Option<CompactString> {
    let mut symbol = symbol?;
    if symbol.flags & SYMBOL_ALIAS == 0 {
        return if check_target {
            tags(checker, &symbol).await
        } else {
            None
        };
    }
    let target = checker.aliased_symbol(&symbol).await;
    // An unresolvable or circular alias (`export { a as b, b as a }`) can
    // lead back to itself; the checker's own walk stops there too.
    let mut seen = Vec::new();
    while symbol.flags & SYMBOL_ALIAS != 0 && seen.len() < MAX_ALIAS_CHAIN {
        if seen.contains(&symbol.id) {
            break;
        }
        seen.push(symbol.id.clone());
        if let Some(found) = tags(checker, &symbol).await {
            return Some(found);
        }
        if symbol.declarations.is_empty() {
            break;
        }
        let Some(next) = checker.immediate_aliased_symbol(&symbol).await else {
            break;
        };
        symbol = next;
        if check_target && target.as_ref().is_some_and(|target| target.id == symbol.id) {
            return tags(checker, &symbol).await;
        }
    }
    None
}

/// `getJsDocDeprecation(checker.getResolvedSignature(call))`.
async fn signature(checker: &mut Checker<'_>, call: &NodeHandle) -> Option<CompactString> {
    let session = checker.session();
    let resolved = session
        .client()
        .get_resolved_signature(
            session.snapshot().handle.clone(),
            session.project_handle(),
            call.clone(),
        )
        .await
        .ok()
        .flatten()?;
    let declaration = resolved.declaration?;
    let path = declaration.declaring_path()?;
    let encoded = checker.source_file(path.as_str()).await?;
    let (text, pos, end) = match declaration.parse() {
        Ok(parsed) => {
            let (text, _, _) = checker_node_text(encoded.as_bytes(), 1).ok()?;
            (text, parsed.pos, parsed.end)
        }
        Err(_) => {
            let index = declaration
                .as_str()
                .split('.')
                .next()?
                .parse::<usize>()
                .ok()?;
            checker_node_text(encoded.as_bytes(), index).ok()?
        }
    };
    let start = utf16_to_byte(text, pos)?;
    let stop = utf16_to_byte(text, end)?;
    deprecated_tag(text.get(start..stop)?)
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

/// The reason of a `@deprecated` tag in the JSDoc comments that lead `node`.
pub(in crate::linter) fn deprecated_tag(node: &str) -> Option<CompactString> {
    let mut rest = node;
    let mut found = None;
    loop {
        rest = rest.trim_start();
        if let Some(body) = rest.strip_prefix("/**") {
            let close = body.find("*/")?;
            let comment = body.get(..close)?;
            if let Some(reason) = tag_reason(comment) {
                found = Some(reason);
            }
            rest = body.get(close + 2..)?;
        } else if let Some(body) = rest.strip_prefix("/*") {
            let close = body.find("*/")?;
            rest = body.get(close + 2..)?;
        } else if let Some(body) = rest.strip_prefix("//") {
            rest = body
                .find('\n')
                .and_then(|line| body.get(line..))
                .unwrap_or("");
        } else {
            return found;
        }
    }
}

fn tag_reason(comment: &str) -> Option<CompactString> {
    let mut reason: Option<CompactString> = None;
    for line in comment.lines() {
        let line = line.trim_start();
        let line = line.strip_prefix('*').unwrap_or(line).trim();
        if let Some(collected) = reason.as_mut() {
            if line.starts_with('@') {
                break;
            }
            if !line.is_empty() {
                if !collected.is_empty() {
                    collected.push(' ');
                }
                collected.push_str(line);
            }
            continue;
        }
        let Some(at) = line.find("@deprecated") else {
            continue;
        };
        let after = line.get(at + "@deprecated".len()..).unwrap_or("");
        if after
            .chars()
            .next()
            .is_some_and(|next| next.is_alphanumeric() || next == '_')
        {
            continue;
        }
        let before = line.get(..at).unwrap_or("");
        if !before.trim().is_empty() && !before.ends_with('{') {
            continue;
        }
        reason = Some(CompactString::from(after.trim()));
    }
    reason
}

#[cfg(test)]
mod tests {
    use super::deprecated_tag;

    #[test]
    fn reads_the_tag_of_the_leading_jsdoc_only() {
        let text = "\n  /** @deprecated */\n  querySelector(a: string): Element;";
        assert_eq!(deprecated_tag(text).as_deref(), Some(""));
        let text =
            "\n  /**\n   * Returns the first element.\n   */\n  querySelector(a: string): Element;";
        assert_eq!(deprecated_tag(text), None);
    }

    #[test]
    fn keeps_the_reason_until_the_next_tag() {
        let text = "/**\n * @deprecated Use b(value).\n * @param x what\n */ b(): string;";
        assert_eq!(deprecated_tag(text).as_deref(), Some("Use b(value)."));
    }

    #[test]
    fn ignores_block_and_line_comments() {
        let text = "/* @deprecated */\n// @deprecated\nfunction f(): void;";
        assert_eq!(deprecated_tag(text), None);
    }
}
