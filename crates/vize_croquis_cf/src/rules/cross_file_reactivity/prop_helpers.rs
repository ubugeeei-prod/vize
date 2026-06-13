use super::name_helpers::{component_names_match, prop_names_match};
use super::path_helpers::import_targets_path;
use super::types::ReactivityLossReason;
use crate::registry::ModuleEntry;
use vize_carton::CompactString;
use vize_croquis::reactivity::ReactivityLossKind;

#[derive(Debug, Clone)]
pub(super) struct PropLoss {
    pub(super) offset: u32,
    pub(super) reason: ReactivityLossReason,
}

pub(super) fn reactive_source_from_expression<'a>(
    analysis: &'a vize_croquis::Croquis,
    expression: &str,
) -> Option<&'a vize_croquis::reactivity::ReactiveSource> {
    let root = expression_root_identifier(expression)?;
    analysis.reactivity.lookup(root)
}

fn expression_root_identifier(expression: &str) -> Option<&str> {
    let expression = expression.trim_start();
    let mut chars = expression.char_indices();
    let (_, first) = chars.next()?;

    if !(first == '_' || first == '$' || first.is_ascii_alphabetic()) {
        return None;
    }

    let mut end = first.len_utf8();
    for (idx, ch) in chars {
        if ch == '_' || ch == '$' || ch.is_ascii_alphanumeric() {
            end = idx + ch.len_utf8();
        } else {
            break;
        }
    }

    Some(&expression[..end])
}

pub(super) fn prop_reactivity_loss(
    analysis: &vize_croquis::Croquis,
    prop_name: &str,
) -> Option<PropLoss> {
    for loss in analysis.reactivity.losses() {
        match &loss.kind {
            ReactivityLossKind::PropsDestructure { .. } => {}
            ReactivityLossKind::ReactiveDestructure {
                destructured_props, ..
            } if prop_list_contains(destructured_props, prop_name) => {
                return Some(PropLoss {
                    offset: loss.start,
                    reason: ReactivityLossReason::Destructured {
                        props: destructured_props.clone(),
                    },
                });
            }
            ReactivityLossKind::ReactivePropertyExtract {
                prop_name: extracted,
                ..
            } if prop_names_match(extracted.as_str(), prop_name) => {
                return Some(PropLoss {
                    offset: loss.start,
                    reason: ReactivityLossReason::DirectExtraction,
                });
            }
            ReactivityLossKind::FunctionArgumentExtract {
                source_name,
                argument_name,
                ..
            } if reactivity_loss_source_matches_prop(source_name.as_str(), prop_name)
                || reactivity_loss_source_matches_prop(argument_name.as_str(), prop_name) =>
            {
                return Some(PropLoss {
                    offset: loss.start,
                    reason: ReactivityLossReason::DirectExtraction,
                });
            }
            ReactivityLossKind::GetterCallExtract {
                source_name,
                getter_name,
                ..
            } if reactivity_loss_source_matches_prop(source_name.as_str(), prop_name)
                || prop_names_match(getter_name.as_str(), prop_name) =>
            {
                return Some(PropLoss {
                    offset: loss.start,
                    reason: ReactivityLossReason::DirectExtraction,
                });
            }
            ReactivityLossKind::PlainValueAlias {
                source_name,
                alias_name,
                target_name,
            } if alias_name == "<mutation>"
                && (reactivity_loss_source_matches_prop(source_name.as_str(), prop_name)
                    || reactivity_loss_source_matches_prop(target_name.as_str(), prop_name)) =>
            {
                return Some(PropLoss {
                    offset: loss.start,
                    reason: ReactivityLossReason::NonReactiveIntermediate {
                        intermediate: target_name.clone(),
                    },
                });
            }
            ReactivityLossKind::PlainValueAlias {
                source_name,
                alias_name,
                target_name,
            } if reactivity_loss_source_matches_prop(source_name.as_str(), prop_name)
                || reactivity_loss_source_matches_prop(alias_name.as_str(), prop_name)
                || prop_names_match(target_name.as_str(), prop_name) =>
            {
                return Some(PropLoss {
                    offset: loss.start,
                    reason: ReactivityLossReason::NonReactiveIntermediate {
                        intermediate: target_name.clone(),
                    },
                });
            }
            _ => {}
        }
    }

    None
}

fn prop_list_contains(props: &[CompactString], prop_name: &str) -> bool {
    props
        .iter()
        .any(|prop| prop.as_str() == "(rest)" || prop_names_match(prop.as_str(), prop_name))
}

fn reactivity_loss_source_matches_prop(source_name: &str, prop_name: &str) -> bool {
    if prop_names_match(source_name, prop_name) {
        return true;
    }

    let Some(rest) = source_name.strip_prefix("props.") else {
        return false;
    };
    let first_segment = rest.split(['.', '[', '?', '!']).next().unwrap_or(rest);
    prop_names_match(first_segment, prop_name)
}

pub(super) fn component_usage_targets_child(
    usage_name: &str,
    child_entry: &ModuleEntry,
    aliases: &[CompactString],
) -> bool {
    child_entry
        .component_name
        .as_deref()
        .is_some_and(|component_name| component_names_match(usage_name, component_name))
        || aliases
            .iter()
            .any(|alias| component_names_match(usage_name, alias.as_str()))
}

pub(super) fn imported_aliases_for_child(
    parent_entry: &ModuleEntry,
    child_entry: &ModuleEntry,
) -> Vec<CompactString> {
    let parent_dir = parent_entry.path.parent();
    let mut aliases = Vec::new();

    for scope in parent_entry.analysis.scopes.iter() {
        let vize_croquis::ScopeData::ExternalModule(data) = scope.data() else {
            continue;
        };

        if !import_targets_path(data.source.as_str(), parent_dir, child_entry.path.as_path()) {
            continue;
        }

        aliases.extend(scope.bindings().map(|(name, _)| CompactString::new(name)));
    }

    aliases
}
