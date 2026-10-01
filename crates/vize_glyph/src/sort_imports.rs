//! Convert public Oxfmt-shaped settings to the pinned native formatter options.
use crate::FormatError;
pub use oxc_formatter::SortImportsOptions as ImportSortOptions;
use oxc_formatter::{CustomGroupDefinition, GroupEntry, ImportModifier, ImportSelector};
use vize_l0::{
    ToCompactString,
    config::{ImportSortGroup, SortImportsSetting},
};

/// Validate and prepare sorting once, before formatting any source files.
pub fn resolve_sort_imports(
    setting: Option<&SortImportsSetting>,
) -> Result<Option<ImportSortOptions>, FormatError> {
    let config = match setting {
        None | Some(SortImportsSetting::Disabled(false)) => return Ok(None),
        Some(SortImportsSetting::Config(config)) => config,
        _ => {
            return Err(invalid(
                "sortImports must be false or a configuration object",
            ));
        }
    };
    let mut options = ImportSortOptions::default();
    if let Some(value) = config.partition_by_newline {
        options.partition_by_newline = value;
    }
    if let Some(value) = config.partition_by_comment {
        options.partition_by_comment = value;
    }
    if let Some(value) = config.sort_side_effects {
        options.sort_side_effects = value;
    }
    if let Some(value) = config.ignore_case {
        options.ignore_case = value;
    }
    if let Some(value) = config.newlines_between {
        options.newlines_between = value;
    }
    if let Some(value) = &config.order {
        options.order = value
            .parse()
            .map_err(|_| invalid("sortImports.order must be asc or desc"))?;
    }
    if let Some(value) = &config.internal_pattern {
        options.internal_pattern = value.iter().map(|v| v.as_str().into()).collect();
    }
    if let Some(groups) = &config.groups {
        options.groups.clear();
        let mut pending_boundary = None;
        for group in groups {
            let names = match group {
                ImportSortGroup::Name(name) => vec![name.as_str()],
                ImportSortGroup::Names(names) => names.iter().map(|name| name.as_str()).collect(),
                ImportSortGroup::Boundary { newlines_between } => {
                    pending_boundary = Some(*newlines_between);
                    continue;
                }
                _ => return Err(invalid("unsupported sortImports group")),
            };
            if !options.groups.is_empty() {
                options
                    .newline_boundary_overrides
                    .push(pending_boundary.take());
            }
            options
                .groups
                .push(names.into_iter().map(GroupEntry::parse).collect());
        }
    }
    if let Some(groups) = &config.custom_groups {
        options.custom_groups = groups
            .iter()
            .map(|group| {
                let selector = group
                    .selector
                    .as_deref()
                    .map(|s| {
                        ImportSelector::parse(s)
                            .ok_or_else(|| invalid("invalid sortImports custom group selector"))
                    })
                    .transpose()?;
                let modifiers = group
                    .modifiers
                    .iter()
                    .map(|m| {
                        ImportModifier::parse(m)
                            .ok_or_else(|| invalid("invalid sortImports custom group modifier"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(CustomGroupDefinition {
                    group_name: group.group_name.as_str().into(),
                    element_name_pattern: group
                        .element_name_pattern
                        .iter()
                        .map(|v| v.as_str().into())
                        .collect(),
                    selector,
                    modifiers,
                })
            })
            .collect::<Result<Vec<_>, FormatError>>()?;
    }
    options.validate().map_err(|message| invalid(&message))?;
    Ok(Some(options))
}

fn invalid(message: &str) -> FormatError {
    FormatError::ScriptFormatError(message.to_compact_string())
}
