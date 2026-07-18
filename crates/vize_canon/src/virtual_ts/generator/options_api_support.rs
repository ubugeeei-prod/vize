use oxc_syntax::identifier::is_identifier_part;
use vize_carton::String;
use vize_croquis::{Croquis, OptionGroup};

pub(super) use crate::options_api_setup_spread::is_safe_value_identifier;
use crate::virtual_ts::props::OptionsApiPropsSource;

pub(super) fn follows_default_keyword(rest: &str) -> bool {
    rest.chars()
        .next()
        .is_none_or(|ch| ch != '\\' && !is_identifier_part(ch))
}

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

pub(super) fn props_source_from_object(source: &str) -> OptionsApiPropsSource {
    OptionsApiPropsSource::DeferredObject(String::from(source))
}

#[cfg(test)]
mod tests {
    use super::follows_default_keyword;

    #[test]
    fn default_export_boundary_accepts_punctuation_but_not_identifiers() {
        for rest in ["", " ", "{", "(", "/* comment */{"] {
            assert!(follows_default_keyword(rest), "expected boundary: {rest}");
        }
        for rest in ["Thing", "π", "\\u0061"] {
            assert!(
                !follows_default_keyword(rest),
                "identifier continuation is not a boundary: {rest}"
            );
        }
    }
}
