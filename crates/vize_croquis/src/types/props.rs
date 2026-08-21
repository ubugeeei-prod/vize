use vize_carton::{CompactString, FxHashSet};

use super::{TypeProperty, TypeResolver, is_valid_identifier};

mod members;

impl TypeResolver {
    /// Extract properties from type arguments.
    ///
    /// Handles inline object types and type references resolved from the local
    /// type definition store.
    pub fn extract_properties(&self, type_args: &str) -> Vec<TypeProperty> {
        let mut resolving = FxHashSet::default();
        let mut seen = FxHashSet::default();
        self.extract_properties_inner(type_args, &mut resolving, &mut seen)
    }

    fn extract_properties_inner(
        &self,
        type_args: &str,
        resolving: &mut FxHashSet<CompactString>,
        seen: &mut FxHashSet<CompactString>,
    ) -> Vec<TypeProperty> {
        let content = type_args.trim();
        if let Some((base, picked)) = parse_pick_type(content) {
            let mut local_seen = FxHashSet::default();
            return self
                .extract_properties_inner(base, resolving, &mut local_seen)
                .into_iter()
                .filter(|prop| {
                    picked
                        .iter()
                        .any(|name| name.as_str() == prop.name.as_str())
                })
                .collect();
        }

        let (resolved_name, resolved_content) = if content.starts_with('{') {
            let body = if content.ends_with('}') {
                &content[1..content.len() - 1]
            } else {
                content
            };
            (None, body)
        } else {
            let lookup = strip_generic_params(content);
            if let Some(body) = self.definitions.resolve(lookup) {
                let body = body.trim();
                let body = if body.starts_with('{') && body.ends_with('}') {
                    &body[1..body.len() - 1]
                } else {
                    body
                };
                (Some(CompactString::new(lookup)), body)
            } else {
                return Vec::new();
            }
        };

        let mut properties = Vec::new();
        if let Some(name) = resolved_name {
            if !resolving.insert(name.clone()) {
                return Vec::new();
            }
            for heritage in self.definitions.interface_extends(&name) {
                let mut local_seen = FxHashSet::default();
                push_unique_properties(
                    &mut properties,
                    self.extract_properties_inner(heritage.as_str(), resolving, &mut local_seen),
                    seen,
                );
            }
            resolving.remove(&name);
        }

        push_unique_properties(
            &mut properties,
            self.parse_type_members(resolved_content),
            seen,
        );
        properties
    }

    fn parse_type_members(&self, content: &str) -> Vec<TypeProperty> {
        members::parse(self, content)
    }

    fn parse_single_property(&self, segment: &str) -> Option<TypeProperty> {
        let trimmed = segment.trim();
        if trimmed.is_empty() {
            return None;
        }

        let colon_pos = trimmed.find(':')?;
        let name_part = &trimmed[..colon_pos];
        let type_part = &trimmed[colon_pos + 1..];
        let optional = name_part.ends_with('?');
        let name = name_part.trim().trim_end_matches('?').trim();

        if name.is_empty() || !is_valid_identifier(name) {
            return None;
        }

        Some(TypeProperty {
            name: CompactString::new(name),
            prop_type: Some(CompactString::new(type_part.trim())),
            optional,
        })
    }
}

fn push_unique_properties(
    out: &mut Vec<TypeProperty>,
    properties: Vec<TypeProperty>,
    seen: &mut FxHashSet<CompactString>,
) {
    for property in properties {
        if seen.insert(property.name.clone()) {
            out.push(property);
        }
    }
}

fn strip_generic_params(type_name: &str) -> &str {
    match type_name.find('<') {
        Some(pos) => type_name[..pos].trim(),
        None => type_name.trim(),
    }
}

fn parse_pick_type(source: &str) -> Option<(&str, Vec<CompactString>)> {
    let inner = source.strip_prefix("Pick<")?.strip_suffix('>')?;
    let args = split_top_level_comma(inner);
    if args.len() != 2 {
        return None;
    }
    let keys = extract_string_literal_union(args[1]);
    (!keys.is_empty()).then_some((args[0].trim(), keys))
}

fn split_top_level_comma(source: &str) -> Vec<&str> {
    let mut args = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;
    let mut quote = None;
    let mut escape = false;
    for (index, ch) in source.char_indices() {
        if let Some(quote_ch) = quote {
            if escape {
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }
        match ch {
            '"' | '\'' => quote = Some(ch),
            '<' | '{' | '(' | '[' => depth += 1,
            '>' | '}' | ')' | ']' => depth -= 1,
            ',' if depth == 0 => {
                args.push(source[start..index].trim());
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    args.push(source[start..].trim());
    args
}

fn extract_string_literal_union(source: &str) -> Vec<CompactString> {
    let mut names = Vec::new();
    let mut chars = source.char_indices();
    while let Some((start, ch)) = chars.next() {
        if ch != '"' && ch != '\'' {
            continue;
        }
        let quote = ch;
        let mut escape = false;
        for (end, inner) in chars.by_ref() {
            if escape {
                escape = false;
                continue;
            }
            if inner == '\\' {
                escape = true;
                continue;
            }
            if inner == quote {
                names.push(CompactString::new(&source[start + quote.len_utf8()..end]));
                break;
            }
        }
    }
    names
}
