//! Bounded style evaluation from the producer's original expression AST.

use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind, PropertyKey, PropertyKind};
use vize_l0::{FxHashMap, String, profiler::global_profiler};

/// Serialize a pure object with literal string keys/values as Vue inline CSS.
///
/// This never parses source, resolves references, calls code, or clones an AST.
/// Unknown shapes keep the runtime style operation. Integer/prototype keys and
/// CSS declaration separators are deliberately outside this bounded contract.
/// Duplicate keys retain their first insertion position and last value, before
/// camel-case keys are normalized (distinct JS keys can normalize identically).
/// Serialization follows Vue 3.6.0-rc.11 `stringifyStyle` and `hyphenate`.
pub fn literal_object_style(expression: &Expression<'_>) -> Option<String> {
    let Expression::ObjectExpression(object) = expression else {
        return None;
    };
    if object.properties.is_empty() {
        return None;
    }
    let profiler = global_profiler();
    let profiling = profiler.is_enabled();
    let result = vize_l0::profile!("relief.inline_style.literal_object", evaluate(object));
    if profiling {
        profiler.record_counter_enabled("relief.inline_style.literal_object_candidates", 1);
        profiler.record_counter_enabled(
            "relief.inline_style.literal_object_folds",
            u64::from(result.is_some()),
        );
        profiler.record_counter_enabled(
            "relief.inline_style.literal_object_output_bytes",
            result.as_ref().map_or(0, |css| css.len() as u64),
        );
    }
    result
}

fn evaluate(object: &ObjectExpression<'_>) -> Option<String> {
    let mut properties = Vec::with_capacity(object.properties.len());
    let mut indices = FxHashMap::default();
    for item in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = item else {
            return None;
        };
        if property.kind != PropertyKind::Init
            || property.method
            || property.shorthand
            || property.computed
        {
            return None;
        }
        let key = match &property.key {
            PropertyKey::StaticIdentifier(identifier) => identifier.name.as_str(),
            PropertyKey::StringLiteral(literal) if !literal.lone_surrogates => {
                literal.value.as_str()
            }
            _ => return None,
        };
        if !style_key(key) {
            return None;
        }
        let Expression::StringLiteral(value) = &property.value else {
            return None;
        };
        if value.lone_surrogates {
            return None;
        }
        let value = value.value.as_str();
        if value.is_empty()
            || value.chars().any(|c| {
                c.is_control()
                    || matches!(
                        c,
                        ';' | '{' | '}' | '(' | ')' | '[' | ']' | '\'' | '"' | '\\'
                    )
            })
            || value.contains("/*")
            || value.contains("*/")
            || !priority_is_proven(key, value)
        {
            return None;
        }
        if let Some(&index) = indices.get(key) {
            *properties.get_mut(index)? = (key, value);
        } else {
            indices.insert(key, properties.len());
            properties.push((key, value));
        }
    }
    let mut css = String::default();
    for (key, value) in properties {
        if key.starts_with("--") {
            css.push_str(key);
        } else {
            // Vue's /\B([A-Z])/g inserts a dash only after a word
            // character; an initial/vendor or hyphen-following capital
            // is lowercased without inserting an extra dash.
            let mut previous_word = false;
            for c in key.chars() {
                if c.is_ascii_uppercase() && previous_word {
                    css.push('-');
                }
                css.push(c.to_ascii_lowercase());
                previous_word = c.is_ascii_alphanumeric() || c == '_';
            }
        }
        css.push(':');
        css.push_str(value);
        css.push(';');
    }
    Some(css)
}

fn priority_is_proven(key: &str, value: &str) -> bool {
    let Some((stem, token)) = value.split_once('!') else {
        return true;
    };
    // Vue's runtime uses the case-sensitive, end-anchored /\s*!important$/,
    // whereas CSS has a different whitespace and priority-token grammar.
    // Keep every unproved spelling on the runtime path. ASCII also excludes
    // NBSP/FEFF and other JS whitespace that CSS does not strip here.
    // Custom properties and empty stems retain their CSSOM setter behavior.
    !key.starts_with("--")
        && value.is_ascii()
        && token == "important"
        && stem.bytes().any(|byte| byte != b' ')
}

fn style_key(key: &str) -> bool {
    // These are CSSOM/Object interface members, not ordinary CSS properties.
    // In particular cssText replaces declarations and cssFloat is an alias
    // whose CSS spelling is not the result of hyphenating the JS key.
    if matches!(
        key,
        "__proto__"
            | "prototype"
            | "constructor"
            | "cssText"
            | "cssFloat"
            | "length"
            | "parentRule"
            | "item"
            | "getPropertyValue"
            | "getPropertyPriority"
            | "setProperty"
            | "removeProperty"
            | "getPropertyCSSValue"
            | "getPropertyShorthand"
            | "isPropertyImplicit"
            | "toString"
            | "toLocaleString"
            | "valueOf"
            | "hasOwnProperty"
            | "isPrototypeOf"
            | "propertyIsEnumerable"
            | "__defineGetter__"
            | "__defineSetter__"
            | "__lookupGetter__"
            | "__lookupSetter__"
    ) {
        return false;
    }
    let custom = key.starts_with("--");
    let name = key.strip_prefix("--").unwrap_or(key);
    if !custom
        && (!name.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
            || ["webkit", "moz", "ms", "o"].iter().any(|prefix| {
                name.strip_prefix(prefix)
                    .and_then(|suffix| suffix.as_bytes().first())
                    .is_some_and(u8::is_ascii_uppercase)
            }))
    {
        return false;
    }
    !name.is_empty()
        && name.as_bytes().first().is_some_and(u8::is_ascii_alphabetic)
        && name
            .bytes()
            .all(|c| c.is_ascii_alphabetic() || matches!(c, b'-' | b'_'))
}
