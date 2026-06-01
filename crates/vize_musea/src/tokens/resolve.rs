use serde_json::Value;
use vize_carton::{FxHashMap, FxHashSet, String, ToCompactString};

use super::types::{DesignToken, FlattenedToken, ResolvedTokens, TokenCategory, ValidationResult};

const MAX_RESOLVE_DEPTH: usize = 10;

pub fn build_token_map(categories: &[TokenCategory]) -> FxHashMap<String, DesignToken> {
    let mut map = FxHashMap::default();
    append_token_map(&mut map, categories, &[]);
    map
}

pub fn resolve_token_categories(mut categories: Vec<TokenCategory>) -> ResolvedTokens {
    let token_map = build_token_map(&categories);
    resolve_references(&mut categories, &token_map);
    let resolved_map = build_token_map(&categories);
    let mut primitive_count = 0;
    let mut semantic_count = 0;
    for token in resolved_map.values() {
        if token.tier.as_deref() == Some("semantic") {
            semantic_count += 1;
        } else {
            primitive_count += 1;
        }
    }

    ResolvedTokens {
        categories,
        token_count: resolved_map.len() as u32,
        token_map: resolved_map,
        primitive_count,
        semantic_count,
    }
}

pub fn flatten_token_categories(categories: &[TokenCategory]) -> Vec<FlattenedToken> {
    let mut flattened = Vec::new();
    append_flattened(&mut flattened, categories, &[]);
    flattened
}

pub fn validate_reference(
    token_map: &FxHashMap<String, DesignToken>,
    reference: &str,
    self_path: Option<&str>,
) -> ValidationResult {
    if !token_map.contains_key(reference) {
        return invalid(vize_carton::cstr!(
            "Reference target \"{reference}\" does not exist"
        ));
    }

    let mut visited = FxHashSet::default();
    if let Some(self_path) = self_path {
        visited.insert(self_path.to_compact_string());
    }
    let mut current = reference.to_compact_string();
    let mut depth = 0;

    while depth < MAX_RESOLVE_DEPTH {
        if !visited.insert(current.clone()) {
            return invalid(vize_carton::cstr!(
                "Circular reference detected at \"{current}\""
            ));
        }

        let Some(target) = token_map.get(&current) else {
            break;
        };
        let Some(next) = reference_name(&target.value) else {
            break;
        };
        current = next.to_compact_string();
        depth += 1;
    }

    if depth >= MAX_RESOLVE_DEPTH {
        return invalid("Reference chain too deep (max 10)".into());
    }

    ValidationResult {
        valid: true,
        error: None,
    }
}

pub fn find_dependent_tokens(
    token_map: &FxHashMap<String, DesignToken>,
    target_path: &str,
) -> Vec<String> {
    let mut dependents = Vec::new();
    for (path, token) in token_map {
        if reference_name(&token.value) == Some(target_path) {
            dependents.push(path.clone());
        }
    }
    dependents
}

fn append_token_map(
    map: &mut FxHashMap<String, DesignToken>,
    categories: &[TokenCategory],
    prefix: &[String],
) {
    for category in categories {
        let cat_key = category_key(&category.name);
        let mut path = prefix.to_vec();
        path.push(cat_key);

        for (name, token) in &category.tokens {
            let mut dot_path = join_path(&path);
            if !dot_path.is_empty() {
                dot_path.push('.');
            }
            dot_path.push_str(name);
            map.insert(dot_path, token.clone());
        }

        append_token_map(map, &category.subcategories, &path);
    }
}

fn append_flattened(
    flattened: &mut Vec<FlattenedToken>,
    categories: &[TokenCategory],
    parent_path: &[String],
) {
    for category in categories {
        let mut category_path = parent_path.to_vec();
        category_path.push(category.name.clone());

        for (name, token) in &category.tokens {
            let mut path = join_path(&category_path);
            if !path.is_empty() {
                path.push('.');
            }
            path.push_str(name);
            flattened.push(FlattenedToken {
                name: name.clone(),
                path,
                category_path: category_path.clone(),
                value: token.value.clone(),
                token_type: token.token_type.clone(),
                description: token.description.clone(),
            });
        }

        append_flattened(flattened, &category.subcategories, &category_path);
    }
}

fn resolve_references(
    categories: &mut [TokenCategory],
    token_map: &FxHashMap<String, DesignToken>,
) {
    for category in categories {
        for token in category.tokens.values_mut() {
            resolve_token(token, token_map);
        }
        resolve_references(&mut category.subcategories, token_map);
    }
}

fn resolve_token(token: &mut DesignToken, token_map: &FxHashMap<String, DesignToken>) {
    if let Some(reference) = reference_name(&token.value) {
        token.tier.get_or_insert_with(|| "semantic".into());
        token.reference = Some(reference.into());
        token.resolved_value = resolve_value(reference, token_map, 0, &mut FxHashSet::default());
    } else {
        token.tier.get_or_insert_with(|| "primitive".into());
    }
}

fn resolve_value(
    reference: &str,
    token_map: &FxHashMap<String, DesignToken>,
    depth: usize,
    visited: &mut FxHashSet<String>,
) -> Option<Value> {
    if depth >= MAX_RESOLVE_DEPTH || !visited.insert(reference.into()) {
        return None;
    }

    let target = token_map.get(reference)?;
    if let Some(next) = reference_name(&target.value) {
        return resolve_value(next, token_map, depth + 1, visited);
    }
    Some(target.value.clone())
}

fn reference_name(value: &Value) -> Option<&str> {
    let value = value.as_str()?;
    (value.len() > 2 && value.starts_with('{') && value.ends_with('}'))
        .then(|| &value[1..value.len() - 1])
}

fn category_key(name: &str) -> String {
    let mut key = String::new("");
    let mut pending_dash = false;
    for ch in name.chars() {
        if ch.is_whitespace() {
            pending_dash = !key.is_empty();
            continue;
        }
        if pending_dash {
            key.push('-');
            pending_dash = false;
        }
        key.push(ch.to_ascii_lowercase());
    }
    key
}

fn join_path(parts: &[String]) -> String {
    let mut path = String::new("");
    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            path.push('.');
        }
        path.push_str(part);
    }
    path
}

fn invalid(error: String) -> ValidationResult {
    ValidationResult {
        valid: false,
        error: Some(error),
    }
}
