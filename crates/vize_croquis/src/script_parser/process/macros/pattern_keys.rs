//! The keys a destructuring pattern reads from its source object.

use oxc_ast::ast::{ObjectPattern, PropertyKey};
use vize_carton::CompactString;

use super::super::bindings::get_binding_pattern_name;

pub(super) fn collect_object_pattern_keys(obj: &ObjectPattern<'_>) -> Vec<CompactString> {
    let mut keys = Vec::new();

    for prop in obj.properties.iter() {
        let key_name = match &prop.key {
            PropertyKey::StaticIdentifier(id) => Some(id.name.as_str()),
            PropertyKey::StringLiteral(s) => Some(s.value.as_str()),
            _ => None,
        };

        if let Some(key_name) = key_name {
            keys.push(CompactString::new(key_name));
        } else if let Some(name) = get_binding_pattern_name(&prop.value) {
            keys.push(CompactString::new(&name));
        }
    }

    if obj.rest.is_some() {
        keys.push(CompactString::new("(rest)"));
    }

    keys
}
