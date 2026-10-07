use oxc_ast::ast::{ObjectPattern, PropertyKey};

use super::{CompactString, ReactiveValueOrigin, ScriptParseResult, get_binding_pattern_name};

pub(super) fn record_object_pattern_property_origins(
    result: &mut ScriptParseResult,
    obj: &ObjectPattern<'_>,
    source_name: CompactString,
) {
    for prop in obj.properties.iter() {
        let prop_name = match &prop.key {
            PropertyKey::StaticIdentifier(id) => CompactString::new(id.name.as_str()),
            PropertyKey::StringLiteral(s) => CompactString::new(s.value.as_str()),
            _ => {
                let Some(local_name) = get_binding_pattern_name(&prop.value) else {
                    continue;
                };
                CompactString::new(&local_name)
            }
        };

        let Some(local_name) = get_binding_pattern_name(&prop.value) else {
            continue;
        };

        result.record_reactive_origin(
            CompactString::new(&local_name),
            ReactiveValueOrigin::ReactiveProperty {
                source_name: source_name.clone(),
                prop_name,
            },
        );
    }

    if let Some(rest) = &obj.rest
        && let Some(local_name) = get_binding_pattern_name(&rest.argument)
    {
        result.record_reactive_origin(
            CompactString::new(&local_name),
            ReactiveValueOrigin::ReactiveProperty {
                source_name,
                prop_name: CompactString::new("(rest)"),
            },
        );
    }
}
