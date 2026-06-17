use oxc_ast::ast::{ObjectExpression, ObjectPropertyKind};
use vize_carton::String;
use vize_croquis::{Croquis, OptionGroup};

use crate::virtual_ts::props::OptionsApiPropsSource;

pub(super) fn extend_options_api_descriptor_names<'a>(
    names: &mut Vec<&'a str>,
    summary: &'a Croquis,
) {
    let Some(descriptor) = summary.options_descriptor.as_ref() else {
        return;
    };
    names.extend(descriptor.members.iter().filter_map(|member| {
        matches!(
            member.group,
            OptionGroup::Props
                | OptionGroup::Inject
                | OptionGroup::Computed
                | OptionGroup::Methods
                | OptionGroup::Data
                | OptionGroup::Setup
        )
        .then_some(member.name.as_str())
        .filter(|name| is_safe_value_identifier(name))
    }));
}

pub(super) fn props_source_from_object(
    object: &ObjectExpression<'_>,
    source: &str,
) -> OptionsApiPropsSource {
    let source = String::from(source);
    if object
        .properties
        .iter()
        .any(|property| matches!(property, ObjectPropertyKind::SpreadProperty(_)))
    {
        OptionsApiPropsSource::DeferredObject(source)
    } else {
        OptionsApiPropsSource::Object(source)
    }
}

pub(super) fn is_safe_value_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_' || first == '$') {
        return false;
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '$')
}
